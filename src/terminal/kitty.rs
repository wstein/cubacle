//! Asking the terminal whether it speaks the Kitty graphics protocol.
//!
//! Two queries and a sentinel go out in one write. The first is the neutral
//! form `ESC _ G i=31,q=1 ESC \` (no payload, nothing stored); real terminals
//! answer it with an error reply such as `EINVAL: dimensions required`. The
//! second is the query of the protocol specification (`a=q` with a 1x1 RGB
//! pixel), for terminals that ignore malformed commands. A graphics reply that
//! carries `i=31` proves the protocol is understood, an error reply as well as
//! `OK`. The sentinel is `CSI c` (DA1), which nearly every terminal answers
//! after the earlier queries: when it comes without any graphics reply the
//! terminal has no graphics. Reading goes on until the sentinel's answer or the
//! timeout, so no reply byte is left in the input of the shell.
//!
//! The I/O is behind [`Wire`]; nothing here needs a tty.

use super::parse::parse_replies;

/// The neutral query: stores nothing, always answered with an error.
pub const NEUTRAL_QUERY: &str = "\x1b_Gi=31,q=1\x1b\\";
/// The query of the specification: answered `OK` where the protocol exists.
pub const SPEC_QUERY: &str = "\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\";
/// Primary device attributes, the sentinel.
pub const SENTINEL: &str = "\x1b[c";

/// Reads that may come back empty before the wait ends (100 ms each).
const IDLE_READS: u32 = 2;
/// A hard limit on reads, so that a stream of garbage cannot hold the program.
const MAX_READS: u32 = 64;

/// The terminal as the probe sees it.
pub trait Wire {
    /// Writes the bytes; false when that fails.
    fn send(&mut self, bytes: &[u8]) -> bool;
    /// Waits up to about 100 ms; an empty answer is a timeout, `None` an error.
    fn read(&mut self) -> Option<Vec<u8>>;
}

/// Whether `bytes` hold a graphics reply (`ESC _ G`, keys, `;`) with the id 31.
pub fn graphics_reply(bytes: &[u8]) -> bool {
    let mut rest = bytes;
    while let Some(at) = rest.windows(3).position(|w| w == b"\x1b_G") {
        let body = &rest[at + 3..];
        let end = body
            .iter()
            .position(|&b| b == b';' || b == 0x1b)
            .unwrap_or(body.len());
        if body.get(end) == Some(&b';') && body[..end].split(|&b| b == b',').any(|k| k == b"i=31") {
            return true;
        }
        rest = body;
    }
    false
}

/// Sends the queries and reads until the sentinel is answered; true when the
/// terminal answered a graphics query.
pub fn probe_with(wire: &mut dyn Wire) -> bool {
    let queries = format!("{NEUTRAL_QUERY}{SPEC_QUERY}{SENTINEL}");
    if !wire.send(queries.as_bytes()) {
        return false;
    }
    let mut seen = Vec::new();
    let (mut idle, mut reads) = (0, 0);
    while idle < IDLE_READS && reads < MAX_READS && seen.len() < 4096 {
        reads += 1;
        match wire.read() {
            None => break,
            Some(chunk) if chunk.is_empty() => idle += 1,
            Some(chunk) => {
                idle = 0;
                seen.extend_from_slice(&chunk);
            }
        }
        if parse_replies(&seen).done {
            break;
        }
    }
    graphics_reply(&seen)
}

#[cfg(test)]
mod tests {
    use super::*;

    const KITTY_ERROR: &str = "\x1b_Gi=31;EINVAL:Zero width/height not allowed\x1b\\";
    const HERDR_ERROR: &str = "\x1b_Gi=31;EINVAL: dimensions required\x1b\\";
    const OK: &str = "\x1b_Gi=31;OK\x1b\\";
    const DA1: &str = "\x1b[?62;4;22c";

    /// Gives the chunks one per read, then timeouts; remembers what was sent.
    struct Fake {
        chunks: Vec<Vec<u8>>,
        sent: Vec<u8>,
        reads: u32,
    }

    impl Fake {
        fn new(chunks: &[&str]) -> Fake {
            Fake {
                chunks: chunks.iter().rev().map(|c| c.as_bytes().to_vec()).collect(),
                sent: Vec::new(),
                reads: 0,
            }
        }
    }

