//! The counting rules, one arm per metric.

use super::unit::Unit;
use super::Metric;
use crate::puzzle::PuzzleKind;

/// Outer-block turns a unit is worth: 0 for a rotation, 1 for a block that
/// touches an outer face, 2 for an inner block (two outer-block turns make it).
fn outer_blocks(kind: PuzzleKind, unit: &Unit) -> usize {
    if unit.is_rotation(kind) {
        0
    } else if unit.touches_outside(kind) {
        1
    } else {
        2
    }
}

fn per_unit(kind: PuzzleKind, unit: &Unit, metric: Metric) -> usize {
    let rotation = unit.is_rotation(kind);
    let angle = if unit.half { 2 } else { 1 };
    match metric {
        Metric::Htm | Metric::Obtm => outer_blocks(kind, unit),
        Metric::Qtm => outer_blocks(kind, unit) * angle,
        Metric::Stm => {
            if rotation {
                0
            } else {
                unit.layers()
            }
        }
        Metric::Etm => 1,
        Metric::Btm => usize::from(!rotation),
        Metric::Atm => 0,
    }
}

/// Runs of turns about one axis. A rotation joins a run of its own axis (and
/// adds nothing) and ends a run of any other axis.
fn axial_runs(kind: PuzzleKind, units: &[Unit]) -> usize {
    let mut runs = 0;
    let mut current: Option<u8> = None;
    for unit in units {
        if unit.is_rotation(kind) {
            if current != Some(unit.axis) {
                current = None;
            }
        } else if current != Some(unit.axis) {
            runs += 1;
            current = Some(unit.axis);
        }
    }
    runs
}

pub(crate) fn count_units(kind: PuzzleKind, units: &[Unit], metric: Metric) -> usize {
    if metric == Metric::Atm {
        return axial_runs(kind, units);
    }
    units.iter().map(|unit| per_unit(kind, unit, metric)).sum()
}
