use super::curve::{in_front_of_axis, length, occluded, project_all, ring_point, P2, P3};
use super::*;
use crate::puzzle::{Face, Facelets, PuzzleKind};
use crate::render::geometry::{cube_shapes, Camera};
use crate::render::scene::HEIGHT;
use crate::render::style::Style;
use crate::render::{icon, Backdrop, Icon};

const ROTATIONS: [&str; 9] = ["x", "y", "z", "x2", "y2", "z2", "x'", "y'", "z'"];
const KINDS: [PuzzleKind; 6] = [
    PuzzleKind::Cube2,
    PuzzleKind::Cube3,
    PuzzleKind::Cube4,
    PuzzleKind::Cube5,
    PuzzleKind::Cube6,
    PuzzleKind::Cube7,
];

fn act(text: &str) -> Action {
    text.parse().unwrap()
}

fn projection(n: usize) -> Projection {
    Projection::new(n, 10.0, 9.0, 80.0, 76.0)
}

fn pixels_per_unit() -> f64 {
    f64::from(Style::default().size.max(16)) / HEIGHT
}

/// The face a direction points to, for the axis directions of the model.
fn face_of(v: P3) -> Face {
    let k = (0..3)
        .max_by(|&a, &b| v[a].abs().total_cmp(&v[b].abs()))
        .unwrap();
    match (k, v[k] > 0.0) {
        (0, true) => Face::R,
        (0, false) => Face::L,
        (1, true) => Face::U,
        (1, false) => Face::D,
        (_, true) => Face::F,
        (_, false) => Face::B,
    }
}

fn center(kind: PuzzleKind, face: Face) -> usize {
    face as usize * kind.face_len() + kind.face_len() / 2
}

fn sub3(a: P3, b: P3) -> P3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: P3, b: P3) -> f64 {
    (0..3).map(|k| a[k] * b[k]).sum()
}

/// The arrow runs the way `apply()` moves the stickers: from the tail to the
/// head it goes from the face at orbit angle 0 towards the face at 90 degrees
/// when that sticker moves there, and the other way round otherwise.
#[test]
fn arrows_run_the_way_apply_moves_the_stickers() {
    let kind = PuzzleKind::Cube3;
    let proj = projection(3);
    for text in ["x", "x'", "y", "y'", "z", "z'"] {
        let action = act(text);
        let (axis, _) = action.axis_sign();
        let orbit = Orbit::new(&action, 3, &proj, pixels_per_unit(), Backdrop::default());
        let plan = orbit.arrow_plan();
        let head = &plan.heads[0];
        let tail = plan.tail.expect("a quarter turn has a round tail");
        let travel = sub3(head.tip, tail);

        // the two faces 45 degrees to either side of the middle of the arc
        let middle = MIDDLE[axis].to_radians();
        let eighth = std::f64::consts::FRAC_PI_4;
        let (from, to) = (
            ring_point(axis, 1.0, middle - eighth),
            ring_point(axis, 1.0, middle + eighth),
        );
        let (a, b) = (face_of(from), face_of(to));
        let after = action.apply(&Facelets::solved(kind)).unwrap();
        let a_to_b = after.stickers()[center(kind, b)] == a;
        let b_to_a = after.stickers()[center(kind, a)] == b;
        assert!(a_to_b != b_to_a, "{text}: {a:?} {b:?}");
        let along = dot3(travel, sub3(to, from));
        assert!(
            if a_to_b { along > 0.0 } else { along < 0.0 },
            "{text}: stickers {a:?} -> {b:?} {a_to_b}, arrow {along}"
        );
    }
}

