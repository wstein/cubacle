//! Move icons: a small picture of a cube state and the move about to be made.
//!
//! The view is a deep oblique one (Front, Up and Right faces) of flat bright
//! stickers. The moving layers keep their colours while the rest is darkened
//! (or the moving layers turn grey, see [`Shading`]); a soft violet arrow with
//! a white outline lies on the face where the layer is visible, an upright
//! violet "2x" marks half turns, whole-cube rotations get a curved arrow, and
//! the notation is written below the cube in Quicksand Bold. [`state_icon`]
//! draws a state alone, framed, for the result of a group of moves. The [`Backdrop`]
//! decides how strongly the resting pieces are dimmed and the colour of the
//! notation.
//!
//! Everything is a pure function of the state, the [`Action`] and the
//! [`Style`], for any cube size from 2 to 7. The result is a list of filled
//! polygons ([`Icon`]) that is written as SVG ([`Icon::to_svg`]), rasterised
//! ([`Icon::to_rgba`]), encoded for the Kitty graphics protocol
//! ([`Raster::to_kitty`]) or drawn with letters ([`Icon::to_text`]). The module
//! depends only on `puzzle`, does no I/O and keeps no state, so it can become a
//! crate of its own.
//!
//! ```
//! use cubacle::puzzle::{Facelets, PuzzleKind};
//! use cubacle::render::{icon, Action, Style};
//!
//! let state = Facelets::solved(PuzzleKind::Cube3);
//! let action: Action = "R".parse().unwrap();
//! let svg = icon(&state, &action, &Style::default()).unwrap().to_svg();
//! assert!(svg.starts_with("<svg"));
//! ```

mod action;
mod arrow;
mod brackets;
mod font;
mod geometry;
mod glyphs;
mod kitty;
mod label;
mod net;
mod pair;
mod raster;
mod round;
mod scene;
mod state;
mod style;
mod svg;
mod text;

pub use action::{Action, Axis, RenderError};
pub use arrow::Direction;
pub use brackets::{bracket_height, brackets, BlockSpan, UnitSpan};
pub use kitty::KittyPlacement;
pub use net::{state_icon_net, state_icon_net_against};
pub use pair::{opposite_view, state_icon_pair, state_icon_pair_against};
pub use raster::Raster;
pub use style::{Backdrop, Palette, Rgb, Shading, Style};

use crate::puzzle::Facelets;
use geometry::Camera;
use scene::Shape;

/// A drawn icon: shapes in a 114 x 109.2 unit canvas, ready for any writer.
#[derive(Debug, Clone, PartialEq)]
pub struct Icon {
    shapes: Vec<Shape>,
    label: String,
    size: u32,
    background: Option<Rgb>,
    canvas: (f64, f64),
    grid: text::Grid,
    direction: Option<Direction>,
}

/// Draws `action` on `state`.
///
/// Fails when the action turns a layer the cube does not have.
pub fn icon(state: &Facelets, action: &Action, style: &Style) -> Result<Icon, RenderError> {
    let kind = state.kind();
    if !action.fits(kind) {
        return Err(RenderError {
            action: *action,
            kind,
        });
    }
    let n = kind.size();
    let label = action.to_string();
    let pixels_per_unit = f64::from(style.size.max(16)) / scene::HEIGHT;
    let projection = geometry::Projection::new(n, 10.0, 9.0, 80.0, 76.0);
    let pixels_per_model = projection.scale * pixels_per_unit;
    let mut shapes = geometry::cube_shapes(
        state,
        Some(action),
        style,
        Camera::Above,
        &projection,
        pixels_per_model,
    );
    let plan = arrow::plan(action, n);
    shapes.extend(arrow::shapes(
        &plan,
        action,
        n,
        &projection,
        pixels_per_unit,
    ));
    shapes.extend(label::shapes(&label, style.backdrop));
    Ok(Icon {
        grid: text::grid(state, action, style, plan.direction),
        direction: Some(plan.direction),
        shapes,
        label,
        size: style.size,
        background: style.background,
        canvas: (scene::WIDTH, scene::HEIGHT),
    })
}

/// Draws the cube `state` and nothing else: no arrow, no notation, no dimming,
/// every sticker in full colour, inside a thin frame so that it cannot be
/// mistaken for a move. It has the size of a move icon, so the two sit in one
/// row. [`Icon::label`] is `"state"` and [`Icon::direction`] is `None`.
pub fn state_icon(state: &Facelets, style: &Style) -> Icon {
    state_icon_seen(state, style, Camera::Above)
}

