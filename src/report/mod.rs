//! The facts of an answer as plain data, and their JSON form.
//!
//! The REST server and `cubacle solve --json` both write the solve body
//! through [`SolveReport::to_json`], so the bytes come from one place. The
//! key names and their order are part of the API: new keys are appended.

mod format;
mod json;

use std::time::Duration;

use crate::metrics::{count, count_layer_moves, Metric};
use crate::notation::Notation;
use crate::phases::from_stages;
use crate::puzzle::{LayerMove, MoveSequence, PuzzleKind, SolveEvidence, SolveStage};
use crate::ranges::{compress, write as write_ranges, Optimize};

pub use format::JsonFormat;
pub use json::{write_string, Object};

/// The name of a cube kind in reports: `3x3x3`.
pub fn kind_name(kind: PuzzleKind) -> String {
    let size = kind.size();
    format!("{size}x{size}x{size}")
}

/// The seven counts of `docs/metrics.md`, always written in this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    pub htm: usize,
    pub qtm: usize,
    pub stm: usize,
    pub etm: usize,
    pub obtm: usize,
    pub btm: usize,
    pub atm: usize,
}

impl Counts {
    fn with(count: impl Fn(Metric) -> usize) -> Counts {
        Counts {
            htm: count(Metric::Htm),
            qtm: count(Metric::Qtm),
            stm: count(Metric::Stm),
            etm: count(Metric::Etm),
            obtm: count(Metric::Obtm),
            btm: count(Metric::Btm),
            atm: count(Metric::Atm),
        }
    }

    /// The counts of a solution.
    pub fn of(moves: &MoveSequence, kind: PuzzleKind) -> Counts {
        Counts::with(|metric| count(moves, kind, metric))
    }

    /// The counts of a solution compressed to layer ranges.
    pub fn of_ranges(moves: &[LayerMove], kind: PuzzleKind) -> Counts {
        Counts::with(|metric| count_layer_moves(moves, kind, metric))
    }

    /// Appends the counts as a JSON object.
    pub fn write_json(&self, out: &mut String) {
        let mut object = Object::open(out);
        object.number("htm", self.htm);
        object.number("qtm", self.qtm);
        object.number("stm", self.stm);
        object.number("etm", self.etm);
        object.number("obtm", self.obtm);
        object.number("btm", self.btm);
        object.number("atm", self.atm);
        object.close();
    }
}

/// A solution compressed to layer ranges (`--optimize ranges`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compressed {
    pub text: String,
    pub length: usize,
    pub metrics: Counts,
}

/// One phase of the answer: its name and how many moves it takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseCount {
    pub name: String,
    pub moves: usize,
}

/// The answer to a solve request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveReport {
    pub kind: String,
    pub solution: String,
    /// The moves that undo `solution`, in the same notation.
    pub inverse: String,
    pub length: usize,
    pub metrics: Counts,
    pub compressed: Option<Compressed>,
    pub elapsed_micros: u64,
    /// The engine's own phases; empty when it reports none.
    pub phases: Vec<PhaseCount>,
    /// The name of the known pattern the start matches, when asked for.
    pub pattern_hint: Option<String>,
    pub evidence: SolveEvidence,
}

