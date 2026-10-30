//! Optional scaling of positive functional-test budgets, never production limits.
//! CUBACLE_TEST_TIME_FACTOR is a positive integer, defaults to 1, and may not
//! extend a test budget beyond ten minutes. Explicit expiration cases do not use it.

const MAX_BUDGET_MS: u64 = 600_000;

pub fn milliseconds(base: u64) -> u64 {
    scaled(
        base,
        std::env::var("CUBACLE_TEST_TIME_FACTOR").ok().as_deref(),
    )
}

fn scaled(base: u64, value: Option<&str>) -> u64 {
    let factor = value
        .and_then(|text| text.parse::<u64>().ok())
        .filter(|&factor| factor > 0)
        .unwrap_or(1);
    base.saturating_mul(factor).min(MAX_BUDGET_MS)
}

#[cfg(test)]
mod tests {
    use super::scaled;

    #[test]
    fn defaults_and_invalid_factors_preserve_the_budget() {
        assert_eq!(scaled(1500, None), 1500);
        for value in [
            "",
            "0",
            "-1",
            "1.5",
            "3x",
            "NaN",
            "Infinity",
            "18446744073709551616",
        ] {
            assert_eq!(scaled(1500, Some(value)), 1500, "{value}");
        }
    }

    #[test]
    fn scales_valid_integer_factors() {
        assert_eq!(scaled(1500, Some("1")), 1500);
        assert_eq!(scaled(1500, Some("3")), 4500);
        assert_eq!(scaled(30_000, Some("3")), 90_000);
    }

    #[test]
    fn clamps_large_factors_without_overflow_or_truncation() {
        for value in ["400", "4294967296", "18446744073709551615"] {
            assert_eq!(scaled(1500, Some(value)), 600_000, "{value}");
            assert_eq!(scaled(30_000, Some(value)), 600_000, "{value}");
        }
    }
}
