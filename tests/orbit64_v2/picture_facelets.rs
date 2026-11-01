//! Orbit64 V2 picture facelets through the public API: the independent picture
//! vectors of the format in both directions and whole-cube rotations written
//! independently of the library. The strings that are no cube are in
//! `picture_errors.rs`.
//! Vectors: flix-orbit64 (Apache-2.0, Werner Stein).

use cubacle::notation::orbit64::v2::facelets::{decode, encode, from_facelets, to_facelets};
use cubacle::notation::orbit64::v2::{entries, entry, Cube, Model, Pose};
use std::collections::HashSet;

const ROWS: &str = include_str!("../fixtures/orbit64_v2_picture_facelets.txt");
const SLOTS: &str = include_str!("../fixtures/orbit64_v2_slots.txt");

/// The solved 3x3x3 as written in `FORMAT-V2.md`, "Picture facelets".
pub(super) const SOLVED3: &str = "GGJEAADCA BBLREXOJW EDCSIQRLN PKMMMISOV HFFVQTUNQ KHIWUUXPT";

pub(super) struct Row {
    pub(super) id: usize,
    pub(super) moves: &'static str,
    pub(super) facelets: &'static str,
    pub(super) token: &'static str,
    /// The facelets already canonical, or the canonical reading of them.
    pub(super) canonical: &'static str,
}

pub(super) fn rows() -> Vec<Row> {
    ROWS.lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| {
            let f: Vec<&str> = line.split('|').collect();
            assert_eq!(f.len(), 5, "{line}");
            Row {
                id: f[0].parse().unwrap(),
                moves: f[1],
                facelets: f[2],
                token: f[3],
                canonical: if f[4].is_empty() { f[2] } else { f[4] },
            }
        })
        .collect()
}

pub(super) fn state_of(id: usize, facelets: &str) -> Cube {
    let e = entry(id).unwrap();
    from_facelets(e.n, e.model, e.pose, facelets).unwrap()
}

/// The facelets of a slot line of `FORMAT-V2-SLOTS.md`, e.g. `slot(3, "C 0")`.
pub(super) fn slot(n: usize, label: &str) -> Vec<usize> {
    SLOTS
        .lines()
        .filter(|line| !line.starts_with('#'))
        .find_map(|line| {
            let f: Vec<&str> = line.split('|').collect();
            (f[0] == n.to_string() && f[1] == label)
                .then(|| f[2].split(' ').map(|i| i.parse().unwrap()).collect())
        })
        .unwrap_or_else(|| panic!("no slot {label} for {n}"))
}

/// The solved picture of a size, from the vectors: the row of its canonical
/// entry that follows no move.
pub(super) fn solved(n: usize) -> &'static str {
    rows()
        .into_iter()
        .find(|r| r.id == 4 * (n - 2) && r.moves.is_empty())
        .unwrap()
        .facelets
}

#[test]
fn the_fixture_has_every_row_of_the_reference_file() {
    let rows = rows();
    assert_eq!(rows.len(), 174);
    let pictures: Vec<usize> = entries()
        .iter()
        .filter(|e| e.model == Model::Picture)
        .map(|e| e.id)
        .collect();
    assert_eq!(pictures.len(), 26);
    for id in pictures {
        let seen = rows.iter().filter(|r| r.id == id).count();
        assert!(seen >= 3, "entry {id}: {seen} rows");
        assert!(rows.iter().any(|r| r.id == id && r.moves.is_empty()));
    }
    assert!(rows
        .iter()
        .all(|r| entry(r.id).unwrap().model == Model::Picture));
}

#[test]
fn every_row_encodes_to_its_token() {
    for row in rows() {
        for text in [row.facelets, row.canonical] {
            assert_eq!(
                encode(row.id, text).as_deref(),
                Ok(row.token),
                "entry {} after {:?}",
                row.id,
                row.moves
            );
        }
    }
}

#[test]
fn every_token_decodes_to_its_facelets() {
    for row in rows() {
        let (found, text) = decode(row.token).unwrap();
        assert_eq!(found.id, row.id, "{:?}", row.moves);
        assert_eq!(text, row.canonical, "entry {} {:?}", row.id, row.moves);
    }
}

