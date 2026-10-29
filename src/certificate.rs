//! The certificate of an exact search (`solve --certificate`) and its check
//! (`verify`).
//!
//! A certificate is plain text, one `key: value` per line, stable across
//! versions of the same format number. It carries everything a second program
//! needs to recompute the answer: the cube and the twist at the start, the
//! target, the moves and the zero-cost rotations, and the end state. It also
//! records what the search did: the lower bound at the start, the nodes of
//! every depth searched out, and a digest of every table. `verify` replays the
//! moves; it does not rebuild the tables, so the shortness is checked only for
//! consistency (the depths from the bound to the answer are all accounted for).

use cubacle::engines::three::SearchRecord;
use cubacle::notation::parse_partial;
use cubacle::puzzle::{
    parse_twist_masks, rotation_moves, track_moves, twist_masks_text, CenterTwist, Facelets,
    MoveSequence, Target, TWIST_CONVENTION,
};

use crate::picture_cmd::{End, Picture};

const FORMAT: &str = "cubacle-certificate 1";

/// What the search ended in.
pub(crate) enum Outcome<'a> {
    Solved {
        moves: &'a MoveSequence,
        end: &'a End,
    },
    /// No answer: the message says why (a limit, or searched out).
    Failed(&'a str),
}

pub(crate) struct Certificate<'a> {
    pub(crate) start: &'a Facelets,
    pub(crate) picture: &'a Picture,
    pub(crate) target: &'a Target,
    pub(crate) any_orientation: bool,
    pub(crate) outcome: Outcome<'a>,
    pub(crate) record: Option<&'a SearchRecord>,
}

fn target_text(target: &Target) -> String {
    target.to_text().unwrap_or_else(|| "-".into())
}

impl Certificate<'_> {
    pub(crate) fn text(&self) -> String {
        let mut lines = vec![
            FORMAT.to_owned(),
            format!("convention: {TWIST_CONVENTION}"),
            "puzzle: 3x3x3".into(),
            format!("start: {}", self.start),
            format!("start-twist: {}", self.picture.twist),
        ];
        if !self.picture.frame.is_empty() {
            lines.push(format!("frame: {}", self.picture.frame));
        }
        lines.push(format!("target: {}", target_text(self.target)));
        lines.push(format!(
            "target-twist: {}",
            twist_masks_text(self.target.twist_masks())
        ));
        lines.push(format!(
            "orientation: {}",
            if self.any_orientation { "any" } else { "fixed" }
        ));
        match &self.outcome {
            Outcome::Solved { moves, end } => {
                lines.push(format!("moves: {moves}"));
                lines.push(format!("length: {}", moves.len()));
                lines.push(format!("final-rotation: {}", end.rotation));
                lines.push(format!("end: {}", end.state));
                lines.push(format!("end-twist: {}", end.twist));
                lines.push("result: shortest, searched out below its length".into());
            }
            Outcome::Failed(why) => lines.push(format!("result: not proven, {why}")),
        }
        if let Some(record) = self.record {
            lines.push(format!("views: {}", record.views));
            lines.push(format!("lower-bound: {}", record.start_bound));
            for (depth, nodes) in &record.completed {
                lines.push(format!("searched-out: {depth} {nodes}"));
            }
            if let Some((depth, nodes)) = record.found {
                lines.push(format!("found: {depth} {nodes}"));
            }
            for table in &record.tables {
                lines.push(format!(
                    "table: {} {} fnv1a64:{:016x}",
                    table.name, table.cells, table.fnv1a64
                ));
            }
        }
        lines.join("\n")
    }
}

