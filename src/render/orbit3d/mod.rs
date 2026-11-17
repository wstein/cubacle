//! Whole-cube rotations (`x`, `y`, `z`, their primes and half turns) as an arc
//! on an orbit around the rotation axis.
//!
//! The axis (a thin line through the cube that sticks out on both sides) and the
//! orbit (a thin ring around the cube center) are curves in model coordinates,
//! projected with the same [`Projection`] as the cube. The soft arrow of
//! `arrow.rs` runs on a part of the orbit. Every sample of a curve is classified:
//! does the cube hide it (the ray towards the viewer hits the cube), and is it
//! in front of or behind the axis line? The polygons come back in painter
//! order: what the cube hides before the cube, the rest after it, and in each
//! group the back of the orbit, the axis, the front of the orbit.

mod curve;
#[cfg(test)]
mod tests;

use curve::{
    axis_point, cut_end, in_front_of_axis, length, occluded, project_all, reversed, ring_point, P2,
    P3,
};

use super::action::Action;
use super::arrow::{arrowhead, halo_grow, mark_beside, ribbon, shaft_width, sub, unit, Metrics};
use super::geometry::Projection;
use super::round::disc;
use super::scene::{Part, Shape, WIDTH};
use super::style::{Rgb, ARROW_VIOLET, ORBIT_AXIS, ORBIT_RING, WHITE};
use crate::puzzle::Turn;

/// Radius of the orbit in half cube edges (model units per `n`). Larger values
/// push the picture out of the canvas.
const RADIUS: f64 = 1.52;
/// How far the axis line sticks out beyond the orbit, in half cube edges.
const OVERHANG: f64 = 0.36;
/// The cube body is drawn this much larger than the stickers' cube.
const BODY_RIM: f64 = 0.1;
/// Line widths of the axis and of the orbit, in canvas units.
const AXIS_WIDTH: f64 = 1.5;
const RING_WIDTH: f64 = 1.0;
/// Every polygon stays this far from the canvas edge and above the notation,
/// which starts at 94 (`label.rs`).
const MARGIN: f64 = 1.0;
const BOTTOM: f64 = 92.0;
/// Angle of the middle of the arc, in degrees, per axis: over the edge between
/// two visible faces (`x`: Up and Front, `y`: Front and Right, `z`: Up and Right).
const MIDDLE: [f64; 3] = [45.0, -45.0, -45.0];
/// Full span of the arc in degrees per axis: a quarter and a half turn. The
/// values give `x`, `y` and `z` about the same length on the screen.
const SPAN: [[f64; 2]; 3] = [[110.0, 146.4], [110.0, 146.4], [105.0, 127.5]];
/// Samples of the axis line, of the full orbit and steps of the arc.
const AXIS_SAMPLES: usize = 240;
const RING_SAMPLES: usize = 360;
const ARC_STEPS: usize = 72;

/// Where a piece lies relative to the axis line; the order is the painter order.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Depth {
    Behind,
    Axis,
    Front,
}

/// Polygons that are painted together.
struct Item {
    hidden: bool,
    depth: Depth,
    shapes: Vec<Shape>,
}

/// The polygons of a rotation icon: what the cube hides and what it does not.
pub(super) struct Layers {
    pub(super) behind: Vec<Shape>,
    pub(super) front: Vec<Shape>,
}

struct Orbit<'a> {
    axis: usize,
    /// Orbit radius in model units.
    radius: f64,
    /// Half edge of the cube that hides things, in model units.
    reach: f64,
    n: f64,
    half: bool,
    prime: bool,
    proj: &'a Projection,
    pixels_per_unit: f64,
}

impl<'a> Orbit<'a> {
    fn new(action: &Action, n: usize, proj: &'a Projection, pixels_per_unit: f64) -> Orbit<'a> {
        let Action::Rotate { turn, .. } = action else {
            panic!("not a whole-cube rotation: {action}");
        };
        let (axis, _) = action.axis_sign();
        let n = n as f64;
        Orbit {
            axis,
            radius: RADIUS * n,
            reach: n + BODY_RIM,
            n,
            half: action.is_half(),
            prime: *turn == Turn::CounterClockwise,
            proj,
            pixels_per_unit,
        }
    }
}

/// The picture of the rotation `action` on a cube of size `n`.
pub(super) fn shapes(action: &Action, n: usize, proj: &Projection, pixels_per_unit: f64) -> Layers {
    let orbit = Orbit::new(action, n, proj, pixels_per_unit);
    let mut items = orbit.axis_items();
    items.extend(orbit.ring_items());
    items.extend(orbit.arrow_items());
    // stable: within one depth the axis, the ring and the arrow keep this order
    items.sort_by_key(|item| item.depth);
    let mut layers = Layers {
        behind: Vec::new(),
        front: Vec::new(),
    };
    for item in items {
        let target = if item.hidden {
            &mut layers.behind
        } else {
            &mut layers.front
        };
        target.extend(item.shapes);
    }
    if orbit.half {
        layers.front.extend(orbit.half_mark());
    }
    layers
}

/// Is `q` at least `extra` inside the drawing area?
fn inside_canvas(q: P2, extra: f64) -> bool {
    let edge = MARGIN + extra;
    q[0] >= edge && q[0] <= WIDTH - edge && q[1] >= edge && q[1] <= BOTTOM - extra
}

impl Orbit<'_> {
    fn hidden(&self, p: P3) -> bool {
        occluded(p, self.reach)
    }

