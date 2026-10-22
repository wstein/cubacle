//! Origin: Rust re-implementation of the slot conventions of flix-orbit64
//! (`Orbit64.Orbit` and `Orbit64.Net`, Apache-2.0, Werner Stein).
//!
//! The orbit structure follows from the cube size `n` alone. Corners and
//! midges use the standard 3x3x3 slots; wings use the 4x4x4 reference slots at
//! each depth; centres are ordered by facelet index, orbit by orbit. A slot is
//! the list of facelets a piece shows there, the first being the one its
//! orientation counts from. A piece is named by the slot it occupies when the
//! cube is solved, so its colours are the faces of that slot's facelets.

/// Facelet indices of each corner slot on the 3x3x3: URF UFL ULB UBR DFR DLF DBL DRB.
pub(crate) const CORNER_FACELETS: [[usize; 3]; 8] = [
    [8, 9, 20],
    [6, 18, 38],
    [0, 36, 47],
    [2, 45, 11],
    [29, 26, 15],
    [27, 44, 24],
    [33, 53, 42],
    [35, 17, 51],
];

/// Face colours (indices into `URFDLB`) of each corner piece, clockwise.
pub(crate) const CORNER_COLOURS: [[usize; 3]; 8] = [
    [0, 1, 2],
    [0, 2, 4],
    [0, 4, 5],
    [0, 5, 1],
    [3, 2, 1],
    [3, 4, 2],
    [3, 5, 4],
    [3, 1, 5],
];

/// Facelet indices of each midge slot on the 3x3x3: UR UF UL UB DR DF DL DB FR FL BL BR.
const MIDGE_FACELETS: [[usize; 2]; 12] = [
    [5, 10],
    [7, 19],
    [3, 37],
    [1, 46],
    [32, 16],
    [28, 25],
    [30, 43],
    [34, 52],
    [23, 12],
    [21, 41],
    [50, 39],
    [48, 14],
];

/// Facelet indices of each wing slot on the 4x4x4: the reference sticker, then the other.
const WING_FACELETS: [[usize; 2]; 24] = [
    [1, 82],
    [7, 18],
    [8, 66],
    [14, 34],
    [17, 11],
    [23, 84],
    [24, 43],
    [30, 59],
    [33, 13],
    [39, 20],
    [40, 75],
    [46, 50],
    [49, 45],
    [55, 29],
    [56, 77],
    [62, 93],
    [65, 4],
    [71, 36],
    [72, 91],
    [78, 52],
    [81, 2],
    [87, 68],
    [88, 27],
    [94, 61],
];

/// The slot tables of an `n x n x n` cube, `2 <= n <= 7`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Layout {
    pub(super) n: usize,
    pub(super) corners: Vec<Vec<usize>>,
    /// Empty on even cubes.
    pub(super) midges: Vec<Vec<usize>>,
    /// One table of 24 slots per wing orbit, depth 1 first.
    pub(super) wings: Vec<Vec<Vec<usize>>>,
    /// One table of 24 facelets per centre orbit.
    pub(super) centres: Vec<Vec<usize>>,
    /// The six facelets of the fixed centres on an odd cube, else empty.
    pub(super) fixed: Vec<usize>,
}

impl Layout {
    pub(super) fn new(n: usize) -> Layout {
        let corner_axis = |c: usize| [0, 0, n - 1][c];
        let midge_axis = |c: usize| [0, n / 2, n - 1][c];
        let corners = from_grid(n, 3, &CORNER_FACELETS, &corner_axis);
        let midges = if n % 2 == 1 {
            from_grid(n, 3, &MIDGE_FACELETS, &midge_axis)
        } else {
            Vec::new()
        };
        let wings = (1..=(n - 2) / 2)
            .map(|depth| {
                let wing_axis = |c: usize| [0, depth, n - 1 - depth, n - 1][c];
                from_grid(n, 4, &WING_FACELETS, &wing_axis)
            })
            .collect();
        let fixed = if n % 2 == 1 && n >= 3 {
            (0..6).map(|f| f * n * n + (n * n - 1) / 2).collect()
        } else {
            Vec::new()
        };
        Layout {
            n,
            corners,
            midges,
            wings,
            centres: centre_orbits(n),
            fixed,
        }
    }