impl SolveReport {
    /// The report of `solution`, written in `notation`; `optimize` asks for the
    /// compressed form too.
    pub fn of(
        kind: PuzzleKind,
        solution: &MoveSequence,
        notation: Notation,
        optimize: Option<Optimize>,
        elapsed: Duration,
    ) -> SolveReport {
        let compressed = optimize.map(|_| {
            let packed = compress(kind, solution);
            Compressed {
                text: write_ranges(notation, kind, &packed),
                length: packed.len(),
                metrics: Counts::of_ranges(&packed, kind),
            }
        });
        SolveReport {
            kind: kind_name(kind),
            solution: notation.write(kind, solution),
            inverse: notation.write(kind, &solution.inverse()),
            length: solution.len(),
            metrics: Counts::of(solution, kind),
            compressed,
            elapsed_micros: u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX),
            phases: Vec::new(),
            pattern_hint: None,
            evidence: SolveEvidence::default(),
        }
    }

    /// Adds the phases of the engine's `stages`; unchanged when they do not
    /// cover the `length` moves of the solution exactly.
    #[must_use]
    pub fn with_phases(mut self, stages: &[SolveStage]) -> SolveReport {
        if let Some(found) = from_stages(stages, self.length) {
            self.phases = found
                .into_iter()
                .map(|phase| PhaseCount {
                    name: phase.name.to_owned(),
                    moves: phase.moves.len(),
                })
                .collect();
        }
        self
    }

    /// Adds the name of the known pattern the start matches.
    #[must_use]
    pub fn with_pattern_hint(mut self, name: Option<String>) -> SolveReport {
        self.pattern_hint = name;
        self
    }

    /// Adds evidence from the actual route/result. Inconsistent proof claims fail closed.
    #[must_use]
    pub fn with_evidence(mut self, mut evidence: SolveEvidence) -> Self {
        if evidence.proven
            && evidence
                .lower_bound
                .and_then(|bound| usize::try_from(bound).ok())
                != Some(self.metrics.htm)
        {
            evidence.proven = false;
            evidence.lower_bound = None;
        }
        self.evidence = evidence;
        self
    }

    /// Appends the report as one compact JSON object. Keys: `kind`,
    /// `solution`, `inverse`, `length`, `metrics`, then `compressed`,
    /// `compressedLength`, `compressedMetrics` when present, `elapsedMicros`,
    /// then `phases` (an array of `{name, moves}`) when the engine reports
    /// them and `patternHint` when it was asked for.
    pub fn write_json(&self, out: &mut String) {
        self.write_json_format(out, JsonFormat::default());
    }

    /// Writes the selected format, retaining the exact field order of format 1.
    pub fn write_json_format(&self, out: &mut String, format: JsonFormat) {
        let mut object = Object::open(out);
        object.string("kind", &self.kind);
        object.string("solution", &self.solution);
        object.string("inverse", &self.inverse);
        object.number("length", self.length);
        object.value("metrics", |out| self.metrics.write_json(out));
        if let Some(compressed) = &self.compressed {
            object.string("compressed", &compressed.text);
            object.number("compressedLength", compressed.length);
            object.value("compressedMetrics", |out| {
                compressed.metrics.write_json(out);
            });
        }
        object.number("elapsedMicros", self.elapsed_micros);
        if !self.phases.is_empty() {
            object.value("phases", |out| {
                out.push('[');
                for (at, phase) in self.phases.iter().enumerate() {
                    if at > 0 {
                        out.push(',');
                    }
                    let mut item = Object::open(out);
                    item.string("name", &phase.name);
                    item.number("moves", phase.moves);
                    item.close();
                }
                out.push(']');
            });
        }
        if let Some(name) = &self.pattern_hint {
            object.string("patternHint", name);
        }
        if format == JsonFormat::V2 {
            object.number("formatVersion", 2);
            object.string("toolVersion", env!("CARGO_PKG_VERSION"));
            object.value("gitCommit", |out| match format::git_commit() {
                Some(commit) => write_string(out, commit),
                None => out.push_str("null"),
            });
            object.string("frame", self.evidence.frame.as_str());
            object.value("proven", |out| {
                out.push_str(if self.evidence.proven {
                    "true"
                } else {
                    "false"
                })
            });
            object.value("lowerBound", |out| match self.evidence.lower_bound {
                Some(bound) => out.push_str(&bound.to_string()),
                None => out.push_str("null"),
            });
        }
        object.close();
    }

    /// The report as one compact JSON object.
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        self.write_json(&mut out);
        out
    }

    /// The report as one compact JSON object in the requested format.
    pub fn to_json_format(&self, format: JsonFormat) -> String {
        let mut out = String::new();
        self.write_json_format(&mut out, format);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::parse_expression;

    fn report(optimize: Option<Optimize>) -> SolveReport {
        let moves = parse_expression("R U R'", PuzzleKind::Cube3).unwrap();
        SolveReport::of(
            PuzzleKind::Cube3,
            &moves,
            Notation::Wca,
            optimize,
            Duration::from_micros(1234),
        )
    }

    #[test]
    fn the_body_has_the_documented_keys_in_order() {
        assert_eq!(
            report(None).to_json_format(JsonFormat::V1),
            "{\"kind\":\"3x3x3\",\"solution\":\"R U R'\",\"inverse\":\"R U' R'\",\"length\":3,\
             \"metrics\":{\"htm\":3,\"qtm\":3,\"stm\":3,\"etm\":3,\"obtm\":3,\"btm\":3,\"atm\":3},\
             \"elapsedMicros\":1234}"
        );
    }

    #[test]
    fn the_compressed_keys_sit_before_the_elapsed_time() {
        let text = report(Some(Optimize::Ranges)).to_json();
        let keys = [
            "\"kind\"",
            "\"solution\"",
            "\"inverse\"",
            "\"length\"",
            "\"metrics\"",
            "\"compressed\"",
            "\"compressedLength\"",
            "\"compressedMetrics\"",
            "\"elapsedMicros\"",
        ];
        let places: Vec<usize> = keys.iter().map(|k| text.find(k).unwrap()).collect();
        assert!(places.windows(2).all(|w| w[0] < w[1]), "{text}");
    }

    #[test]
    fn phases_and_the_hint_come_after_the_elapsed_time() {
        let mut full = report(None);
        full.phases = vec![
            PhaseCount {
                name: "centers".into(),
                moves: 2,
            },
            PhaseCount {
                name: "edges".into(),
                moves: 1,
            },
        ];
        full.pattern_hint = Some("superflip".into());
        let text = full.to_json_format(JsonFormat::V1);
        assert!(
            text.ends_with(
                "\"elapsedMicros\":1234,\"phases\":[{\"name\":\"centers\",\"moves\":2},\
                 {\"name\":\"edges\",\"moves\":1}],\"patternHint\":\"superflip\"}"
            ),
            "{text}"
        );
        assert!(!report(None).to_json().contains("phases"));
    }

    #[test]
    fn stages_that_do_not_cover_the_moves_give_no_phases() {
        let stage = SolveStage { name: "x", end: 2 };
        assert!(report(None).with_phases(&[stage]).phases.is_empty());
    }

    #[test]
    fn text_in_a_report_is_escaped() {
        let mut report = report(None);
        report.solution = "a\"b\\".into();
        assert!(report.to_json().contains("\"solution\":\"a\\\"b\\\\\""));
    }
    #[test]
    fn version_two_appends_metadata_after_every_original_field() {
        let one = report(None).to_json_format(JsonFormat::V1);
        let two = report(None).to_json();
        assert!(two.starts_with(&one[..one.len() - 1]));
        assert!(two.contains("\"formatVersion\":2,\"toolVersion\":"));
        assert!(two.ends_with("\"frame\":\"fixed\",\"proven\":false,\"lowerBound\":null}"));
        let proved = report(None).with_evidence(SolveEvidence::optimal(
            crate::puzzle::SolveFrame::AnyOrientation,
            3,
        ));
        assert!(proved
            .to_json()
            .ends_with("\"frame\":\"any-orientation\",\"proven\":true,\"lowerBound\":3}"));
        assert_eq!(proved.to_json_format(JsonFormat::V1), one);
    }

    #[test]
    fn inconsistent_or_incomplete_proof_evidence_is_not_promoted() {
        let report =
            report(None).with_evidence(SolveEvidence::optimal(crate::puzzle::SolveFrame::Fixed, 4));
        assert!(!report.evidence.proven);
        assert_eq!(report.evidence.lower_bound, None);
        assert_eq!("1".parse::<JsonFormat>(), Ok(JsonFormat::V1));
        assert_eq!("2".parse::<JsonFormat>(), Ok(JsonFormat::V2));
        assert!("3".parse::<JsonFormat>().is_err());
    }
}