    fn depth_of(&self, p: P3) -> Depth {
        if in_front_of_axis(self.axis, p) {
            Depth::Front
        } else {
            Depth::Behind
        }
    }

    /// Start and end angle of the arc.
    fn arc(&self) -> (f64, f64) {
        let span = SPAN[self.axis][usize::from(self.half)].to_radians();
        let middle = MIDDLE[self.axis].to_radians();
        (middle - span / 2.0, middle + span / 2.0)
    }

    /// The axis line from `-reach` to `reach`: as long as the orbit allows and
    /// the canvas permits, with its square ends inside the drawing area.
    fn axis_reach(&self) -> f64 {
        let mut reach = self.radius + OVERHANG * self.n;
        let fits = |s: f64| {
            let ends = [s, -s].map(|s| self.proj.project(axis_point(self.axis, s)));
            ends.iter().all(|&q| inside_canvas(q, AXIS_WIDTH / 2.0))
        };
        while reach > 0.0 && !fits(reach) {
            reach -= 0.05;
        }
        reach
    }

    fn axis_items(&self) -> Vec<Item> {
        let reach = self.axis_reach();
        let points: Vec<P3> = (0..=AXIS_SAMPLES)
            .map(|i| {
                axis_point(
                    self.axis,
                    reach * (2.0 * i as f64 / AXIS_SAMPLES as f64 - 1.0),
                )
            })
            .collect();
        self.runs(&points, &|_| Depth::Axis)
            .into_iter()
            .map(|run| self.line(run, AXIS_WIDTH, ORBIT_AXIS))
            .collect()
    }

    fn ring_items(&self) -> Vec<Item> {
        let points: Vec<P3> = (0..=RING_SAMPLES)
            .map(|i| {
                let angle = std::f64::consts::TAU * i as f64 / RING_SAMPLES as f64;
                ring_point(self.axis, self.radius, angle)
            })
            .collect();
        self.runs(&points, &|p| self.depth_of(p))
            .into_iter()
            .map(|run| self.line(run, RING_WIDTH, ORBIT_RING))
            .collect()
    }

    fn line(&self, run: Run, width: f64, ink: Rgb) -> Item {
        let flat = project_all(&run.points, self.proj);
        Item {
            hidden: run.hidden,
            depth: run.depth,
            shapes: vec![Shape::new(Part::Arrow, ribbon(&flat, width), ink)],
        }
    }

    fn metrics(&self) -> Metrics {
        Metrics::new(shaft_width(self.n as usize, self.pixels_per_unit), 1.0)
    }

    /// Where the soft arrow runs on the orbit. A prime has its head at the start,
    /// a half turn at both ends.
    fn arrow_plan(&self) -> ArrowPlan {
        let (start, end) = self.arc();
        let points: Vec<P3> = (0..=ARC_STEPS)
            .map(|i| {
                let angle = start + (end - start) * i as f64 / ARC_STEPS as f64;
                ring_point(self.axis, self.radius, angle)
            })
            .collect();
        let head_len = self.metrics().head_len;
        let head_at_end = !self.prime || self.half;
        let head_at_start = self.prime || self.half;
        let heads = usize::from(head_at_end) + usize::from(head_at_start);

        // The shaft is shortened by one fifth, the heads keep their size.
        let total = length(&project_all(&points, self.proj));
        let shaft = (total - head_len * heads as f64).max(0.0);
        let mut path = cut_end(&points, self.proj, shaft * 0.1);
        path = reversed(&cut_end(&reversed(&path), self.proj, shaft * 0.1));
        let (tip_start, tip_end) = (path[0], path[path.len() - 1]);
        let mut head_list = Vec::new();
        if head_at_end {
            path = cut_end(&path, self.proj, head_len);
            head_list.push(Head {
                tip: tip_end,
                base: path[path.len() - 1],
            });
        }
        if head_at_start {
            path = reversed(&cut_end(&reversed(&path), self.proj, head_len));
            head_list.push(Head {
                tip: tip_start,
                base: path[0],
            });
        }
        // The round tail sits at the end without a head.
        let tail = match (head_at_start, head_at_end) {
            (true, false) => Some(path[path.len() - 1]),
            (false, true) => Some(path[0]),
            _ => None,
        };
        ArrowPlan {
            shaft: path,
            heads: head_list,
            tail,
        }
    }

