//! Evidence about the actual goal and search outcome, independent of presentation.

/// The orientation equivalence of the goal the engine actually searched.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SolveFrame {
    #[default]
    Fixed,
    AnyOrientation,
}

impl SolveFrame {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixed => "fixed",
            Self::AnyOrientation => "any-orientation",
        }
    }
}

/// A bound in HTM for this result's actual goal and frame, never a requested option.
/// Default evidence makes no shortest-path claim. Non-HTM proofs must not populate it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SolveEvidence {
    pub frame: SolveFrame,
    pub proven: bool,
    /// Compact checked storage keeps the copyable result below the large-error threshold.
    /// Every current proof producer has a bound of at most 255 HTM moves.
    pub lower_bound: Option<u32>,
}

impl SolveEvidence {
    pub fn optimal(frame: SolveFrame, length: usize) -> Self {
        let lower_bound = u32::try_from(length).ok();
        Self {
            frame,
            proven: lower_bound.is_some(),
            lower_bound,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_stays_compact_and_never_truncates_a_bound() {
        assert!(std::mem::size_of::<SolveEvidence>() <= 16);
        let exact = SolveEvidence::optimal(SolveFrame::Fixed, 21);
        assert!(exact.proven);
        assert_eq!(exact.lower_bound, Some(21));
        if let Ok(too_large) = usize::try_from(u64::from(u32::MAX) + 1) {
            let absent = SolveEvidence::optimal(SolveFrame::Fixed, too_large);
            assert!(!absent.proven);
            assert_eq!(absent.lower_bound, None);
        }
    }
}
