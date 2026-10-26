//! The label strip: `SYNTHETIC <PRESET>` in 5x7 glyphs on the bottom margin,
//! so a synthetic photo is never mistaken for a camera photo.

use super::color::LABEL_COLOR;
use super::draw::put;

/// The 5x7 glyph of a character, rows top to bottom, `1` is lit.
fn glyph(c: char) -> Option<[&'static str; 7]> {
    Some(match c {
        'A' => [
            "01110", "10001", "10001", "11111", "10001", "10001", "10001",
        ],
        'B' => [
            "11110", "10001", "10001", "11110", "10001", "10001", "11110",
        ],
        'C' => [
            "01110", "10001", "10000", "10000", "10000", "10001", "01110",
        ],
        'D' => [
            "11110", "10001", "10001", "10001", "10001", "10001", "11110",
        ],
        'E' => [
            "11111", "10000", "10000", "11110", "10000", "10000", "11111",
        ],
        'G' => [
            "01110", "10001", "10000", "10111", "10001", "10001", "01111",
        ],
        'H' => [
            "10001", "10001", "10001", "11111", "10001", "10001", "10001",
        ],
        'I' => [
            "01110", "00100", "00100", "00100", "00100", "00100", "01110",
        ],
        'L' => [
            "10000", "10000", "10000", "10000", "10000", "10000", "11111",
        ],
        'M' => [
            "10001", "11011", "10101", "10101", "10001", "10001", "10001",
        ],
        'N' => [
            "10001", "11001", "10101", "10011", "10001", "10001", "10001",
        ],
        'O' => [
            "01110", "10001", "10001", "10001", "10001", "10001", "01110",
        ],
        'R' => [
            "11110", "10001", "10001", "11110", "10100", "10010", "10001",
        ],
        'S' => [
            "01111", "10000", "10000", "01110", "00001", "00001", "11110",
        ],
        'T' => [
            "11111", "00100", "00100", "00100", "00100", "00100", "00100",
        ],
        'W' => [
            "10001", "10001", "10001", "10101", "10101", "11011", "10001",
        ],
        'Y' => [
            "10001", "10001", "01010", "00100", "00100", "00100", "00100",
        ],
        '-' => [
            "00000", "00000", "00000", "11111", "00000", "00000", "00000",
        ],
        _ => return None,
    })
}

/// Draws the label; pixels beyond the right or bottom edge are clipped.
pub(crate) fn draw_label(data: &mut [u8], size: usize, preset_name: &str, margin: usize) {
    let scale = (margin.saturating_sub(4) / 7).max(1);
    let top = size as isize - margin as isize + (margin as isize - 7 * scale as isize) / 2;
    let text = format!("SYNTHETIC {}", preset_name.to_uppercase());
    for (index, c) in text.chars().enumerate() {
        let Some(rows) = glyph(c) else { continue };
        for (gy, row) in rows.iter().enumerate() {
            for (gx, bit) in row.bytes().enumerate() {
                if bit != b'1' {
                    continue;
                }
                for dy in 0..scale {
                    for dx in 0..scale {
                        let x = margin + (index * 6 + gx) * scale + dx;
                        let y = top + (gy * scale + dy) as isize;
                        if x < size && (0..size as isize).contains(&y) {
                            put(data, (y as usize * size + x) * 4, LABEL_COLOR);
                        }
                    }
                }
            }
        }
    }
}
