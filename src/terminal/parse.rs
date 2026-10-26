//! Parsing of terminal replies and the choices made from them. Pure: no tty.

/// A color as the terminal reports it, each channel scaled to 0.0..=1.0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    /// Perceived brightness (Rec. 709 weights on the encoded values), 0.0..=1.0.
    pub fn luminance(self) -> f64 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    /// Light backgrounds are those brighter than half.
    pub fn is_light(self) -> bool {
        self.luminance() > 0.5
    }
}

/// What the terminal answered, in any order.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Replies {
    /// OSC 11: the background color.
    pub background: Option<Color>,
    /// CSI 14 t: width and height of the text area in pixels.
    pub area: Option<(u32, u32)>,
    /// CSI 16 t: width and height of one cell in pixels.
    pub cell: Option<(u32, u32)>,
    /// DA1 arrived: the terminal has answered everything it will answer.
    pub done: bool,
}

/// `rgb:RRRR/GGGG/BBBB` with one to four hex digits per channel.
fn parse_color(text: &str) -> Option<Color> {
    let body = text.strip_prefix("rgb:")?;
    let mut channels = body.split('/').map(|digits| {
        if digits.is_empty() || digits.len() > 4 || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let value = u32::from_str_radix(digits, 16).ok()?;
        let max = 16u32.pow(digits.len() as u32) - 1;
        Some(f64::from(value) / f64::from(max))
    });
    let color = Color {
        r: channels.next()??,
        g: channels.next()??,
        b: channels.next()??,
    };
    channels.next().is_none().then_some(color)
}

fn pair(params: &str, first: &str) -> Option<(u32, u32)> {
    let mut parts = params.split(';');
    (parts.next()? == first).then_some(())?;
    let height = parts.next()?.parse().ok()?;
    let width = parts.next()?.parse().ok()?;
    parts.next().is_none().then_some((width, height))
}

/// Reads every complete reply in `bytes`; anything else is skipped.
pub fn parse_replies(bytes: &[u8]) -> Replies {
    let mut replies = Replies::default();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] != 0x1b {
            i += 1;
            continue;
        }
        match bytes[i + 1] {
            b']' => {
                let rest = &bytes[i + 2..];
                let end = rest
                    .iter()
                    .position(|&b| b == 0x07 || b == 0x1b)
                    .unwrap_or(rest.len());
                if let Ok(text) = std::str::from_utf8(&rest[..end]) {
                    if let Some(color) = text.strip_prefix("11;").and_then(parse_color) {
                        replies.background = Some(color);
                    }
                }
                i += 2 + end;
            }
            b'[' => {
                let rest = &bytes[i + 2..];
                let Some(end) = rest.iter().position(|b| (0x40..=0x7e).contains(b)) else {
                    break;
                };
                let params = std::str::from_utf8(&rest[..end]).unwrap_or("");
                match rest[end] {
                    b't' => {
                        if let Some(size) = pair(params, "4") {
                            replies.area = Some(size);
                        } else if let Some(size) = pair(params, "6") {
                            replies.cell = Some(size);
                        }
                    }
                    b'c' if rest[..end].starts_with(b"?") => replies.done = true,
                    _ => {}
                }
                i += 2 + end + 1;
            }
            _ => i += 1,
        }
    }
    replies
}

/// The cell size used when nothing better is known.
pub const ASSUMED_CELL: (u32, u32) = (8, 16);

/// The width in pixels that icons may use.
///
/// The pixel size from the terminal wins, then the pixel size of a cell times
/// the columns, then `columns * 8` (an assumed cell width).
pub fn usable_width(
    columns: u32,
    ioctl_pixels: Option<u32>,
    area: Option<(u32, u32)>,
    cell: Option<(u32, u32)>,
) -> u32 {
    ioctl_pixels
        .filter(|&p| p > 0)
        .or(area.map(|a| a.0).filter(|&p| p > 0))
        .or(cell.map(|c| c.0 * columns).filter(|&p| p > 0))
        .unwrap_or(columns * ASSUMED_CELL.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn osc_11_in_both_terminators_and_precisions() {
        let dark = parse_replies(b"\x1b]11;rgb:2020/2424/2b2b\x1b\\\x1b[?62;c");
        assert!(dark.done);
        assert!(!dark.background.unwrap().is_light());
        let light = parse_replies(b"\x1b]11;rgb:ff/ff/ff\x07");
        assert!(light.background.unwrap().is_light());
        assert!(!light.done);
        let short = parse_replies(b"\x1b]11;rgb:f/f/f\x07").background.unwrap();
        assert_eq!((short.r, short.g, short.b), (1.0, 1.0, 1.0));
    }

    #[test]
    fn malformed_colors_are_ignored() {
        for bad in [
            &b"\x1b]11;rgb:zz/00/00\x07"[..],
            b"\x1b]11;rgb:00/00\x07",
            b"\x1b]11;rgb:00/00/00/00\x07",
            b"\x1b]11;#ffffff\x07",
            b"\x1b]10;rgb:ff/ff/ff\x07",
        ] {
            assert_eq!(parse_replies(bad).background, None, "{bad:?}");
        }
    }

    #[test]
    fn the_threshold_splits_light_from_dark() {
        let grey = |v: f64| Color { r: v, g: v, b: v };
        assert!(grey(0.6).is_light() && !grey(0.4).is_light());
        let blue = Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
        };
        assert!(!blue.is_light());
    }

    #[test]
    fn size_replies_give_width_and_height() {
        let r = parse_replies(b"\x1b[4;600;1000t\x1b[6;20;10t\x1b[?1;2c");
        assert_eq!(r.area, Some((1000, 600)));
        assert_eq!(r.cell, Some((10, 20)));
        assert!(r.done);
    }

    #[test]
    fn a_da1_only_reply_means_no_answers() {
        let r = parse_replies(b"\x1b[?64;1;2c");
        assert_eq!(
            r,
            Replies {
                done: true,
                ..Replies::default()
            }
        );
        assert_eq!(parse_replies(b"junk \x1b[4;60"), Replies::default());
    }

    #[test]
    fn width_prefers_pixels_and_falls_back_to_columns() {
        assert_eq!(usable_width(80, Some(900), Some((1000, 1)), None), 900);
        assert_eq!(usable_width(80, Some(0), Some((1000, 1)), None), 1000);
        assert_eq!(usable_width(80, None, None, Some((9, 18))), 720);
        assert_eq!(usable_width(80, None, None, None), 640);
    }
}
