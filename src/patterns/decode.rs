//! Bytes of a pattern table as text. The tables of CubeTwister were exported
//! on a Mac: when the bytes are not UTF-8 they are Mac Roman, in which the
//! group separator `·` is the single byte 0xE1.

/// Which decoding produced the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Utf8,
    MacRoman,
}

impl Encoding {
    pub fn name(self) -> &'static str {
        match self {
            Encoding::Utf8 => "UTF-8",
            Encoding::MacRoman => "Mac Roman",
        }
    }
}

/// The characters of the bytes 0x80 to 0xFF in Mac Roman.
const MAC_ROMAN_HIGH: [char; 128] = [
    'Ä', 'Å', 'Ç', 'É', 'Ñ', 'Ö', 'Ü', 'á', 'à', 'â', 'ä', 'ã', 'å', 'ç', 'é', 'è', //
    'ê', 'ë', 'í', 'ì', 'î', 'ï', 'ñ', 'ó', 'ò', 'ô', 'ö', 'õ', 'ú', 'ù', 'û', 'ü', //
    '†', '°', '¢', '£', '§', '•', '¶', 'ß', '®', '©', '™', '´', '¨', '≠', 'Æ', 'Ø', //
    '∞', '±', '≤', '≥', '¥', 'µ', '∂', '∑', '∏', 'π', '∫', 'ª', 'º', 'Ω', 'æ', 'ø', //
    '¿', '¡', '¬', '√', 'ƒ', '≈', '∆', '«', '»', '…', '\u{a0}', 'À', 'Ã', 'Õ', 'Œ', 'œ', //
    '–', '—', '“', '”', '‘', '’', '÷', '◊', 'ÿ', 'Ÿ', '⁄', '€', '‹', '›', 'ﬁ', 'ﬂ', //
    '‡', '·', '‚', '„', '‰', 'Â', 'Ê', 'Á', 'Ë', 'È', 'Í', 'Î', 'Ï', 'Ì', 'Ó', 'Ô', //
    '\u{f8ff}', 'Ò', 'Ú', 'Û', 'Ù', 'ı', 'ˆ', '˜', '¯', '˘', '˙', '˚', '¸', '˝', '˛', 'ˇ',
];

/// The text of a table without BOM, and how it was decoded: UTF-8 when the
/// bytes are valid, else Mac Roman (every byte maps to a character).
pub fn decode_bytes(bytes: &[u8]) -> (String, Encoding) {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    match std::str::from_utf8(bytes) {
        Ok(text) => (text.to_owned(), Encoding::Utf8),
        Err(_) => {
            let text = bytes
                .iter()
                .map(|&b| {
                    if b < 0x80 {
                        char::from(b)
                    } else {
                        MAC_ROMAN_HIGH[usize::from(b - 0x80)]
                    }
                })
                .collect();
            (text, Encoding::MacRoman)
        }
    }
}

/// [`decode_bytes`] without the encoding.
pub fn decode_text(bytes: &[u8]) -> String {
    decode_bytes(bytes).0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_is_kept_and_the_bom_dropped() {
        let (text, how) = decode_bytes(b"\xEF\xBB\xBFa\xC2\xB7b");
        assert_eq!((text.as_str(), how), ("a·b", Encoding::Utf8));
    }

    #[test]
    fn other_bytes_are_mac_roman() {
        let (text, how) = decode_bytes(b"R \xE1 U\x0B\x8A\xD0\xA5");
        assert_eq!(text, "R · U\u{b}ä–•");
        assert_eq!(how, Encoding::MacRoman);
        assert_eq!(decode_text(&[0xFF]), "ˇ");
    }
}
