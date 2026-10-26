//! The stages of the answer as a learner sees them.
//!
//! The reference has many small steps; they are grouped here into six
//! stages. All six are in every answer (a stage that needs no moves is
//! empty, not left out), and a stage over 20 moves is cut at the ends of its
//! steps into parts `name (1/2)`. The borders are a grouping for the eye: the
//! cube is where the engine left it at a step end, give or take the moves that
//! normalizing merged across it.
//!
//! | Stage (6x6x6 / 7x7x7) | Engine steps |
//! | --- | --- |
//! | `LR centers on L/R` | inner and outer centers of the left and right faces staged, obliques paired |
//! | `UD/FB centers on their axes` | the same for up and down |
//! | `inner centers solved` / `centers solved` | steps 40 to 60 (centers reducible, bars, daisy) |
//! | `inner wings paired` / `middle wings paired` | the 4x4x4 engine (6x6x6), the 5x5x5 engine on the middle orbit (7x7x7) |
//! | `outer wings paired, centers solved` / `outer wings paired` | the 5x5x5 engine on the outer orbit |
//! | `3x3 stage` | the 3x3x3 solver |

use crate::puzzle::{
    stage_ends, stage_parts, Facelets, Move, MoveSequence, PuzzleKind, SolveStage,
};

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

/// The names a learner sees for the six stages, per cube size.
fn shown(kind: PuzzleKind) -> [&'static str; 6] {
    if kind == PuzzleKind::Cube6 {
        [
            "LR centers on L/R",
            "UD/FB centers on their axes",
            "inner centers solved",
            "inner wings paired",
            "outer wings paired, centers solved",
            "3x3 stage",
        ]
    } else {
        [
            "LR centers on L/R",
            "UD/FB centers on their axes",
            "centers solved",
            "middle wings paired",
            "outer wings paired",
            "3x3 stage",
        ]
    }
}

/// The stages of an answer. `marks` holds each step of the engine in order,
/// with the length of `raw` at its end; `fin` is `raw` after normalizing. A
/// stage over 20 moves is cut at the ends of its steps.
pub(super) fn stages(
    start: &Facelets,
    raw: &[Move],
    marks: &[(&'static str, usize)],
    fin: &[Move],
) -> Vec<SolveStage> {
    let names = shown(start.kind());
    let mut ends = Vec::new();
    let mut named = Vec::new();
    let mut at = 0;
    for (group, name) in GROUPS.iter().zip(names) {
        let mut steps = Vec::new();
        let mut last = at;
        for &(step, end) in marks {
            if group_of(step) == *group {
                steps.push(end - last);
                last = end;
            }
        }
        for (part, len) in stage_parts(name, &steps) {
            at += len;
            ends.push(at);
            named.push(part);
        }
    }
    let mapped = stage_ends(start, raw, &ends, fin, &[]);
    let mut stages: Vec<SolveStage> = named
        .into_iter()
        .zip(mapped)
        .map(|(name, end)| SolveStage { name, end })
        .collect();
    repair(start, raw, &ends, fin, &mut stages);
    // The 3x3 stage turns outer faces only: a slice or wide turn that
    // normalizing moved behind its first outer turn still belongs to the
    // stage before it.
    let plain = |m: &Move| m.depth == 1 && !m.wide;
    if let (Some(last), true) = (fin.iter().rposition(|m| !plain(m)), stages.len() >= 2) {
        let before = stages.len() - 2;
        stages[before].end = stages[before].end.max(last + 1);
    }
    stages
}

/// The most moves a stage end is moved later by [`repair`].
const REPAIR_WINDOW: usize = 4;

/// Moves a stage end later where normalizing merged moves across it, so the
/// cube is never in the state the engine reached there. The stage end goes to
/// the first position where every sticker that was in its solved place at the
/// engine's end is in place again, so the facts of the stage (centers and
/// wings done, outer faces only in the 3x3 stage) hold at its end. The moves
/// themselves are not changed.
fn repair(
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
        // a merged run is a few moves long; kept stickers that match by
        // chance are not found again soon
        while at < fin.len().min(stage.end + REPAIR_WINDOW) {
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
        if kept
            .iter()
            .all(|&i| cur.stickers()[i] == solved.stickers()[i])
        {
            stage.end = at;
        }
    }
    for i in 1..stages.len() {
        stages[i].end = stages[i].end.max(stages[i - 1].end);
    }
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
    fn a_long_stage_is_cut_at_step_ends_and_names_follow_the_size() {
        let start = Facelets::solved(PuzzleKind::Cube7);
        let raw: Vec<Move> = vec!["U".parse().unwrap(); 4];
        // steps of 11, 12 and 11 moves in the first group (marks are lengths of raw)
        let marks = [
            ("LR inner centers staged", 11),
            ("LR obliques paired", 23),
            ("LR centers staged", 34),
            ("3x3x3", 34),
        ];
        let names: Vec<&str> = stages(&start, &raw, &marks, &raw)
            .iter()
            .map(|s| s.name)
            .collect();
        assert_eq!(names[0], "LR centers on L/R (1/3)");
        assert_eq!(names[3], "UD/FB centers on their axes");
        assert_eq!(names.last(), Some(&"3x3 stage"));
        assert_eq!(shown(PuzzleKind::Cube6)[3], "inner wings paired");
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
