//! Membership of well-known subgroups, read from the pieces of the state.
//!
//! Each test is a coordinate check on the pieces (no search). A test is
//! exact where the subgroup has a simple description by pieces, and says so;
//! for groups that have none (`<R,U>`, `<R,U,F>`) it only refutes: a piece
//! that the generators never touch is out of place, or the answer is
//! "possible", which is not a proof of membership (use `solve --moves` for
//! that).

use super::pieces::{frames, Cube, Frame};
use crate::puzzle::PuzzleKind;

/// What the test found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The state is in the subgroup (exact).
    Member,
    /// The state is not in the subgroup (exact).
    NotMember,
    /// No necessary condition is violated; membership is not decided.
    Possible,
}

/// One subgroup test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Membership {
    /// The subgroup, such as `<U,D,R2,L2,F2,B2>`.
    pub name: String,
    pub verdict: Verdict,
    /// The reason in a few words.
    pub why: String,
}

/// The answer of a test in one frame: yes, or the reason for no.
type Answer = Result<(), String>;

fn at_home(cube: &Cube, stickers: impl Iterator<Item = usize>) -> bool {
    let face_len = cube.layout.face_len;
    let mut all = true;
    for s in stickers {
        all &= cube.lab[s] as usize == s / face_len;
    }
    all
}

fn corner_stickers(cube: &Cube) -> impl Iterator<Item = usize> + '_ {
    cube.layout.corners.iter().flatten().copied()
}

fn wing_stickers(cube: &Cube) -> impl Iterator<Item = usize> + '_ {
    cube.layout.wings.iter().flat_map(|w| w.stickers)
}

fn centre_stickers(cube: &Cube) -> impl Iterator<Item = usize> + '_ {
    cube.layout.centres.iter().flatten().copied()
}

fn only(cube: &Cube, free: &str) -> Answer {
    let corners = at_home(cube, corner_stickers(cube));
    let wings = at_home(cube, wing_stickers(cube));
    let centres = at_home(cube, centre_stickers(cube));
    let (ok, bad) = match free {
        "corners" => (wings && centres, "an edge or centre piece is moved"),
        "edges" => (corners && centres, "a corner or centre piece is moved"),
        _ => (corners && wings, "a corner or edge piece is moved"),
    };
    if ok {
        Ok(())
    } else {
        Err(bad.to_owned())
    }
}

/// Pieces the faces `turned` never touch must be at home and oriented.
fn untouched_home(cube: &Cube, turned: &[usize]) -> Answer {
    let mask = turned.iter().fold(0u8, |m, &f| m | 1 << f);
    let order = [0, 2, 1];
    let layout = cube.layout;
    let places = layout
        .corners
        .iter()
        .map(|c| (c.to_vec(), "corner"))
        .chain(layout.wings.iter().map(|w| (w.stickers.to_vec(), "edge")));
    for (place, what) in places {
        if cube.faces_of(&place) & mask != 0 {
            continue;
        }
        let reading = cube.read(&place, order);
        if !(reading.placed && reading.oriented) {
            return Err(format!("a {what} these faces never touch is moved"));
        }
    }
    if at_home(cube, centre_stickers(cube)) {
        Ok(())
    } else {
        Err("a centre is moved".to_owned())
    }
}

/// Whether the pieces of a 3x3x3 are those of the group that keeps U/D (or
/// the axis chosen by `order`) as the domino axis: corners untwisted, edges
/// unflipped, the four middle-layer edges in the middle layer.
fn domino(cube: &Cube, order: [usize; 3]) -> Answer {
    let layout = cube.layout;
    for corner in &layout.corners {
        let r = cube.read(corner, order);
        if !r.valid {
            return Err("the corners are no cube's".to_owned());
        }
        if !r.oriented {
            return Err("a corner is twisted".to_owned());
        }
    }
    for wing in &layout.wings {
        let r = cube.read(&wing.stickers, order);
        if !r.valid {
            return Err("the edges are no cube's".to_owned());
        }
        if !r.oriented {
            return Err("an edge is flipped".to_owned());
        }
        let has = |bits: u8| bits & (1 << order[0] | 1 << (order[0] + 3)) != 0;
        if has(cube.faces_of(&wing.stickers)) != has(r.home) {
            return Err("a middle-layer edge is out of the middle layer".to_owned());
        }
    }
    if at_home(cube, centre_stickers(cube)) {
        Ok(())
    } else {
        Err("a centre is moved".to_owned())
    }
}

fn verdict(answers: Vec<Answer>, exact: bool) -> (Verdict, String) {
    if answers.iter().any(Result::is_ok) {
        if exact {
            (Verdict::Member, "yes".to_owned())
        } else {
            (Verdict::Possible, "no piece outside the group's reach".to_owned())
        }
    } else {
        let why = answers
            .into_iter()
            .find_map(Result::err)
            .unwrap_or_else(|| "no".to_owned());
        (Verdict::NotMember, why)
    }
}

/// The subgroup tests that mean something for a size, in `frames`.
pub(crate) fn memberships(kind: PuzzleKind, state: &[u8]) -> Vec<Membership> {
    let n = kind.size();
    let frames: Vec<Frame> = frames(kind, state);
    let cubes: Vec<Cube> = frames.iter().map(|f| Cube::new(kind, state, f)).collect();
    let run = |test: &dyn Fn(&Cube) -> Answer, exact: bool| {
        verdict(cubes.iter().map(test).collect(), exact)
    };
    let mut out = Vec::new();
    let mut add = |name: &str, (verdict, why): (Verdict, String)| {
        out.push(Membership {
            name: name.to_owned(),
            verdict,
            why,
        });
    };
    if n >= 3 {
        add("corners only", run(&|c| only(c, "corners"), true));
        add("edges only", run(&|c| only(c, "edges"), true));
    }
    if n >= 4 {
        add("centres only", run(&|c| only(c, "centres"), true));
    }
    if n == 3 {
        let g1 = |axis: usize| {
            let order = [axis, (axis + 1) % 3, (axis + 2) % 3];
            run(&|c| domino(c, order), true)
        };
        for (axis, name) in [(0, "U/D"), (2, "F/B"), (1, "R/L")] {
            let (v, why) = g1(axis);
            add(&format!("G1 domino group, {name} axis"), (v, why));
        }
        // U=0, R=1, F=2 as face indices.
        add("<R,U>", run(&|c| untouched_home(c, &[0, 1]), false));
        add("<R,U,F>", run(&|c| untouched_home(c, &[0, 1, 2]), false));
    }
    out
}