#[test]
fn cube_and_facelets_agree_in_both_directions_for_every_row() {
    for row in rows() {
        let e = entry(row.id).unwrap();
        let cube = state_of(row.id, row.facelets);
        assert_eq!(
            cubacle::notation::orbit64::v2::decode(row.token).unwrap().1,
            cube
        );
        // Written back, a framed cube keeps the orientation it was read in;
        // a canonical one writes its canonical picture.
        let written = to_facelets(e.n, e.model, e.pose, &cube).unwrap();
        let expected = if e.pose == Pose::Framed {
            row.facelets
        } else {
            row.canonical
        };
        assert_eq!(written, expected, "entry {} {:?}", row.id, row.moves);
        assert_eq!(state_of(row.id, &written), cube);
    }
}

#[test]
fn a_canonical_read_of_a_framed_row_is_the_canonical_row_of_the_same_moves() {
    // Every framed row has a canonical row of the same size and moves (the
    // entry before it); the two show the same cube in another orientation.
    let rows = rows();
    let mut checked = 0;
    for row in rows
        .iter()
        .filter(|r| entry(r.id).unwrap().pose == Pose::Framed)
    {
        let e = entry(row.id).unwrap();
        let twin = rows
            .iter()
            .find(|r| r.id + 1 == row.id && r.moves == row.moves)
            .unwrap();
        let cube = from_facelets(e.n, e.model, Pose::Canonical, row.facelets).unwrap();
        assert_eq!(cube, state_of(twin.id, twin.facelets), "{:?}", row.moves);
        assert_eq!(
            to_facelets(e.n, e.model, Pose::Canonical, &cube).as_deref(),
            Ok(twin.canonical)
        );
        assert_eq!(
            cubacle::notation::orbit64::v2::encode(twin.id, &cube).as_deref(),
            Ok(twin.token)
        );
        checked += 1;
    }
    assert_eq!(checked, 87);
}

#[test]
fn the_solved_pictures_are_the_zero_tokens_of_every_picture_entry() {
    assert_eq!(solved(3), SOLVED3);
    for e in entries().iter().filter(|e| e.model == Model::Picture) {
        let cube = Cube::solved(e.n, Model::Picture).unwrap();
        let token = cubacle::notation::orbit64::v2::encode(e.id, &cube).unwrap();
        let text = solved(e.n);
        assert_eq!(encode(e.id, text), Ok(token.clone()), "entry {}", e.id);
        assert_eq!(
            to_facelets(e.n, e.model, e.pose, &cube).as_deref(),
            Ok(text),
            "entry {}",
            e.id
        );
        assert_eq!(decode(&token).unwrap().1, text, "entry {}", e.id);
    }
}

#[test]
fn spaces_are_ignored_on_input_and_one_space_separates_fields_on_output() {
    let packed = SOLVED3.replace(' ', "");
    let spaced = format!(" {}  ", SOLVED3.replace("", " "));
    let cube = from_facelets(3, Model::Picture, Pose::Framed, SOLVED3).unwrap();
    for text in [&packed, &spaced] {
        assert_eq!(
            from_facelets(3, Model::Picture, Pose::Framed, text),
            Ok(cube.clone())
        );
    }
    assert_eq!(
        to_facelets(3, Model::Picture, Pose::Framed, &cube).as_deref(),
        Ok(SOLVED3)
    );
}

// ----------------------------------------------- whole-cube rotations

type Vec3 = [i32; 3];

/// Position (doubled grid) and outward normal of facelet `i`, from the
/// geometry of FORMAT.md, written here independently of the library.
fn sticker(n: usize, i: usize) -> (Vec3, Vec3) {
    let m = n as i32 - 1;
    let (face, row, col) = (i / (n * n), (i / n % n) as i32, (i % n) as i32);
    let (a, b) = (2 * col - m, 2 * row - m);
    match face {
        0 => ([a, m, b], [0, 1, 0]),
        1 => ([m, -b, -a], [1, 0, 0]),
        2 => ([a, -b, m], [0, 0, 1]),
        3 => ([a, -m, -b], [0, -1, 0]),
        4 => ([-m, -b, a], [-1, 0, 0]),
        _ => ([-a, -b, -m], [0, 0, -1]),
    }
}