/// A prime has its head at the start of the arc and a round tail at the end; a
/// half turn has two heads and no tail.
#[test]
fn heads_and_tails_follow_the_turn() {
    let proj = projection(5);
    for text in ROTATIONS {
        let orbit = Orbit::new(&act(text), 5, &proj, pixels_per_unit(), Backdrop::default());
        let plan = orbit.arrow_plan();
        let (start, end) = orbit.arc();
        let at = |angle: f64| ring_point(orbit.axis, orbit.radius, angle);
        let gap = |p: P3, q: P3| dot3(sub3(p, q), sub3(p, q)).sqrt();
        let nearer_to_end = |p: P3| gap(p, at(end)) < gap(p, at(start));
        if text.ends_with('2') {
            assert_eq!(plan.heads.len(), 2, "{text}");
            assert!(plan.tail.is_none(), "{text}");
            continue;
        }
        assert_eq!(plan.heads.len(), 1, "{text}");
        let tail = plan.tail.expect("a round tail");
        let prime = text.ends_with('\'');
        assert_eq!(nearer_to_end(plan.heads[0].tip), !prime, "{text}: head");
        assert_eq!(nearer_to_end(tail), prime, "{text}: tail");
    }
}

#[test]
fn a_point_behind_the_cube_is_hidden_and_one_in_front_is_not() {
    let m = 5.1;
    assert!(occluded([0.0, 0.0, -9.0], m));
    assert!(!occluded([0.0, 0.0, 9.0], m));
    // beside the cube, and above it, the ray towards the viewer misses it
    assert!(!occluded([12.0, 0.0, 0.0], m));
    assert!(!occluded([0.0, 12.0, 0.0], m));
    // inside the cube
    assert!(occluded([1.0, 1.0, 1.0], m));
}

/// The half of a circle on the viewer's side is in front of the axis.
#[test]
fn the_near_half_of_the_orbit_is_in_front_of_the_axis() {
    let r = 7.5;
    for axis in 0..3 {
        // angle 0 is the Front for `x` and `y`, the Right for `z`: nearer
        assert!(in_front_of_axis(axis, ring_point(axis, r, 0.0)), "{axis}");
        assert!(
            !in_front_of_axis(axis, ring_point(axis, r, std::f64::consts::PI)),
            "{axis}"
        );
    }
}

/// Behind the axis, then the axis, then in front of it, in both lists.
#[test]
fn the_back_of_the_orbit_is_painted_before_the_axis_and_the_front_after_it() {
    let proj = projection(5);
    for text in ROTATIONS {
        let orbit = Orbit::new(&act(text), 5, &proj, pixels_per_unit(), Backdrop::default());
        let layers = shapes(&act(text), 5, &proj, pixels_per_unit(), Backdrop::default());
        let rank = |list: &[Shape], p: P3| {
            let q = proj.project(p);
            list.iter().position(|s| {
                s.fill == Backdrop::default().ring_ink()
                    && s.points
                        .iter()
                        .any(|v| (v[0] - q[0]).hypot(v[1] - q[1]) < 0.8)
            })
        };
        let axis_at = |list: &[Shape]| -> (usize, usize) {
            let at: Vec<usize> = (0..list.len())
                .filter(|&i| list[i].fill == Backdrop::default().axis_ink())
                .collect();
            (at[0], at[at.len() - 1])
        };
        let (first, last) = axis_at(&layers.front);
        // a visible sample of the back and of the front of the orbit
        let samples: Vec<P3> = (0..RING_SAMPLES)
            .map(|i| {
                let angle = std::f64::consts::TAU * i as f64 / RING_SAMPLES as f64;
                ring_point(orbit.axis, orbit.radius, angle)
            })
            .filter(|&p| !orbit.hidden(p))
            .collect();
        let back = samples
            .iter()
            .copied()
            .find(|&p| orbit.depth_of(p) == Depth::Behind)
            .unwrap();
        let front = samples
            .iter()
            .copied()
            .find(|&p| orbit.depth_of(p) == Depth::Front)
            .unwrap();
        assert!(rank(&layers.front, back).unwrap() < first, "{text}");
        assert!(rank(&layers.front, front).unwrap() > last, "{text}");
    }
}

fn arc_length(text: &str, n: usize) -> f64 {
    let proj = projection(n);
    let orbit = Orbit::new(&act(text), n, &proj, pixels_per_unit(), Backdrop::default());
    let (start, end) = orbit.arc();
    let points: Vec<P3> = (0..=400)
        .map(|i| {
            let angle = start + (end - start) * f64::from(i) / 400.0;
            ring_point(orbit.axis, orbit.radius, angle)
        })
        .collect();
    length(&project_all(&points, &proj))
}

