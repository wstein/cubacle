//! The mistakes the search tries at each position of the printed solution.

use crate::puzzle::{Move, Turn};

use super::state::{MoveId, Perms};
use super::{Mistake, MistakeKind};

/// One way of going wrong at a position: the moves done in place of the
/// printed ones, and how many printed moves they used up.
pub(super) struct Variant {
    pub(super) mistake: Mistake,
    pub(super) moves: Vec<MoveId>,
    /// Printed moves consumed: 1 for a skip or a replacement, 2 for a swap,
    /// 0 for an extra move.
    pub(super) span: usize,
    /// How unlikely the mistake is: lower is more plausible.
    pub(super) score: u8,
}

/// Which mistakes to try.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Breadth {
    /// Skips, wrong turn amount, direction, layers or face, swaps, repeats.
    Likely,
    /// Also any other move instead, and any extra move.
    Any,
}

fn score(kind: &MistakeKind) -> u8 {
    match kind {
        MistakeKind::Skipped
        | MistakeKind::WrongAmount { .. }
        | MistakeKind::WrongDirection { .. }
        | MistakeKind::Swapped { .. }
        | MistakeKind::Repeated => 0,
        MistakeKind::WrongLayers { .. } | MistakeKind::WrongFace { .. } => 1,
        MistakeKind::Extra { .. } => 2,
        MistakeKind::Different { .. } => 3,
    }
}

/// What `done` is, when `printed` was meant.
fn replacement(printed: &Move, done: &Move) -> MistakeKind {
    let same_layers = printed.depth == done.depth && printed.wide == done.wide;
    if printed.face == done.face && same_layers {
        if printed.turn == Turn::Half || done.turn == Turn::Half {
            MistakeKind::WrongAmount { done: *done }
        } else {
            MistakeKind::WrongDirection { done: *done }
        }
    } else if printed.face == done.face && printed.turn == done.turn {
        MistakeKind::WrongLayers { done: *done }
    } else if same_layers && printed.turn == done.turn {
        MistakeKind::WrongFace { done: *done }
    } else {
        MistakeKind::Different { done: *done }
    }
}

/// The variants at every position `0..=n` of the printed solution.
pub(super) fn all(printed: &[MoveId], perms: &Perms, breadth: Breadth) -> Vec<Vec<Variant>> {
    (0..=printed.len())
        .map(|p| at(printed, perms, breadth, p))
        .collect()
}

fn at(printed: &[MoveId], perms: &Perms, breadth: Breadth, p: usize) -> Vec<Variant> {
    let n = printed.len();
    let mut out = Vec::new();
    let make = |step: usize, shown: Option<MoveId>, kind: MistakeKind, moves, span| Variant {
        mistake: Mistake {
            step,
            printed: shown.map(|id| perms.mv(id)),
            kind,
        },
        moves,
        span,
        score: 0,
    };
    if p < n {
        let wrong = perms.mv(printed[p]);
        out.push(make(
            p + 1,
            Some(printed[p]),
            MistakeKind::Skipped,
            vec![],
            1,
        ));
        for id in 0..perms.len() {
            let done = perms.mv(id);
            if id == printed[p] {
                continue;
            }
            let kind = replacement(&wrong, &done);
            let likely = !matches!(kind, MistakeKind::Different { .. });
            if breadth == Breadth::Any || likely {
                out.push(make(p + 1, Some(printed[p]), kind, vec![id], 1));
            }
        }
        if p + 1 < n && printed[p + 1] != printed[p] {
            let next = perms.mv(printed[p + 1]);
            out.push(make(
                p + 1,
                Some(printed[p]),
                MistakeKind::Swapped { next },
                vec![printed[p + 1], printed[p]],
                2,
            ));
        }
    }
    if p >= 1 {
        out.push(make(
            p,
            Some(printed[p - 1]),
            MistakeKind::Repeated,
            vec![printed[p - 1]],
            0,
        ));
    }
    if breadth == Breadth::Any {
        for id in 0..perms.len() {
            if p >= 1 && id == printed[p - 1] {
                continue;
            }
            let (kind, step, shown) = if p < n && id == printed[p] {
                (MistakeKind::Repeated, p + 1, Some(printed[p]))
            } else {
                (
                    MistakeKind::Extra { done: perms.mv(id) },
                    p + 1,
                    printed.get(p).copied(),
                )
            };
            out.push(make(step, shown, kind, vec![id], 0));
        }
    }
    for v in &mut out {
        v.score = score(&v.mistake.kind);
    }
    out
}

/// For each position and span, the indices of the variants with that span.
pub(super) fn by_span(variants: &[Vec<Variant>]) -> Vec<[Vec<usize>; 3]> {
    variants
        .iter()
        .map(|list| {
            let mut spans: [Vec<usize>; 3] = Default::default();
            for (i, v) in list.iter().enumerate() {
                spans[v.span].push(i);
            }
            spans
        })
        .collect()
}