fn apply(rotation: &[Vec3; 3], v: Vec3) -> Vec3 {
    std::array::from_fn(|k| rotation[k][0] * v[0] + rotation[k][1] * v[1] + rotation[k][2] * v[2])
}

/// All 24 proper rotations, as the closure of two quarter turns.
fn rotations() -> Vec<[Vec3; 3]> {
    let x = [[1, 0, 0], [0, 0, -1], [0, 1, 0]];
    let y = [[0, 0, 1], [0, 1, 0], [-1, 0, 0]];
    let mut found = vec![[[1, 0, 0], [0, 1, 0], [0, 0, 1]]];
    let mut next = 0;
    while next < found.len() {
        let current = found[next];
        next += 1;
        for turn in [x, y] {
            let product: [Vec3; 3] = std::array::from_fn(|k| {
                std::array::from_fn(|c| (0..3).map(|j| turn[k][j] * current[j][c]).sum())
            });
            if !found.contains(&product) {
                found.push(product);
            }
        }
    }
    found
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// The direction of row 0 on a face, read off the geometry: from the center
/// facelet to the middle facelet of the face's first row.
fn row0(n: usize, face: usize) -> Vec3 {
    let (center, _) = sticker(n, face * n * n + (n * n - 1) / 2);
    let (top, _) = sticker(n, face * n * n + (n - 1) / 2);
    std::array::from_fn(|k| (top[k] - center[k]).signum())
}

/// A direction on a face turned `k` clockwise quarter turns, seen from outside.
fn spin(normal: Vec3, direction: Vec3, k: usize) -> Vec3 {
    (0..k).fold(direction, |d, _| cross(d, normal))
}

fn symbol_value(c: char) -> usize {
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"
        .find(c)
        .unwrap()
}

pub(super) fn symbol_char(v: usize) -> char {
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"
        .chars()
        .nth(v)
        .unwrap()
}

/// The picture with the whole cube turned: every sticker moves to the facelet
/// its position and normal are carried to, and a fixed center, whose symbol is
/// `4 p + r`, has `r` measured again against the reference of its new face.
fn turned(n: usize, text: &str, rotation: &[Vec3; 3]) -> String {
    let symbols: Vec<char> = text.chars().filter(|&c| c != ' ').collect();
    let index: std::collections::HashMap<(Vec3, Vec3), usize> =
        (0..symbols.len()).map(|i| (sticker(n, i), i)).collect();
    let mut out = vec![' '; symbols.len()];
    for (i, &symbol) in symbols.iter().enumerate() {
        let (p, q) = sticker(n, i);
        let target = index[&(apply(rotation, p), apply(rotation, q))];
        out[target] = symbol;
        if n % 2 == 1 && i % (n * n) == (n * n - 1) / 2 {
            let (face, value) = (i / (n * n), symbol_value(symbol));
            let landed = target / (n * n);
            let own = apply(rotation, spin(q, row0(n, face), value % 4));
            let (_, normal) = sticker(n, target);
            let r = (0..4)
                .find(|&k| spin(normal, row0(n, landed), k) == own)
                .unwrap();
            out[target] = symbol_char(value / 4 * 4 + r);
        }
    }
    out.chunks(n * n)
        .map(|f| f.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn the_independent_rotation_reproduces_the_framed_rows() {
    // A framed row of an odd cube is its canonical picture rotated by the
    // frame, so one of the 24 rotations of the canonical text is the row.
    let turns = rotations();
    assert_eq!(turns.len(), 24);
    let mut checked = 0;
    for row in rows()
        .iter()
        .filter(|r| entry(r.id).unwrap().pose == Pose::Framed && entry(r.id).unwrap().n % 2 == 1)
    {
        let n = entry(row.id).unwrap().n;
        let twin = rows()
            .into_iter()
            .find(|r| r.id + 1 == row.id && r.moves == row.moves)
            .unwrap();
        let all: HashSet<String> = turns.iter().map(|t| turned(n, twin.canonical, t)).collect();
        assert!(
            all.contains(row.facelets),
            "entry {} after {:?}",
            row.id,
            row.moves
        );
        checked += 1;
    }
    assert_eq!(checked, 67);
}

#[test]
fn a_canonical_read_accepts_every_whole_cube_orientation() {
    let turns = rotations();
    for row in rows()
        .iter()
        .filter(|r| entry(r.id).unwrap().pose == Pose::Canonical)
    {
        let e = entry(row.id).unwrap();
        let expected = state_of(row.id, row.canonical);
        let mut texts = HashSet::new();
        for turn in &turns {
            let text = turned(e.n, row.canonical, turn);
            texts.insert(text.clone());
            assert_eq!(
                from_facelets(e.n, e.model, Pose::Canonical, &text),
                Ok(expected.clone()),
                "entry {} after {:?}",
                row.id,
                row.moves
            );
            assert_eq!(encode(row.id, &text).as_deref(), Ok(row.token));
        }
        // A scrambled cube looks different in every orientation.
        if !row.moves.is_empty() {
            assert_eq!(texts.len(), 24, "entry {} after {:?}", row.id, row.moves);
        }
    }
}

#[test]
fn a_framed_read_of_an_odd_cube_keeps_its_orientation() {
    let turns = rotations();
    for row in rows()
        .iter()
        .filter(|r| entry(r.id).unwrap().pose == Pose::Canonical && entry(r.id).unwrap().n % 2 == 1)
    {
        let n = entry(row.id).unwrap().n;
        let canonical = state_of(row.id, row.canonical);
        let mut frames = HashSet::new();
        for turn in &turns {
            let text = turned(n, row.canonical, turn);
            let cube = from_facelets(n, Model::Picture, Pose::Framed, &text).unwrap();
            assert_eq!(
                to_facelets(n, Model::Picture, Pose::Framed, &cube).as_deref(),
                Ok(&text[..])
            );
            frames.insert(cube.frame);
            assert_eq!(Cube { frame: 0, ..cube }, canonical, "{:?}", row.moves);
        }
        assert_eq!(frames.len(), 24, "entry {}", row.id);
    }
}

#[test]
fn a_fixed_center_rotation_is_the_symbol_itself() {
    // U and R centers each turned a quarter: symbols B (4 * 0 + 1) and F (4 * 1 + 1).
    let text = put(SOLVED3, &[(4, 'B'), (13, 'F')]);
    let cube = from_facelets(3, Model::Picture, Pose::Canonical, &text).unwrap();
    assert_eq!(cube.rotations, vec![1, 1, 0, 0, 0, 0]);
    assert_eq!(
        to_facelets(3, Model::Picture, Pose::Canonical, &cube),
        Ok(text)
    );
}

#[test]
fn a_rotated_cube_keeps_its_stickers_and_spins_the_fixed_centers() {
    // After `x` the cube shows F R D B L U in its slots; the stickers did not
    // change, only the fixed centers have their rotation measured anew.
    let rotated = "EDCSIQRLN ORBJFBWXL PKMMMISOV TPXUWWIHK FTQFTNHVU ACDACEJGG";
    assert_eq!(encode(5, rotated).as_deref(), Ok("AAAAAAAAAAAAAE"));
    assert_eq!(encode(4, rotated).as_deref(), Ok("AAAAAAAAAAAAA"));
    // The turn `x` of FORMAT.md carries the front to the top.
    let x = [[1, 0, 0], [0, 0, 1], [0, -1, 0]];
    assert_eq!(turned(3, SOLVED3, &x), rotated);
}

/// The string with the symbol at each given facelet index replaced.
pub(super) fn put(text: &str, edits: &[(usize, char)]) -> String {
    let mut symbols: Vec<char> = text.chars().filter(|&c| c != ' ').collect();
    let n2 = symbols.len() / 6;
    for &(at, c) in edits {
        symbols[at] = c;
    }
    symbols
        .chunks(n2)
        .map(|f| f.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}
