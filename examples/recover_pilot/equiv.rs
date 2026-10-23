//! Whether an explanation that names other steps than the injected ones still
//! gives the very cube the person holds (commuting moves, or one move that
//! can be read two ways).

use cubacle::puzzle::{Facelets, Move, MoveSequence};
use cubacle::recover::{Finding, Mistake, MistakeKind};

/// The moves the explanation says were done: the printed ones up to `done`
/// with the mistakes applied, then the stray moves.
pub fn replay(solution: &MoveSequence, finding: &Finding) -> Option<MoveSequence> {
    let Finding::Mistakes {
        mistakes,
        done,
        strays,
        ..
    } = finding
    else {
        return None;
    };
    let printed = solution.moves();
    let mut out: Vec<Move> = Vec::new();
    let mut skip_next = false;
    for step in 1..=*done {
        if std::mem::take(&mut skip_next) {
            continue;
        }
        let mv = printed[step - 1];
        match mistakes.iter().find(|m| m.step == step) {
            None => out.push(mv),
            Some(Mistake { kind, .. }) => match kind {
                MistakeKind::Skipped => {}
                MistakeKind::WrongAmount { done }
                | MistakeKind::WrongDirection { done }
                | MistakeKind::WrongLayers { done }
                | MistakeKind::WrongFace { done }
                | MistakeKind::Different { done } => out.push(*done),
                MistakeKind::Swapped { next } => {
                    out.push(*next);
                    out.push(mv);
                    skip_next = true;
                }
                MistakeKind::Repeated => {
                    out.push(mv);
                    out.push(mv);
                }
                MistakeKind::Extra { done } => {
                    out.push(*done);
                    out.push(mv);
                }
            },
        }
    }
    // An extra move after the last printed move belongs to step done + 1.
    if let Some(Mistake {
        kind: MistakeKind::Extra { done },
        printed: None,
        ..
    }) = mistakes.iter().find(|m| m.step == done + 1)
    {
        out.push(*done);
    }
    out.extend(strays.iter().copied());
    Some(MoveSequence::new(out))
}

/// The explanation gives exactly `now` from `original`.
pub fn same_cube(
    original: &Facelets,
    solution: &MoveSequence,
    finding: &Finding,
    now: &Facelets,
) -> bool {
    replay(solution, finding)
        .and_then(|seq| original.apply(&seq).ok())
        .is_some_and(|cube| cube == *now)
}