/// The arcs of `x`, `y` and `z` are about 74 canvas units long on the screen,
/// the arcs of the half turns about 90, whatever the cube size.
#[test]
fn arcs_have_the_same_length_on_the_screen() {
    for n in 2..=7 {
        for (text, want) in [
            ("x", 74.0),
            ("y", 74.0),
            ("z", 74.0),
            ("x2", 90.0),
            ("y2", 90.0),
            ("z2", 90.0),
        ] {
            let got = arc_length(text, n);
            assert!(
                (got - want).abs() <= 0.03 * want,
                "{text} on {n}x{n}: {got:.2} against {want}"
            );
        }
    }
}

/// The full spans: a quarter turn is 110 degrees on `x` and `y` and 105 on
/// `z`, a half turn 146.4 and 127.5.
#[test]
fn spans_are_the_decided_angles() {
    let proj = projection(5);
    for (text, degrees, middle) in [
        ("x", 110.0, 45.0),
        ("y", 110.0, -45.0),
        ("z", 105.0, -45.0),
        ("x2", 146.4, 45.0),
        ("y2", 146.4, -45.0),
        ("z2", 127.5, -45.0),
    ] {
        let orbit = Orbit::new(&act(text), 5, &proj, pixels_per_unit(), Backdrop::default());
        let (start, end) = orbit.arc();
        assert!((end - start).to_degrees() - degrees < 1e-9, "{text}");
        assert!(
            ((end - start).to_degrees() - degrees).abs() < 1e-9,
            "{text}"
        );
        let center = 0.5 * (start + end);
        assert!((center.to_degrees() - middle).abs() < 1e-9, "{text}");
    }
}

fn smallest_edge_distance(icon: &Icon) -> (f64, f64) {
    let (mut edge, mut low) = (f64::INFINITY, 0.0_f64);
    for s in icon
        .shapes
        .iter()
        .filter(|s| matches!(s.part, Part::Arrow | Part::Halo | Part::Mark))
    {
        for &[x, y] in &s.points {
            edge = edge.min(x).min(WIDTH - x).min(y);
            low = low.max(y);
        }
    }
    (edge, low)
}

/// Every polygon of the orbit, the axis, the arrow and the mark, halos
/// included, stays 1 unit from the canvas edge and above the notation.
#[test]
fn rotation_icons_stay_in_the_drawing_area() {
    for kind in KINDS {
        for text in ROTATIONS {
            let drawn = icon(&Facelets::solved(kind), &act(text), &Style::default()).unwrap();
            let (edge, low) = smallest_edge_distance(&drawn);
            assert!(edge >= MARGIN, "{kind:?} {text}: edge distance {edge:.3}");
            assert!(low <= BOTTOM, "{kind:?} {text}: lowest point {low:.3}");
        }
    }
}

fn body(icon: &Icon) -> Vec<Vec<P2>> {
    let bodies = icon.shapes.iter().filter(|s| s.part == Part::Body);
    bodies.map(|s| s.points.clone()).collect()
}

/// The cube of a rotation icon is that of a regular move icon: same canvas,
/// same projection, same polygons.
#[test]
fn rotation_icons_use_the_raster_of_regular_moves() {
    let style = Style::default();
    let state = Facelets::solved(PuzzleKind::Cube5);
    let regular = icon(&state, &act("R"), &style).unwrap();
    let proj = projection(5);
    let ppu = pixels_per_unit();
    let drawn = cube_shapes(&state, None, &style, Camera::Above, &proj, proj.scale * ppu);
    let cube: Vec<Vec<P2>> = drawn
        .iter()
        .filter(|s| s.part == Part::Body)
        .map(|s| s.points.clone())
        .collect();
    for text in ROTATIONS {
        let rotation = icon(&state, &act(text), &style).unwrap();
        assert_eq!(body(&regular), body(&rotation), "{text}");
        assert_eq!(body(&rotation), cube, "{text}");
        assert_eq!(regular.canvas, rotation.canvas, "{text}");
    }
}

