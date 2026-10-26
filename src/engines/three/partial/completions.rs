//! The second route: enumerate the complete cubes a target allows and solve each.

use std::time::Instant;

use super::super::cubie::{Cubie, CORNER_FACELET, EDGE_FACELET};
use super::super::solver::Min2PhaseSolver;
use super::{color, finish, MAX_COMPLETIONS, MAX_EDGES, MAX_NODES};
use crate::puzzle::{
    relative_state, Face, Facelets, Failure, MoveSequence, SolveOptions, SolveResult, Solver,
    Target,
};

/// One slot's candidates: the sticker colors it may show, as complete
/// readings of that slot's pieces.
struct Slot {
    facelets: Vec<u8>,
    /// (piece, colors per sticker) for every piece and orientation that fits.
    options: Vec<(usize, Vec<usize>)>,
    corner: bool,
}

fn slots_of(sets: &[u8]) -> Vec<Slot> {
    let mut slots = Vec::new();
    for facelets in &CORNER_FACELET {
        slots.push(Slot {
            facelets: facelets.to_vec(),
            options: Vec::new(),
            corner: true,
        });
    }
    for facelets in &EDGE_FACELET {
        slots.push(Slot {
            facelets: facelets.to_vec(),
            options: Vec::new(),
            corner: false,
        });
    }
    for slot in &mut slots {
        let homes: Vec<Vec<usize>> = if slot.corner {
            CORNER_FACELET
                .iter()
                .map(|f| f.iter().map(|&x| color(x)).collect())
                .collect()
        } else {
            EDGE_FACELET
                .iter()
                .map(|f| f.iter().map(|&x| color(x)).collect())
                .collect()
        };
        let n = slot.facelets.len();
        for (piece, home) in homes.iter().enumerate() {
            for shift in 0..n {
                let shown: Vec<usize> = (0..n).map(|k| home[(k + shift) % n]).collect();
                let fits = shown
                    .iter()
                    .zip(&slot.facelets)
                    .all(|(&c, &f)| sets[usize::from(f)] >> c & 1 == 1);
                if fits {
                    slot.options.push((piece, shown));
                }
            }
        }
    }
    slots
}

/// Every complete cube the target allows, up to [`MAX_COMPLETIONS`]; `None`
/// when there are more or the enumeration took too long.
fn completions(sets: &[u8]) -> Option<Vec<Facelets>> {
    let slots = slots_of(sets);
    let mut found = Vec::new();
    let mut nodes = 0usize;
    let mut chosen: Vec<usize> = Vec::new();
    let mut used = [[false; 12]; 2];
    let ok = walk(&slots, sets, &mut chosen, &mut used, &mut found, &mut nodes);
    ok.then_some(found)
}

fn walk(
    slots: &[Slot],
    sets: &[u8],
    chosen: &mut Vec<usize>,
    used: &mut [[bool; 12]; 2],
    found: &mut Vec<Facelets>,
    nodes: &mut usize,
) -> bool {
    *nodes += 1;
    if *nodes > MAX_NODES {
        return false;
    }
    let at = chosen.len();
    if at == slots.len() {
        let mut stickers: Vec<Face> = (0..54).map(|i| Face::ALL[i / 9]).collect();
        // centers keep their own color; the pieces fill the rest
        for (slot, &pick) in slots.iter().zip(chosen.iter()) {
            for (&facelet, &c) in slot.facelets.iter().zip(&slot.options[pick].1) {
                stickers[usize::from(facelet)] = Face::ALL[c];
            }
        }
        if let Ok(cube) = Facelets::from_stickers(stickers) {
            let mut check = Cubie::new();
            if check.set_from_facelet(&cube.to_string()) == 0
                && check.verify() == 0
                && (0..54).all(|i| sets[i] >> cube.stickers()[i].index() & 1 == 1)
            {
                if found.len() == MAX_COMPLETIONS {
                    return false;
                }
                found.push(cube);
            }
        }
        return true;
    }
    let slot = &slots[at];
    let group = usize::from(!slot.corner);
    for (pick, (piece, _)) in slot.options.iter().enumerate() {
        if used[group][*piece] {
            continue;
        }
        used[group][*piece] = true;
        chosen.push(pick);
        let ok = walk(slots, sets, chosen, used, found, nodes);
        chosen.pop();
        used[group][*piece] = false;
        if !ok {
            return false;
        }
    }
    true
}

pub(super) fn by_completions(
    engine: &Min2PhaseSolver,
    started: Instant,
    start: &Facelets,
    target: &Target,
    options: &SolveOptions,
) -> SolveResult {
    let Some(all) = completions(target.sets()) else {
        return finish(
            started,
            Err(Failure::Exhausted),
            format!(
                "unsupported: the target allows more than {MAX_COMPLETIONS} complete cubes. Supported on the 3x3x3: whole edges (up to {MAX_EDGES}, such as `cross`), or open stickers that leave at most {MAX_COMPLETIONS} possible cubes"
            ),
        );
    };
    if all.is_empty() {
        return finish(
            started,
            Err(Failure::Exhausted),
            "unreachable: no cube meets the target (the pieces cannot be arranged with these colors)".into(),
        );
    }
    let mut best: Option<MoveSequence> = None;
    let mut worst_failure = None;
    for goal in &all {
        let Ok(relative) = relative_state(start, goal) else {
            continue;
        };
        let result = engine.solve(&relative, options);
        match result.outcome {
            Ok(moves) => {
                if best.as_ref().is_none_or(|b| moves.len() < b.len()) {
                    best = Some(moves);
                }
            }
            Err(failure) => worst_failure = Some((failure, result.message)),
        }
    }
    match best {
        Some(moves) => {
            finish(
                started,
                Ok(moves),
                format!(
                    "best found: the shortest of the two-phase answers to {} complete cube(s) the target allows, not proven shortest",
                    all.len()
                ),
            )
        }
        None => {
            let (failure, message) = worst_failure.unwrap_or((Failure::Exhausted, None));
            finish(
                started,
                Err(failure),
                message.unwrap_or_else(|| "no answer within the limits".into()),
            )
        }
    }
}
