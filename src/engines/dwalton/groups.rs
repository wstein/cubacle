//! The stages of the answer as a learner sees them.
//!
//! The reference has many small steps; they are grouped here into six
//! stages, the same for both sizes. All six are in every answer (a stage that
//! needs no moves is empty, not left out). The cube at the end of a stage is
//! in the state the engine reached there: the inner centers are done at the
//! end of `inner wings and centers`, all centers and wings at the end of
//! `outer wings and centers`.
//!
//! | Stage | 6x6x6 steps | 7x7x7 steps |
//! | --- | --- | --- |
//! | `left/right centers staged` | inner x-centers staged and obliques paired, centers staged | inner centers staged, obliques paired, centers staged |
//! | `up/down centers staged` | the same for up and down | the same for up and down |
//! | `centers prepared` | steps 50 and 60 (centers reducible) | steps 40, 50 and 60 (bars, daisy) |
//! | `inner wings and centers` | the inner wings and centers, by the 4x4x4 engine | the inner wings and centers, by the 5x5x5 engine |
//! | `outer wings and centers` | the outer wings and centers, by the 5x5x5 engine | the outer wings and centers, by the 5x5x5 engine |
//! | `3x3 stage` | the 3x3x3 solver | the 3x3x3 solver |

use crate::puzzle::{stage_ends, Facelets, Move, MoveSequence, SolveStage};

/// The stages in order.
pub(super) const GROUPS: [&str; 6] = [
    "left/right centers staged",
    "up/down centers staged",
    "centers prepared",
    "inner wings and centers",
    "outer wings and centers",
    "3x3 stage",
];

/// The stage a step of the reference belongs to.
pub(super) fn group_of(step: &str) -> &'static str {
    match step {
        "LR inner-x staged, obliques paired"
        | "LR inner centers staged"
        | "LR obliques paired"
        | "LR centers staged" => GROUPS[0],
        "UD inner-x staged, obliques paired"
        | "UD inner centers staged"
        | "UD obliques paired"
        | "UD centers staged" => GROUPS[1],
        "inner wings (4x4x4)" | "inner wings (5x5x5)" => GROUPS[3],
        "outer wings (5x5x5)" => GROUPS[4],
        "3x3x3" => GROUPS[5],
        _ => GROUPS[2],
    }
}

/// The stages of an answer. `marks` holds each step of the engine in order,
/// with the length of `raw` at its end; `fin` is `raw` after normalizing.
pub(super) fn stages(
    start: &Facelets,
    raw: &[Move],
    marks: &[(&'static str, usize)],
    fin: &[Move],
) -> Vec<SolveStage> {
    let ends = raw_ends(marks);
    let mapped = stage_ends(start, raw, &ends, fin, &[]);
    GROUPS
        .iter()
        .zip(mapped)
        .map(|(&name, end)| SolveStage { name, end })
        .collect()
}

/// Moves a stage end later where normalizing merged moves across it, so the
/// cube is never in the state the engine reached there. The stage end goes to
/// the first position where every sticker that was in its solved place at the
/// engine's end is in place again, so the facts of the stage (centers and
/// wings done, outer faces only in the 3x3 stage) hold at its end. The moves
/// themselves are not changed.
pub(super) fn repair(
    start: &Facelets,
    raw: &[Move],
    raw_ends: &[usize],
    fin: &[Move],
    stages: &mut [SolveStage],
) {
    let solved = Facelets::solved(start.kind());
    let state = |m: &[Move]| start.apply(&MoveSequence::new(m.to_vec())).ok();
    for (stage, &raw_end) in stages.iter_mut().zip(raw_ends) {
        if stage.end >= fin.len() {
            continue;
        }
        let (Some(have), Some(want)) = (state(&fin[..stage.end]), state(&raw[..raw_end])) else {
            continue;
        };
        if have == want {
            continue;
        }
        let kept: Vec<usize> = (0..solved.stickers().len())
            .filter(|&i| want.stickers()[i] == solved.stickers()[i])
            .collect();
        let mut at = stage.end;
        let mut cur = have;
        while at < fin.len() {
            if kept
                .iter()
                .all(|&i| cur.stickers()[i] == solved.stickers()[i])
            {
                break;
            }
            let Ok(next) = cur.apply(&MoveSequence::new(vec![fin[at]])) else {
                break;
            };
            cur = next;
            at += 1;
        }
        stage.end = at;
    }
    for i in 1..stages.len() {
        stages[i].end = stages[i].end.max(stages[i - 1].end);
    }
}

/// The end of each stage in the raw moves.
pub(super) fn raw_ends(marks: &[(&'static str, usize)]) -> Vec<usize> {
    let mut ends = Vec::with_capacity(GROUPS.len());
    let mut at = 0;
    for group in GROUPS {
        for &(name, end) in marks {
            if group_of(name) == group {
                at = end;
            }
        }
        ends.push(at);
    }
    ends
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_step_of_the_reference_has_its_stage() {
        let steps = [
            ("LR inner-x staged, obliques paired", 0),
            ("LR centers staged", 0),
            ("UD inner-x staged, obliques paired", 1),
            ("UD centers staged", 1),
            ("LR centers reducible", 2),
            ("UD FB centers reducible", 2),
            ("LR inner centers staged", 0),
            ("LR obliques paired", 0),
            ("UD inner centers staged", 1),
            ("UD obliques paired", 1),
            ("LR centers vertical bars", 2),
            ("LR solved, UD vertical bars", 2),
            ("centers daisy solved", 2),
            ("inner wings (4x4x4)", 3),
            ("inner wings (5x5x5)", 3),
            ("outer wings (5x5x5)", 4),
            ("3x3x3", 5),
        ];
        for (step, group) in steps {
            assert_eq!(group_of(step), GROUPS[group], "{step}");
        }
    }

    #[test]
    fn marks_become_ends_and_an_unused_group_is_empty() {
        let start = Facelets::solved(crate::puzzle::PuzzleKind::Cube6);
        let raw: Vec<Move> = "U R U' R'"
            .parse::<crate::puzzle::MoveSequence>()
            .unwrap()
            .moves()
            .to_vec();
        let marks = [("LR centers staged", 2), ("3x3x3", 4)];
        let stages = stages(&start, &raw, &marks, &raw);
        let ends: Vec<usize> = stages.iter().map(|s| s.end).collect();
        assert_eq!(ends, [2, 2, 2, 2, 2, 4]);
    }
}
