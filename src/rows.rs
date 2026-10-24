//! Row layout for `solve --icons`: how many icons fit side by side.
//!
//! Pure functions of pixel sizes, so they are tested without a terminal.

use std::ops::Range;

/// Transparent pixels between two icons of a row.
pub const GAP: u32 = 8;

/// Icons per row: as many as fit `width` pixels (at least one), or `forced`.
pub fn per_row(width: u32, icon_width: u32, gap: u32, forced: Option<usize>) -> usize {
    if let Some(columns) = forced {
        return columns.max(1);
    }
    let fit = (u64::from(width) + u64::from(gap)) / (u64::from(icon_width) + u64::from(gap)).max(1);
    usize::try_from(fit).unwrap_or(usize::MAX).max(1)
}

/// Cuts `steps` into rows of at most `per_row` steps; rows never cross the ends of the range.
pub fn rows(steps: Range<usize>, per_row: usize) -> Vec<Range<usize>> {
    let per_row = per_row.max(1);
    let mut out = Vec::new();
    let mut at = steps.start;
    while at < steps.end {
        let end = (at + per_row).min(steps.end);
        out.push(at..end);
        at = end;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_many_icons_as_fit_and_at_least_one() {
        // n icons need n * 100 + (n - 1) * 8 pixels
        assert_eq!(per_row(1000, 100, 8, None), 9);
        assert_eq!(per_row(964, 100, 8, None), 9);
        assert_eq!(per_row(963, 100, 8, None), 8);
        assert_eq!(per_row(50, 100, 8, None), 1);
        assert_eq!(per_row(0, 100, 8, None), 1);
        assert_eq!(per_row(640, 0, 0, None), 640);
    }

    #[test]
    fn columns_override_the_width() {
        assert_eq!(per_row(50, 100, 8, Some(5)), 5);
        assert_eq!(per_row(5000, 100, 8, Some(0)), 1);
    }

    #[test]
    fn rows_wrap_and_stay_inside_the_range() {
        assert_eq!(rows(0..7, 3), vec![0..3, 3..6, 6..7]);
        assert_eq!(rows(5..9, 10), vec![5..9]);
        assert_eq!(rows(4..4, 3), Vec::<Range<usize>>::new());
        assert_eq!(rows(0..2, 0), vec![0..1, 1..2]);
    }
}
