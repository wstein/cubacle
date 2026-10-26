//! What the terminal can tell: its size in pixels and its background color.
//!
//! The answers come from `TIOCGWINSZ` and, when that has no pixel size, from
//! the replies to `CSI 14 t` and `CSI 16 t`; the background comes from `OSC 11`.
//! The queries go through `/dev/tty` in a temporary raw mode that is always
//! restored. A trailing DA1 query (`CSI c`) is answered by nearly every
//! terminal, so one that ignores the other queries still returns at once. The
//! parsing is in `parse` and has no tty.
//!
//! Nothing is asked when standard input or output is not a terminal, or inside
//! `tmux` or `screen`, which would swallow the queries or need a passthrough
//! that this program does not use; the defaults apply there.

mod kitty;
mod parse;

pub use parse::Color;
use parse::{usable_width, ASSUMED_CELL};

/// Everything that is known about the terminal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Terminal {
    /// Usable width in pixels.
    pub width_px: u32,
    /// Size of one cell in pixels.
    pub cell: (u32, u32),
    /// The background color, when the terminal reported one.
    pub background: Option<Color>,
}

impl Default for Terminal {
    fn default() -> Self {
        Terminal {
            width_px: 80 * ASSUMED_CELL.0,
            cell: ASSUMED_CELL,
            background: None,
        }
    }
}

/// The environment says a multiplexer sits between us and the terminal.
pub fn behind_multiplexer(env: &dyn Fn(&str) -> Option<String>) -> bool {
    env("TMUX").is_some()
        || env("STY").is_some()
        || env("TERM").is_some_and(|t| t.starts_with("screen") || t.starts_with("tmux"))
}

fn columns_from(env: &dyn Fn(&str) -> Option<String>) -> u32 {
    env("COLUMNS")
        .and_then(|c| c.parse().ok())
        .filter(|&c| c > 0)
        .unwrap_or(80)
}

/// Asks the terminal; falls back to defaults where it cannot or must not.
/// `want_background` is false when the color is not needed (`--background`).
pub fn probe(want_background: bool) -> Terminal {
    use std::io::IsTerminal;
    let env = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
    let mut columns = columns_from(&env);
    let mut ioctl_pixels = None;
    let mut cell = None;
    let mut area = None;
    let mut background = None;
    if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
        if let Some((cols, rows, x, y)) = imp::window_size() {
            if cols > 0 {
                columns = u32::from(cols);
            }
            if x > 0 && y > 0 && cols > 0 && rows > 0 {
                ioctl_pixels = Some(u32::from(x));
                cell = Some((u32::from(x / cols), u32::from(y / rows)));
            }
        }
        if !behind_multiplexer(&env) {
            let replies = imp::query(want_background, ioctl_pixels.is_none());
            background = replies.background;
            area = replies.area;
            if cell.is_none() {
                cell = replies.cell;
            }
        }
    }
    Terminal {
        width_px: usable_width(columns, ioctl_pixels, area, cell),
        cell: cell.filter(|c| c.0 > 0 && c.1 > 0).unwrap_or(ASSUMED_CELL),
        background,
    }
}

/// Asks the terminal whether it speaks the Kitty graphics protocol, at most
/// once per process. False when standard input or output is not a terminal,
/// inside `tmux` or `screen`, on timeout and on every error.
pub fn supports_kitty() -> bool {
    use std::io::IsTerminal;
    static ANSWER: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ANSWER.get_or_init(|| {
        let env = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
        std::io::stdin().is_terminal()
            && std::io::stdout().is_terminal()
            && !behind_multiplexer(&env)
            && imp::supports_kitty()
    })
}

#[cfg(unix)]
mod imp {
    use super::parse::{parse_replies, Replies};
    use rustix::termios::{
        tcgetattr, tcgetwinsize, tcsetattr, LocalModes, OptionalActions, SpecialCodeIndex, Termios,
    };
    use std::fs::{File, OpenOptions};
    use std::io::{Read, Write};

    pub fn window_size() -> Option<(u16, u16, u16, u16)> {
        let size = tcgetwinsize(std::io::stdout()).ok()?;
        Some((size.ws_col, size.ws_row, size.ws_xpixel, size.ws_ypixel))
    }

    /// The tty in raw mode; the saved settings come back when it is dropped, on every path.
    struct RawTty {
        tty: File,
        saved: Termios,
    }

    impl RawTty {
        fn open() -> Option<RawTty> {
            let tty = OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/tty")
                .ok()?;
            let saved = tcgetattr(&tty).ok()?;
            let mut raw = saved.clone();
            raw.local_modes
                .remove(LocalModes::ICANON | LocalModes::ECHO);
            // A read returns after 100 ms without a byte.
            raw.special_codes[SpecialCodeIndex::VMIN] = 0;
            raw.special_codes[SpecialCodeIndex::VTIME] = 1;
            tcsetattr(&tty, OptionalActions::Now, &raw).ok()?;
            Some(RawTty { tty, saved })
        }
    }

    impl Drop for RawTty {
        fn drop(&mut self) {
            // Flush drops replies that arrive late, so they never reach the shell.
            let _ = tcsetattr(&self.tty, OptionalActions::Flush, &self.saved);
        }
    }

    impl super::kitty::Wire for RawTty {
        fn send(&mut self, bytes: &[u8]) -> bool {
            self.tty
                .write_all(bytes)
                .and_then(|()| self.tty.flush())
                .is_ok()
        }

        fn read(&mut self) -> Option<Vec<u8>> {
            let mut buf = [0u8; 256];
            match self.tty.read(&mut buf) {
                Ok(n) => Some(buf[..n].to_vec()),
                Err(_) => None,
            }
        }
    }

    pub fn supports_kitty() -> bool {
        RawTty::open().is_some_and(|mut raw| super::kitty::probe_with(&mut raw))
    }

    pub fn query(background: bool, sizes: bool) -> Replies {
        if !background && !sizes {
            return Replies::default();
        }
        let Some(mut raw) = RawTty::open() else {
            return Replies::default();
        };
        let mut queries = String::new();
        if background {
            queries.push_str("\x1b]11;?\x1b\\");
        }
        if sizes {
            queries.push_str("\x1b[14t\x1b[16t");
        }
        queries.push_str("\x1b[c");
        if raw
            .tty
            .write_all(queries.as_bytes())
            .and_then(|()| raw.tty.flush())
            .is_err()
        {
            return Replies::default();
        }
        let mut seen = Vec::new();
        let mut buf = [0u8; 256];
        while seen.len() < 2048 {
            match raw.tty.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => seen.extend_from_slice(&buf[..n]),
            }
            if parse_replies(&seen).done {
                break;
            }
        }
        parse_replies(&seen)
    }
}

#[cfg(not(unix))]
mod imp {
    use super::parse::Replies;

    pub fn window_size() -> Option<(u16, u16, u16, u16)> {
        None
    }

    pub fn supports_kitty() -> bool {
        false
    }

    pub fn query(_background: bool, _sizes: bool) -> Replies {
        Replies::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiplexers_are_recognized() {
        let with = |name: &'static str, value: &'static str| {
            move |n: &str| (n == name).then(|| value.to_owned())
        };
        assert!(behind_multiplexer(&with("TMUX", "/tmp/x")));
        assert!(behind_multiplexer(&with("STY", "1.pts")));
        assert!(behind_multiplexer(&with("TERM", "screen-256color")));
        assert!(!behind_multiplexer(&with("TERM", "xterm-kitty")));
    }
}
