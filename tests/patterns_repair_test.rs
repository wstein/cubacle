//! Pattern repair: a built-in pattern plus a few planted moves is found again,
//! in either order, in a turned frame and with other colors.

use std::time::{Duration, Instant};

use cubacle::patterns::{self, Pattern, RepairLimits, RepairOrder};
use cubacle::puzzle::{Face, Facelets, Move, MoveSequence, PuzzleKind};

fn library() -> Vec<&'static Pattern> {
    patterns::all().iter().collect()
}

fn generator(name: &str, kind: PuzzleKind) -> MoveSequence {
    patterns::find(name)
        .unwrap()
        .generator(kind.size(), None)
        .unwrap()
        .sequence()
        .unwrap()
}

/// The whole-cube turn about the vertical axis, applied to a move sequence.
fn turned(moves: &MoveSequence) -> MoveSequence {
    moves
        .moves()
        .iter()
        .map(|m| {
            let face = match m.face {
                Face::R => Face::B,
                Face::B => Face::L,
                Face::L => Face::F,
                Face::F => Face::R,
                other => other,
            };
            Move::new(face, m.depth, m.wide, m.turn)
        })
        .collect()
}

/// Rename the colors: every face letter becomes the next one.
fn recolored(cube: &Facelets) -> Facelets {
    let next = |f: Face| {
        let at = Face::ALL.iter().position(|&x| x == f).unwrap();
        Face::ALL[(at + 1) % 6]
    };
    Facelets::from_stickers(cube.stickers().iter().map(|&f| next(f)).collect()).unwrap()
}

struct Plant<'a> {
    pattern: &'a str,
    kind: PuzzleKind,
    extra: &'a str,
    before: bool,
    turn: bool,
    recolor: bool,
}

fn plant(p: &Plant<'_>) -> Facelets {
    let kind = p.kind;
    let mut g = generator(p.pattern, kind);
    if p.turn {
        g = turned(&g);
    }
    let extra: MoveSequence = p.extra.parse().unwrap();
    let solved = Facelets::solved(kind);
    let cube = if p.before {
        solved.apply(&extra).unwrap().apply(&g).unwrap()
    } else {
        solved.apply(&g).unwrap().apply(&extra).unwrap()
    };
    if p.recolor {
        recolored(&cube)
    } else {
        cube
    }
}

/// Finds the plant again with a verified fix no longer than the planted one.
fn recover(p: &Plant<'_>, blocks: usize, seconds: u64) -> Duration {
    let cube = plant(p);
    let limits = RepairLimits {
        max_blocks: blocks,
        max_time: Duration::from_secs(seconds),
        ..RepairLimits::default()
    };
    let started = Instant::now();
    let out = cubacle::patterns::repair(&cube, &library(), &limits);
    let elapsed = started.elapsed();
    let fix = out
        .found
        .unwrap_or_else(|| panic!("no fix for {} + {}", p.pattern, p.extra));
    assert!(fix.blocks <= blocks, "{} {}", fix.id, fix.moves);
    assert!(!out.stats.timed_out);
    assert!(matches!(
        fix.order,
        RepairOrder::After | RepairOrder::Before
    ));
    elapsed
}

fn case<'a>(pattern: &'a str, kind: PuzzleKind, extra: &'a str, before: bool) -> Plant<'a> {
    Plant {
        pattern,
        kind,
        extra,
        before,
        turn: false,
        recolor: false,
    }
}

#[test]
fn a_cube3_pattern_with_one_to_four_extra_moves() {
    for (extra, blocks) in [("R", 1), ("R U", 2), ("R U' F2", 3), ("R U' F2 L", 4)] {
        for before in [false, true] {
            let t = recover(
                &case("checkerboard", PuzzleKind::Cube3, extra, before),
                blocks,
                30,
            );
            eprintln!("cube3 checkerboard + {extra} before={before}: {t:?}");
        }
    }
}

#[test]
fn a_turned_and_recolored_cube3_pattern() {
    for before in [false, true] {
        let p = Plant {
            turn: true,
            recolor: true,
            ..case("six-spots", PuzzleKind::Cube3, "U R2 D'", before)
        };
        let t = recover(&p, 3, 30);
        eprintln!("cube3 six-spots turned+recolored before={before}: {t:?}");
    }
}

#[test]
fn a_cube4_pattern_with_wide_and_inner_moves() {
    for (extra, blocks) in [("Rw", 1), ("Rw U", 2), ("2R Fw' 2U2", 3)] {
        for before in [false, true] {
            let t = recover(
                &case("checkerboard", PuzzleKind::Cube4, extra, before),
                blocks,
                60,
            );
            eprintln!("cube4 checkerboard + {extra} before={before}: {t:?}");
        }
    }
}

#[test]
fn a_cube7_pattern_with_wide_blocks() {
    for (extra, blocks) in [("3Rw", 1), ("3Bw B' 3Rw", 2), ("3Rw U 3Fw'", 3)] {
        for before in [false, true] {
            let t = recover(
                &case("checkerboard", PuzzleKind::Cube7, extra, before),
                blocks,
                120,
            );
            eprintln!("cube7 checkerboard + {extra} before={before}: {t:?}");
        }
    }
}

#[test]
fn a_random_state_has_no_fix() {
    let cube = cubacle::puzzle::Facelets::solved(PuzzleKind::Cube3)
        .apply(&"R U F' L2 D B R' U2 F D' L B2 R F'".parse().unwrap())
        .unwrap();
    let limits = RepairLimits {
        max_blocks: 4,
        ..RepairLimits::default()
    };
    let out = cubacle::patterns::repair(&cube, &library(), &limits);
    assert!(out.found.is_none(), "{:?}", out.found);
    assert!(!out.stats.timed_out);
}

#[test]
fn the_time_limit_is_respected() {
    let cube = plant(&case(
        "checkerboard",
        PuzzleKind::Cube7,
        "3Rw U 3Fw' D",
        false,
    ));
    let limits = RepairLimits {
        max_blocks: 7,
        max_time: Duration::from_millis(300),
        ..RepairLimits::default()
    };
    let started = Instant::now();
    let out = cubacle::patterns::repair(&cube, &library(), &limits);
    assert!(started.elapsed() < Duration::from_secs(10));
    assert!(out.found.is_some() || out.stats.timed_out);
}