    pub(super) fn face_len(&self) -> usize {
        self.n * self.n
    }

    /// The face a facelet shows on the solved cube.
    pub(super) fn solved_face(&self, facelet: usize) -> usize {
        facelet / self.face_len()
    }

    /// The colours of the piece that belongs in `slot` of `table`, in slot order.
    pub(super) fn piece_colours(&self, table: &[Vec<usize>], piece: usize) -> Vec<usize> {
        table[piece]
            .iter()
            .map(|&facelet| self.solved_face(facelet))
            .collect()
    }

    pub(super) fn has_fixed_centres(&self) -> bool {
        !self.fixed.is_empty()
    }
}

/// Moves reference facelets (`b x b` faces) onto an `n x n` face, axis by axis.
fn from_grid<const K: usize>(
    n: usize,
    b: usize,
    reference: &[[usize; K]],
    axis: &dyn Fn(usize) -> usize,
) -> Vec<Vec<usize>> {
    reference
        .iter()
        .map(|slot| {
            slot.iter()
                .map(|&f| {
                    let (face, row, col) = (f / (b * b), f / b % b, f % b);
                    face * n * n + axis(row) * n + axis(col)
                })
                .collect()
        })
        .collect()
}

/// The centre orbits: interior cells that turn into each other under a quarter
/// turn of their face, ordered by lowest facelet index. The middle cell of an
/// odd face is fixed and belongs to none.
fn centre_orbits(n: usize) -> Vec<Vec<usize>> {
    let last = n - 1;
    let mut orbits = Vec::new();
    for row in 1..last {
        for col in 1..last {
            let rank = row * n + col;
            let partners = [
                col * n + last - row,
                (last - row) * n + last - col,
                (last - col) * n + row,
            ];
            let representative = partners.iter().all(|&p| rank <= p);
            if !representative || (n % 2 == 1 && row == n / 2 && col == n / 2) {
                continue;
            }
            let cells = [rank, partners[0], partners[1], partners[2]];
            orbits.push(
                (0..6 * n * n)
                    .filter(|i| cells.contains(&(i % (n * n))))
                    .collect(),
            );
        }
    }
    orbits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn written(layout: &Layout) -> Vec<usize> {
        let mut all: Vec<usize> = layout.corners.iter().flatten().copied().collect();
        all.extend(layout.midges.iter().flatten());
        all.extend(layout.wings.iter().flatten().flatten());
        all.extend(layout.centres.iter().flatten());
        all.extend(&layout.fixed);
        all.sort_unstable();
        all
    }

    #[test]
    fn every_facelet_belongs_to_exactly_one_slot() {
        for n in 2..=7 {
            let layout = Layout::new(n);
            let expected: Vec<usize> = (0..6 * n * n).collect();
            assert_eq!(written(&layout), expected, "{n}x{n}x{n}");
        }
    }

    #[test]
    fn orbit_counts_follow_the_size() {
        for n in 2..=7usize {
            let layout = Layout::new(n);
            assert_eq!(layout.wings.len(), (n - 2) / 2, "{n}");
            assert_eq!(layout.centres.len(), (n - 2) * (n - 2) / 4, "{n}");
            assert_eq!(layout.midges.len(), if n % 2 == 1 { 12 } else { 0 }, "{n}");
        }
    }

    #[test]
    fn the_corner_constants_are_the_standard_slots_and_colours() {
        let layout = Layout::new(3);
        for piece in 0..8 {
            assert_eq!(layout.corners[piece], CORNER_FACELETS[piece]);
            assert_eq!(
                layout.piece_colours(&layout.corners, piece),
                CORNER_COLOURS[piece]
            );
        }
    }

    #[test]
    fn the_five_by_five_wings_and_centres_match_the_reference_tables() {
        let layout = Layout::new(5);
        assert_eq!(layout.wings[0][0], [1, 128]);
        assert_eq!(layout.wings[0][23], [148, 96]);
        assert_eq!(layout.centres[0][..4], [6, 8, 16, 18]);
        assert_eq!(layout.centres[1][..4], [7, 11, 13, 17]);
        assert_eq!(layout.fixed, [12, 37, 62, 87, 112, 137]);
        assert_eq!(layout.midges[0], [14, 27]);
    }
}