/// The `key: value` lines of a certificate.
fn field<'a>(lines: &[(&'a str, &'a str)], key: &str) -> Option<&'a str> {
    lines.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

/// Replays a certificate: the stickers through the facelet model, the twist
/// through the center tracker, then the target on both. Returns a report line.
pub(crate) fn verify(text: &str) -> Result<String, String> {
    let mut input = text.lines();
    if input.next().map(str::trim) != Some(FORMAT) {
        return Err(format!(
            "not a certificate: the first line must be {FORMAT:?}"
        ));
    }
    let lines: Vec<(&str, &str)> = input
        .filter_map(|line| line.split_once(": "))
        .map(|(k, v)| (k.trim(), v.trim()))
        .collect();
    let need = |key: &str| {
        field(&lines, key).ok_or_else(|| format!("the certificate has no {key:?} line"))
    };
    if need("convention")? != TWIST_CONVENTION {
        return Err(format!(
            "the twist convention {:?} is not the one this program replays ({TWIST_CONVENTION})",
            need("convention")?
        ));
    }
    let start = Facelets::parse(need("start")?).map_err(|e| format!("start: {e}"))?;
    let twist: CenterTwist = need("start-twist")?
        .parse()
        .map_err(|e| format!("start-twist: {e}"))?;
    let target_line = need("target")?;
    let target = match parse_partial(target_line, None).map_err(|e| format!("target: {e}"))? {
        Some(target) => target,
        None => {
            Target::complete(&Facelets::parse(target_line).map_err(|e| format!("target: {e}"))?)
        }
    };
    let masks =
        parse_twist_masks(need("target-twist")?).map_err(|e| format!("target-twist: {e}"))?;
    let target = target.with_twist(masks);
    let any = match need("orientation")? {
        "any" => true,
        "fixed" => false,
        other => return Err(format!("orientation: {other:?} is neither any nor fixed")),
    };
    let result = need("result")?;
    if !result.starts_with("shortest") {
        return Ok(format!(
            "a certificate without an answer ({result}); nothing to replay"
        ));
    }
    let moves: MoveSequence = need("moves")?.parse().map_err(|e| format!("moves: {e}"))?;
    if moves.moves().iter().any(|m| m.depth != 1 || m.wide) {
        return Err(
            "moves: only outer face turns are counted moves; rotations belong in final-rotation"
                .into(),
        );
    }
    let end = start.apply(&moves).map_err(|e| e.to_string())?;
    let tracked = track_moves(twist, &moves).map_err(|e| e.to_string())?;
    let rotation = need("final-rotation")?;
    if !any && !rotation.is_empty() {
        return Err("final-rotation: a fixed orientation allows no rotation".into());
    }
    let turned = rotation_moves(rotation).map_err(|e| format!("final-rotation: {e}"))?;
    let end = end.apply(&turned).map_err(|e| e.to_string())?;
    let end_twist = tracked.twist.rotated(rotation).map_err(|e| e.to_string())?;
    if end.to_string() != need("end")? {
        return Err("end: the stickers after the moves differ from the certificate".into());
    }
    if end_twist.to_string() != need("end-twist")? {
        return Err(format!(
            "end-twist: the replay gives {end_twist}, the certificate says {}",
            need("end-twist")?
        ));
    }
    if !target.matches(&end) || !target.twist_met(&end_twist) {
        return Err("the end state does not meet the target".into());
    }
    let length = need("length")?;
    if length != moves.len().to_string() {
        return Err(format!(
            "length: {length} does not match the {} moves",
            moves.len()
        ));
    }
    // the searched-out depths run from the bound up to the length, none missing
    let bound: usize = need("lower-bound")?
        .parse()
        .map_err(|_| "lower-bound is no number".to_owned())?;
    let depths: Vec<usize> = lines
        .iter()
        .filter(|(k, _)| *k == "searched-out")
        .filter_map(|(_, v)| v.split_whitespace().next()?.parse().ok())
        .collect();
    let want: Vec<usize> = (bound..moves.len()).collect();
    if depths != want {
        return Err(format!(
            "searched-out: the depths {depths:?} are not the depths {want:?} from the lower bound to the answer"
        ));
    }
    Ok(format!(
        "valid: {} moves reach the target, twist {} to {end_twist}; the search accounts for every depth from the lower bound {bound} up to the length (the tables are not rebuilt)",
        moves.len(),
        twist
    ))
}