/// [`state_icon`] from above (U, F, R) or from below (D, F, R) the cube.
fn state_icon_seen(state: &Facelets, style: &Style, camera: Camera) -> Icon {
    let n = state.kind().size();
    let pixels_per_unit = f64::from(style.size.max(16)) / scene::HEIGHT;
    let projection = geometry::Projection::seen_from(camera, n, 10.0, 9.0, 80.0, 76.0);
    let pixels_per_model = projection.scale * pixels_per_unit;
    let mut shapes =
        geometry::cube_shapes(state, None, style, camera, &projection, pixels_per_model);
    shapes.extend(state::frame(style.backdrop));
    Icon {
        grid: text::state_grid(state, style, camera),
        direction: None,
        shapes,
        label: "state".to_owned(),
        size: style.size,
        background: style.background,
        canvas: (scene::WIDTH, scene::HEIGHT),
    }
}

/// How many stickers of `state` differ in colour from `reference`.
pub fn differing_stickers(state: &Facelets, reference: &Facelets) -> usize {
    state
        .stickers()
        .iter()
        .zip(reference.stickers())
        .filter(|(a, b)| a != b)
        .count()
}

/// Like [`state_icon`], with a small amber square on every visible sticker
/// whose colour differs from `reference`. Stickers on the three faces that are
/// out of sight cannot be marked; [`differing_stickers`] counts them all.
pub fn state_icon_against(state: &Facelets, reference: &Facelets, style: &Style) -> Icon {
    state_icon_against_seen(state, reference, style, Camera::Above)
}

fn state_icon_against_seen(
    state: &Facelets,
    reference: &Facelets,
    style: &Style,
    camera: Camera,
) -> Icon {
    let mut icon = state_icon_seen(state, style, camera);
    let n = state.kind().size();
    let projection = geometry::Projection::seen_from(camera, n, 10.0, 9.0, 80.0, 76.0);
    let marks = geometry::difference_marks(state, reference, camera, &projection);
    // the marks go under the frame, over the stickers
    let at = icon
        .shapes
        .iter()
        .position(|s| s.part == scene::Part::Frame);
    let at = at.unwrap_or(icon.shapes.len());
    icon.shapes.splice(at..at, marks);
    icon
}

impl Icon {
    /// The notation shown beside the cube, e.g. `Rw2` or `2R2`.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Which way the arrow points on the screen; `None` for a [`state_icon`].
    pub fn direction(&self) -> Option<Direction> {
        self.direction
    }

    /// A self-contained SVG document, identical on every run.
    pub fn to_svg(&self) -> String {
        svg::write(
            &self.shapes,
            &self.label,
            self.size,
            self.background,
            self.canvas,
        )
    }

    /// Anti-aliased RGBA pixels, `height` pixels tall and as wide as the canvas is (about 1.06 times: the notation sits below the cube).
    pub fn to_rgba(&self, height: u32) -> Raster {
        raster::rasterise_canvas(&self.shapes, self.canvas, height, self.background)
    }

    /// The icon as letters; with `ansi` as coloured cells. See the module `text`.
    pub fn to_text(&self, ansi: bool) -> String {
        self.grid.render(ansi)
    }
}

impl Raster {
    /// Kitty graphics escapes that display the pixels where the cursor is.
    pub fn to_kitty(&self, placement: &KittyPlacement) -> String {
        kitty::encode(self, placement)
    }
}

#[cfg(test)]
mod mark_tests {
    use super::*;
    use crate::puzzle::{Facelets, PuzzleKind};
    use scene::Part;

    fn marks(icon: &Icon) -> usize {
        icon.shapes.iter().filter(|s| s.part == Part::Mark).count()
    }

    #[test]
    fn equal_cubes_have_no_marks() {
        for kind in [PuzzleKind::Cube3, PuzzleKind::Cube7] {
            let cube = Facelets::solved(kind);
            let icon = state_icon_against(&cube, &cube, &Style::default());
            assert_eq!(marks(&icon), 0);
            assert_eq!(icon, state_icon(&cube, &Style::default()));
            assert_eq!(differing_stickers(&cube, &cube), 0);
        }
    }

    #[test]
    fn a_turned_layer_marks_the_stickers_that_changed() {
        let solved = Facelets::solved(PuzzleKind::Cube3);
        let turned = Action::Layer("R".parse().unwrap()).apply(&solved).unwrap();
        let icon = state_icon_against(&turned, &solved, &Style::default());
        let count = differing_stickers(&turned, &solved);
        assert_eq!(count, 12);
        // R changes 3 stickers on each of U, F, D, B; D and B are out of sight
        assert_eq!(marks(&icon), 6);
    }
}
