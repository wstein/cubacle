//! The drawing as a list of filled polygons in a 150 x 100 unit canvas.

use super::style::Rgb;

pub(crate) const WIDTH: f64 = 150.0;
pub(crate) const HEIGHT: f64 = 100.0;

/// What a polygon is, so tests and writers can tell them apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Part {
    Shadow,
    Body,
    Sticker,
    Highlight,
    Halo,
    Arrow,
    Badge,
    Label,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Shape {
    pub part: Part,
    pub points: Vec<[f64; 2]>,
    pub fill: Rgb,
    pub opacity: f64,
}

impl Shape {
    pub(crate) fn new(part: Part, points: Vec<[f64; 2]>, fill: Rgb) -> Shape {
        Shape {
            part,
            points,
            fill,
            opacity: 1.0,
        }
    }

    pub(crate) fn with_opacity(mut self, opacity: f64) -> Shape {
        self.opacity = opacity;
        self
    }
}

/// A regular polygon approximating a disc.
pub(crate) fn disc(centre: [f64; 2], radius: f64) -> Vec<[f64; 2]> {
    (0..24)
        .map(|i| {
            let angle = std::f64::consts::TAU * f64::from(i) / 24.0;
            [
                centre[0] + radius * angle.cos(),
                centre[1] + radius * angle.sin(),
            ]
        })
        .collect()
}
