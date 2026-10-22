//! Origin: Rust re-implementation of `Orbit64.Net.toFacelets` and
//! `Orbit64.Net.fromFacelets` (flix-orbit64, Apache-2.0, Werner Stein).
//!
//! Writes the pieces of each slot as stickers and reads them back. Stickers
//! are face indices into `URFDLB`, facelet `face * n * n + row * n + col`, in
//! the canonical pose; the frame is applied or removed by the caller.

use super::coords::Coords;
use super::layout::Layout;

/// The stickers of a cube whose pieces are as `coords` say, canonical pose.
pub(super) fn place(layout: &Layout, coords: &Coords) -> Vec<usize> {
    let mut stickers = vec![0; 6 * layout.face_len()];
    for (face, &at) in layout.fixed.iter().enumerate() {
        stickers[at] = face;
    }
    place_oriented(
        layout,
        &mut stickers,
        &layout.corners,
        &coords.corner_perm,
        &coords.corner_twist,
    );
    place_oriented(
        layout,
        &mut stickers,
        &layout.midges,
        &coords.midge_perm,
        &coords.midge_flip,
    );
    for (table, wings) in layout.wings.iter().zip(&coords.wings) {
        for (slot, &piece) in wings.iter().enumerate() {
            let colours = layout.piece_colours(table, piece);
            for (&at, colour) in table[slot].iter().zip(colours) {
                stickers[at] = colour;
            }
        }
    }
    for (table, colours) in layout.centres.iter().zip(&coords.centres) {
        for (&at, &colour) in table.iter().zip(colours) {
            stickers[at] = colour;
        }
    }
    stickers
}

fn place_oriented(
    layout: &Layout,
    stickers: &mut [usize],
    table: &[Vec<usize>],
    perm: &[usize],
    turn: &[usize],
) {
    for (slot, facelets) in table.iter().enumerate() {
        let colours = layout.piece_colours(table, perm[slot]);
        let m = facelets.len();
        for (k, &at) in facelets.iter().enumerate() {
            stickers[at] = colours[(k + m - turn[slot]) % m];
        }
    }
}

/// Which piece and turn show `seen` in a slot, as `place_oriented` writes them.
fn identify(layout: &Layout, table: &[Vec<usize>], seen: &[usize]) -> Option<(usize, usize)> {
    let m = seen.len();
    (0..table.len()).find_map(|piece| {
        let colours = layout.piece_colours(table, piece);
        (0..m)
            .find(|&turn| (0..m).all(|k| seen[k] == colours[(k + m - turn) % m]))
            .map(|turn| (piece, turn))
    })
}

fn read_oriented(
    layout: &Layout,
    stickers: &[usize],
    table: &[Vec<usize>],
    what: &str,
) -> Result<(Vec<usize>, Vec<usize>), String> {
    let mut perm = Vec::with_capacity(table.len());
    let mut turn = Vec::with_capacity(table.len());
    for (slot, facelets) in table.iter().enumerate() {
        let seen: Vec<usize> = facelets.iter().map(|&at| stickers[at]).collect();
        let (piece, twist) = identify(layout, table, &seen).ok_or_else(|| {
            format!("the {what} in slot {slot} shows colours no {what} of a cube has")
        })?;
        perm.push(piece);
        turn.push(twist);
    }
    Ok((perm, turn))
}

/// The pieces of a cube shown by `stickers`, which are in the canonical pose.
/// `Err` says in plain words which slot shows what no piece can show.
pub(super) fn read(layout: &Layout, stickers: &[usize]) -> Result<Coords, String> {
    let (corner_perm, corner_twist) = read_oriented(layout, stickers, &layout.corners, "corner")?;
    let (midge_perm, midge_flip) = if layout.midges.is_empty() {
        (Vec::new(), Vec::new())
    } else {
        read_oriented(layout, stickers, &layout.midges, "edge")?
    };
    let mut wings = Vec::new();
    for table in &layout.wings {
        let mut perm = Vec::with_capacity(24);
        for (slot, facelets) in table.iter().enumerate() {
            let seen: Vec<usize> = facelets.iter().map(|&at| stickers[at]).collect();
            let piece = (0..table.len())
                .find(|&piece| layout.piece_colours(table, piece) == seen)
                .ok_or_else(|| {
                    format!("the wing in slot {slot} shows colours no wing of a cube has")
                })?;
            perm.push(piece);
        }
        wings.push(perm);
    }
    let centres = layout
        .centres
        .iter()
        .map(|table| table.iter().map(|&at| stickers[at]).collect())
        .collect();
    Ok(Coords {
        corner_perm,
        corner_twist,
        midge_perm,
        midge_flip,
        wings,
        centres,
    })
}

#[cfg(test)]
mod tests {
    use super::super::coords::radices;
    use super::*;

    #[test]
    fn a_solved_cube_is_its_own_stickers() {
        for n in 2..=7 {
            let layout = Layout::new(n);
            let solved = Coords::from_digits(n, &vec![0; radices(n).len()]);
            let expected: Vec<usize> = (0..6 * n * n).map(|i| i / (n * n)).collect();
            assert_eq!(place(&layout, &solved), expected, "{n}");
            assert_eq!(read(&layout, &expected).unwrap(), solved, "{n}");
        }
    }

    #[test]
    fn a_swapped_pair_of_corner_stickers_is_no_piece() {
        let layout = Layout::new(3);
        let mut stickers: Vec<usize> = (0..54).map(|i| i / 9).collect();
        stickers.swap(8, 29);
        assert!(read(&layout, &stickers).unwrap_err().contains("corner"));
    }
}