    /// The `2x` of a half turn: beside the middle of the arc, on the side away
    /// from the cube center.
    fn half_mark(&self) -> Vec<Shape> {
        let middle = MIDDLE[self.axis].to_radians();
        let at = |angle: f64| self.proj.project(ring_point(self.axis, self.radius, angle));
        let (before, point, after) = (at(middle - 0.05), at(middle), at(middle + 0.05));
        let tangent = unit(sub(after, before));
        let mut outward = [tangent[1], -tangent[0]];
        let center = self.proj.project([0.0; 3]);
        let away = sub(point, center);
        if outward[0] * away[0] + outward[1] * away[1] < 0.0 {
            outward = [-outward[0], -outward[1]];
        }
        let shaft = self.metrics().width / 2.0 + halo_grow(self.pixels_per_unit);
        mark_beside(point, outward, shaft, BOTTOM)
    }

    fn arrow_items(&self) -> Vec<Item> {
        let plan = self.arrow_plan();
        let metrics = self.metrics();
        let grow = halo_grow(self.pixels_per_unit);
        let mut items = Vec::new();
        for run in self.runs(&plan.shaft, &|p| self.depth_of(p)) {
            let flat = project_all(&run.points, self.proj);
            items.push(Item {
                hidden: run.hidden,
                depth: run.depth,
                shapes: vec![
                    Shape::new(Part::Halo, ribbon(&flat, metrics.width + 2.0 * grow), WHITE),
                    Shape::new(Part::Arrow, ribbon(&flat, metrics.width), ARROW_VIOLET),
                ],
            });
        }
        for head in &plan.heads {
            let (base, tip) = (self.proj.project(head.base), self.proj.project(head.tip));
            let dir = unit(sub(tip, base));
            items.push(self.piece(
                head.tip,
                vec![
                    Shape::new(Part::Halo, arrowhead(base, dir, &metrics, grow), WHITE),
                    Shape::new(
                        Part::Arrow,
                        arrowhead(base, dir, &metrics, 0.0),
                        ARROW_VIOLET,
                    ),
                ],
            ));
        }
        if let Some(at) = plan.tail {
            let q = self.proj.project(at);
            items.push(self.piece(
                at,
                vec![
                    Shape::new(Part::Halo, disc(q, metrics.width / 2.0 + grow), WHITE),
                    Shape::new(Part::Arrow, disc(q, metrics.width / 2.0), ARROW_VIOLET),
                ],
            ));
        }
        items
    }

    /// A head or tail, classified at its sample `at`.
    fn piece(&self, at: P3, shapes: Vec<Shape>) -> Item {
        Item {
            hidden: self.hidden(at),
            depth: self.depth_of(at),
            shapes,
        }
    }
}

/// A run of samples that share a class.
struct Run {
    hidden: bool,
    depth: Depth,
    points: Vec<P3>,
}

impl Orbit<'_> {
    /// Splits a curve into maximal runs of equal class. Where the cube starts
    /// or stops to hide the curve, the border is found exactly, so the runs
    /// meet there; neighboring visible runs share a sample. A run needs two
    /// samples.
    fn runs(&self, points: &[P3], depth: &dyn Fn(P3) -> Depth) -> Vec<Run> {
        let points = self.with_borders(points);
        let class: Vec<(bool, Depth)> =
            points.iter().map(|&p| (self.hidden(p), depth(p))).collect();
        let mut runs = Vec::new();
        let mut i = 0;
        while i < points.len() {
            let mut j = i;
            while j + 1 < points.len() && class[j + 1] == class[i] {
                j += 1;
            }
            let visible = !class[i].0;
            let lo = if i > 0 && visible && !class[i - 1].0 {
                i - 1
            } else {
                i
            };
            let hi = if j + 1 < points.len() && visible && !class[j + 1].0 {
                j + 1
            } else {
                j
            };
            if hi > lo {
                runs.push(Run {
                    hidden: class[i].0,
                    depth: class[i].1,
                    points: points[lo..=hi].to_vec(),
                });
            }
            i = j + 1;
        }
        runs
    }

    /// `points` with two samples inserted, one on each side, wherever the cube
    /// starts or stops to hide the curve between two samples.
    fn with_borders(&self, points: &[P3]) -> Vec<P3> {
        let mut out = vec![points[0]];
        for pair in points.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if self.hidden(a) != self.hidden(b) {
                let (mut near_a, mut near_b) = (a, b);
                for _ in 0..40 {
                    let mid = [0, 1, 2].map(|k| 0.5 * (near_a[k] + near_b[k]));
                    if self.hidden(mid) == self.hidden(a) {
                        near_a = mid;
                    } else {
                        near_b = mid;
                    }
                }
                out.push(near_a);
                out.push(near_b);
            }
            out.push(b);
        }
        out
    }
}

/// The arrow before it is drawn: the shaft path, where the heads sit and the
/// round tail.
struct ArrowPlan {
    shaft: Vec<P3>,
    heads: Vec<Head>,
    tail: Option<P3>,
}

struct Head {
    tip: P3,
    base: P3,
}