    impl Wire for Fake {
        fn send(&mut self, bytes: &[u8]) -> bool {
            self.sent.extend_from_slice(bytes);
            true
        }
        fn read(&mut self) -> Option<Vec<u8>> {
            self.reads += 1;
            Some(self.chunks.pop().unwrap_or_default())
        }
    }

    #[test]
    fn replies_are_recognised() {
        assert!(graphics_reply(OK.as_bytes()));
        assert!(graphics_reply(KITTY_ERROR.as_bytes()));
        assert!(graphics_reply(HERDR_ERROR.as_bytes()));
        // the id as the only key, and among others
        assert!(graphics_reply(b"\x1b_Gi=31;OK"));
        assert!(graphics_reply(b"\x1b_Gi=31,p=1;EBADF\x1b\\"));
        // concatenated, the way kitty answers both queries
        let both = format!("{KITTY_ERROR}{OK}");
        assert!(graphics_reply(both.as_bytes()));
        // not ours or not graphics
        assert!(!graphics_reply(DA1.as_bytes()));
        assert!(!graphics_reply(b"\x1b_Gi=32;OK\x1b\\"));
        assert!(!graphics_reply(b"\x1b_Gi=311;OK\x1b\\"));
        assert!(!graphics_reply(b"garbage i=31;OK"));
        assert!(!graphics_reply(b""));
        // cut before the semicolon: not yet a reply
        assert!(!graphics_reply(b"\x1b_Gi=31"));
    }

    #[test]
    fn the_write_has_both_queries_and_the_sentinel() {
        let mut wire = Fake::new(&[DA1]);
        probe_with(&mut wire);
        let sent = String::from_utf8(wire.sent).unwrap();
        assert_eq!(sent, format!("{NEUTRAL_QUERY}{SPEC_QUERY}{SENTINEL}"));
        // the neutral form carries no payload and no action
        assert!(!NEUTRAL_QUERY.contains(';') && !NEUTRAL_QUERY.contains("a="));
    }

    #[test]
    fn kitty_answers_twice_and_everything_is_drained() {
        let both = format!("{KITTY_ERROR}{OK}{DA1}");
        let mut wire = Fake::new(&[&both]);
        assert!(probe_with(&mut wire));
        assert!(wire.chunks.is_empty());
        // replies split over reads, also inside an escape sequence
        let mut wire = Fake::new(&[
            "\x1b_Gi=31;EINVAL:Zero wi",
            "dth/height not allowed\x1b",
            "\\\x1b_Gi=31;OK\x1b\\\x1b[?6",
            "2;4c",
        ]);
        assert!(probe_with(&mut wire));
        assert!(wire.chunks.is_empty(), "the sentinel answer was read");
    }

    #[test]
    fn one_error_reply_is_enough() {
        for reply in [KITTY_ERROR, HERDR_ERROR, OK] {
            let text = format!("{reply}{DA1}");
            let mut wire = Fake::new(&[&text]);
            assert!(probe_with(&mut wire), "{reply:?}");
        }
        // the graphics reply and the sentinel answer in separate reads
        let mut wire = Fake::new(&[HERDR_ERROR, DA1]);
        assert!(probe_with(&mut wire));
        assert!(wire.chunks.is_empty());
    }

    #[test]
    fn no_graphics_reply_means_no_support() {
        let mut wire = Fake::new(&[DA1]);
        assert!(!probe_with(&mut wire));
        let mut wire = Fake::new(&["garbage", DA1]);
        assert!(!probe_with(&mut wire));
    }

    #[test]
    fn silence_ends_after_the_idle_reads() {
        let mut wire = Fake::new(&[]);
        assert!(!probe_with(&mut wire));
        assert_eq!(wire.reads, IDLE_READS);
        // a partial reply that never completes does not hang either
        let mut wire = Fake::new(&["\x1b_Gi=31"]);
        assert!(!probe_with(&mut wire));
        assert_eq!(wire.reads, IDLE_READS + 1);
    }

    #[test]
    fn a_failing_wire_gives_false() {
        struct Broken;
        impl Wire for Broken {
            fn send(&mut self, _: &[u8]) -> bool {
                false
            }
            fn read(&mut self) -> Option<Vec<u8>> {
                None
            }
        }
        assert!(!probe_with(&mut Broken));
    }
}
