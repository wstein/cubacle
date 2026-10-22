//! Move icons: a small picture of a cube state and the move about to be made.
//!
//! The view is oblique (Front, Up and Right faces). The moving layers keep
//! their colours while the rest is darkened (or the moving layers turn grey,
//! see [`Shading`]); a red arrow with a white halo lies on the face where the
//! layer is visible, a "2" badge marks half turns, whole-cube rotations get a
//! curved cyan arrow, and the notation is written beside the cube.
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
mod font;
mod geometry;
mod kitty;
mod raster;
mod scene;
mod style;
mod svg;
mod text;

pub use action::{Action, Axis, RenderError};
pub use arrow::Direction;
pub use kitty::KittyPlacement;
pub use raster::Raster;
pub use style::{Palette, Rgb, Shading, Style};

use crate::puzzle::Facelets;
use scene::{Part, Shape, HEIGHT};

/// A drawn icon: shapes in a 150 x 100 unit canvas, ready for any writer.
#[derive(Debug, Clone, PartialEq)]
pub struct Icon {
    shapes: Vec<Shape>,
    label: String,
    size: u32,
    background: Option<Rgb>,
    grid: text::Grid,
    direction: Direction,
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
    let pixels_per_unit = f64::from(style.size.max(16)) / HEIGHT;
    let projection = geometry::Projection::new(n, 10.0, 9.0, 80.0, 76.0);
    let pixels_per_model = projection.scale * pixels_per_unit;
    let mut shapes = Vec::new();
    if style.shadow {
        let ellipse = (0..32)
            .map(|i| {
                let a = std::f64::consts::TAU * f64::from(i) / 32.0;
                [50.0 + 34.0 * a.cos(), 88.5 + 2.6 * a.sin()]
            })
            .collect();
        shapes.push(Shape::new(Part::Shadow, ellipse, style::BODY).with_opacity(0.3));
    }
    shapes.extend(geometry::cube_shapes(
        state,
        action,
        style,
        &projection,
        pixels_per_model,
    ));
    let plan = arrow::plan(action, n);
    shapes.extend(arrow::shapes(
        &plan,
        action,
        n,
        &projection,
        pixels_per_unit,
    ));
    let label = action.to_string();
    shapes.extend(label_shapes(&label, pixels_per_unit));
    Ok(Icon {
        grid: text::grid(state, action, style, plan.direction),
        direction: plan.direction,
        shapes,
        label,
        size: style.size,
        background: style.background,
    })
}

/// The notation beside the cube: light with a dark outline, readable on any backdrop.
fn label_shapes(label: &str, pixels_per_unit: f64) -> Vec<Shape> {
    let units = font::measure(label);
    let mut scale = 26.0 / font::GLYPH_HEIGHT;
    if units * scale > 36.0 {
        scale = 36.0 / units;
    }
    let height = scale * font::GLYPH_HEIGHT;
    let stroke = (0.19 * scale).max(2.0 / pixels_per_unit);
    let (x, y) = (104.0, 47.0 - height / 2.0);
    let outline = stroke + (3.0 / pixels_per_unit).clamp(1.2, 2.0);
    let mut shapes = font::text(label, x, y, height, outline, style::BODY, Part::Label);
    shapes.extend(font::text(
        label,
        x,
        y,
        height,
        stroke,
        Rgb(0xf4, 0xf4, 0xf4),
        Part::Label,
    ));
    shapes
}

impl Icon {
    /// The notation shown beside the cube, e.g. `Rw2` or `2R2`.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Which way the arrow points on the screen.
    pub fn direction(&self) -> Direction {
        self.direction
    }

    /// A self-contained SVG document, identical on every run.
    pub fn to_svg(&self) -> String {
        svg::write(&self.shapes, &self.label, self.size, self.background)
    }

    /// Anti-aliased RGBA pixels, `height` pixels tall and 1.5 times as wide.
    pub fn to_rgba(&self, height: u32) -> Raster {
        raster::rasterise(&self.shapes, height, self.background)
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
