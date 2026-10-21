/// The cube sizes the solver facade can describe.
///
/// Describing a size does not mean an engine for it exists; see
/// [`crate::api::Solver::kind`] for which sizes a solver actually handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PuzzleKind {
    Cube2,
    Cube3,
    Cube4,
    Cube5,
    Cube6,
    Cube7,
}

impl PuzzleKind {
    pub const ALL: [PuzzleKind; 6] = [
        PuzzleKind::Cube2,
        PuzzleKind::Cube3,
        PuzzleKind::Cube4,
        PuzzleKind::Cube5,
        PuzzleKind::Cube6,
        PuzzleKind::Cube7,
    ];

    /// The number of cubies along one edge.
    pub fn size(self) -> usize {
        match self {
            PuzzleKind::Cube2 => 2,
            PuzzleKind::Cube3 => 3,
            PuzzleKind::Cube4 => 4,
            PuzzleKind::Cube5 => 5,
            PuzzleKind::Cube6 => 6,
            PuzzleKind::Cube7 => 7,
        }
    }

    /// The number of stickers on one face.
    pub fn face_len(self) -> usize {
        self.size() * self.size()
    }

    /// The number of stickers on the whole cube: `6 * n * n`.
    pub fn facelet_count(self) -> usize {
        6 * self.face_len()
    }

    pub fn of_size(size: usize) -> Option<PuzzleKind> {
        PuzzleKind::ALL.into_iter().find(|kind| kind.size() == size)
    }

    pub fn of_facelet_count(count: usize) -> Option<PuzzleKind> {
        PuzzleKind::ALL
            .into_iter()
            .find(|kind| kind.facelet_count() == count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facelet_counts_follow_six_n_squared() {
        let counts: Vec<_> = PuzzleKind::ALL.iter().map(|k| k.facelet_count()).collect();
        assert_eq!(counts, [24, 54, 96, 150, 216, 294]);
    }

    #[test]
    fn lookups_round_trip() {
        for kind in PuzzleKind::ALL {
            assert_eq!(PuzzleKind::of_size(kind.size()), Some(kind));
            assert_eq!(
                PuzzleKind::of_facelet_count(kind.facelet_count()),
                Some(kind)
            );
        }
        assert_eq!(PuzzleKind::of_size(1), None);
        assert_eq!(PuzzleKind::of_size(8), None);
        assert_eq!(PuzzleKind::of_facelet_count(55), None);
    }
}
