//! The seven move-counting methods and their names.

use std::fmt;
use std::str::FromStr;

use crate::puzzle::PuzzleKind;

/// A way of counting the moves of a sequence. `docs/metrics.md` defines each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Metric {
    /// Half-turn (face-turn) metric: a turn of a block that touches an outer
    /// face is 1 whatever its angle, an inner block is 2.
    Htm,
    /// Quarter-turn metric: the HTM count, a half turn counting double.
    Qtm,
    /// Slice-turn (layer-turn) metric: the number of layers turned.
    Stm,
    /// Execution-turn metric: every block turn and every rotation is 1.
    Etm,
    /// Outer-block-turn metric: the HTM count, named for big cubes.
    Obtm,
    /// Block-turn metric: every turn of a contiguous block is 1.
    Btm,
    /// Axial-turn metric: a run of turns about one axis is 1.
    Atm,
}

impl Metric {
    /// All metrics, in the order the CLI lists them.
    pub const ALL: [Metric; 7] = [
        Metric::Htm,
        Metric::Qtm,
        Metric::Stm,
        Metric::Etm,
        Metric::Obtm,
        Metric::Btm,
        Metric::Atm,
    ];

    /// The lower-case name: `htm`, `qtm`, `stm`, `etm`, `obtm`, `btm`, `atm`.
    pub fn name(self) -> &'static str {
        match self {
            Metric::Htm => "htm",
            Metric::Qtm => "qtm",
            Metric::Stm => "stm",
            Metric::Etm => "etm",
            Metric::Obtm => "obtm",
            Metric::Btm => "btm",
            Metric::Atm => "atm",
        }
    }

    /// The metrics shown by default for a cube: `htm qtm stm` up to the 3x3x3,
    /// `obtm qtm stm btm` from the 4x4x4 (where blocks and inner layers matter).
    pub fn headline(kind: PuzzleKind) -> &'static [Metric] {
        if kind.size() <= 3 {
            &[Metric::Htm, Metric::Qtm, Metric::Stm]
        } else {
            &[Metric::Obtm, Metric::Qtm, Metric::Stm, Metric::Btm]
        }
    }
}

impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for Metric {
    type Err = String;

    fn from_str(text: &str) -> Result<Metric, String> {
        let lower = text.to_ascii_lowercase();
        let name = match lower.as_str() {
            "ftm" => "htm",
            "ltm" => "stm",
            other => other,
        };
        Metric::ALL
            .into_iter()
            .find(|metric| metric.name() == name)
            .ok_or_else(|| {
                format!("metric must be one of htm qtm stm etm obtm btm atm, not {text:?}")
            })
    }
}