/// The hidden part is drawn before the cube body, the rest after it, and the
/// two parts together are the whole axis.
#[test]
fn the_axis_runs_through_the_cube() {
    let proj = projection(5);
    let ppu = pixels_per_unit();
    for text in ["x", "y", "z"] {
        let layers = shapes(&act(text), 5, &proj, ppu, Backdrop::default());
        let axis = |list: &[Shape]| {
            list.iter()
                .filter(|s| s.fill == Backdrop::default().axis_ink())
                .count()
        };
        assert!(axis(&layers.behind) > 0, "{text}: no axis inside the cube");
        // `z` points at the viewer: only its front end is visible
        let outside = if text == "z" { 1 } else { 2 };
        assert!(axis(&layers.front) >= outside, "{text}: axis ends outside");
    }
}

/// Measurements for the report: `cargo test ... -- --nocapture orbit_measure`.
#[test]
#[ignore = "prints measurements"]
fn orbit_measure() {
    for kind in KINDS {
        let mut line = format!("{}x{}:", kind.size(), kind.size());
        for text in ROTATIONS {
            let drawn = icon(&Facelets::solved(kind), &act(text), &Style::default()).unwrap();
            let (edge, low) = smallest_edge_distance(&drawn);
            line += &format!(" {text} {edge:.2}/{low:.2}");
        }
        println!("{line}");
    }
    for text in ["x", "y", "z", "x2", "y2", "z2"] {
        println!("{text} 5x5 {:.2}", arc_length(text, 5));
    }
}

/// The box around the glyphs of the "2x" mark.
fn mark_box(icon: &Icon) -> [f64; 4] {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for s in icon.shapes.iter().filter(|s| s.part == Part::Mark) {
        for &[x, y] in &s.points {
            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
        }
    }
    [x0, y0, x1, y1]
}

/// The "2x" of a half turn sits beside the arrow: its box touches the side of
/// the shaft with `MARK_GAP` of air, so the box center is at most the half shaft
/// with its halo, plus the gap, plus half the box diagonal from the shaft. One
/// unit more covers the spacing of the shaft samples. The largest distance
/// measured is 18.7 against a bound of 20.1; with the mark in the corner, as
/// before, the distance is 59 or more.
#[test]
fn the_half_turn_mark_sits_beside_the_arrow() {
    for kind in KINDS {
        let n = kind.size();
        for backdrop in [Backdrop::Dark, Backdrop::Light] {
            let style = Style {
                backdrop,
                ..Style::default()
            };
            for text in ["x2", "y2", "z2"] {
                let drawn = icon(&Facelets::solved(kind), &act(text), &style).unwrap();
                let [x0, y0, x1, y1] = mark_box(&drawn);
                let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
                let proj = projection(n);
                let orbit =
                    Orbit::new(&act(text), n, &proj, pixels_per_unit(), Backdrop::default());
                let shaft = project_all(&orbit.arrow_plan().shaft, &proj);
                let gap = shaft
                    .iter()
                    .map(|q| (q[0] - cx).hypot(q[1] - cy))
                    .fold(f64::MAX, f64::min);
                let half = orbit.metrics().width / 2.0 + halo_grow(pixels_per_unit());
                let bound = half + 3.0 + (x1 - x0).hypot(y1 - y0) / 2.0 + 1.0;
                assert!(
                    gap <= bound,
                    "{n}x{n} {text} {backdrop:?}: mark center {gap:.1} from the shaft, bound {bound:.1}"
                );
            }
        }
    }
}

/// The axis is drawn in the color of the backdrop it is meant for.
#[test]
fn the_axis_takes_its_color_from_the_backdrop() {
    for backdrop in [Backdrop::Dark, Backdrop::Light] {
        let proj = projection(5);
        let layers = shapes(&act("x"), 5, &proj, pixels_per_unit(), backdrop);
        let has = |ink: Rgb| layers.front.iter().any(|s| s.fill == ink);
        assert!(has(backdrop.axis_ink()), "{backdrop:?}: axis");
        assert!(has(backdrop.ring_ink()), "{backdrop:?}: ring");
    }
}
