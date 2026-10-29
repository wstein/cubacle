//! A set of sticker positions: the stickers a target cares about.
//!
//! A partial target is a goal set. Which stickers it constrains is a mask, and
//! an engine can ask whether the mask lies inside the part of the cube it knows
//! how to solve (the centers for the reduction engines, later the edges of a
//! cross for the 3x3x3). The mask is generic: it names positions of one cube
//! size and knows nothing of any engine.

use super::PuzzleKind;

/// A set of sticker positions of one cube size.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Mask {
    kind: PuzzleKind,
    member: Vec<bool>,
}

impl Mask {
    /// The empty mask of `kind`.
    pub fn empty(kind: PuzzleKind) -> Mask {
        Mask {
            kind,
            member: vec![false; kind.facelet_count()],
        }
    }

    /// The mask of the stickers `keep` accepts.
    pub fn from_fn(kind: PuzzleKind, keep: impl Fn(usize) -> bool) -> Mask {
        Mask {
            kind,
            member: (0..kind.facelet_count()).map(keep).collect(),
        }
    }

    /// Every center sticker of `kind`: the inner square of each face, which
    /// includes the fixed face center of an odd cube. Empty below the 3x3x3.
    pub fn centers(kind: PuzzleKind) -> Mask {
        let n = kind.size();
        Mask::from_fn(kind, |position| {
            let at = position % kind.face_len();
            let (row, col) = (at / n, at % n);
            (1..n.saturating_sub(1)).contains(&row) && (1..n.saturating_sub(1)).contains(&col)
        })
    }

    pub fn kind(&self) -> PuzzleKind {
        self.kind
    }

    /// Whether the sticker at `position` is in the mask.
    pub fn contains(&self, position: usize) -> bool {
        self.member.get(position).copied().unwrap_or(false)
    }

    /// How many stickers are in the mask.
    pub fn len(&self) -> usize {
        self.member.iter().filter(|&&m| m).count()
    }

    pub fn is_empty(&self) -> bool {
        !self.member.contains(&true)
    }

    /// The positions in the mask, in increasing order.
    pub fn positions(&self) -> impl Iterator<Item = usize> + '_ {
        self.member
            .iter()
            .enumerate()
            .filter_map(|(position, &m)| m.then_some(position))
    }

    /// Whether every sticker of this mask is also in `other` (same size).
    pub fn is_subset_of(&self, other: &Mask) -> bool {
        self.kind == other.kind
            && self
                .member
                .iter()
                .zip(&other.member)
                .all(|(&mine, &theirs)| !mine || theirs)
    }

    /// The stickers of this mask that are not in `other`.
    pub fn outside(&self, other: &Mask) -> Mask {
        Mask {
            kind: self.kind,
            member: self
                .member
                .iter()
                .zip(&other.member)
                .map(|(&mine, &theirs)| mine && !theirs)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_centers_are_the_inner_square_of_every_face() {
        for (kind, per_face) in [
            (PuzzleKind::Cube2, 0),
            (PuzzleKind::Cube3, 1),
            (PuzzleKind::Cube4, 4),
            (PuzzleKind::Cube5, 9),
            (PuzzleKind::Cube6, 16),
            (PuzzleKind::Cube7, 25),
        ] {
            assert_eq!(Mask::centers(kind).len(), 6 * per_face, "{kind:?}");
        }
        let centers = Mask::centers(PuzzleKind::Cube3);
        assert_eq!(
            centers.positions().collect::<Vec<_>>(),
            [4, 13, 22, 31, 40, 49]
        );
    }

    #[test]
    fn subsets_and_differences() {
        let kind = PuzzleKind::Cube4;
        let centers = Mask::centers(kind);
        let first_face = Mask::from_fn(kind, |p| p < kind.face_len());
        assert!(!first_face.is_subset_of(&centers));
        assert_eq!(first_face.outside(&centers).len(), 12);
        let one = Mask::from_fn(kind, |p| p == 5);
        assert!(one.is_subset_of(&centers) && centers.contains(5));
        assert!(Mask::empty(kind).is_empty());
        assert!(!Mask::empty(kind).is_subset_of(&Mask::empty(PuzzleKind::Cube3)));
    }
}
