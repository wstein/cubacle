//! The drawing as a list of filled polygons in a 114 x 109.2 unit canvas: the
//! cube above, the notation below it.

use super::style::Rgb;

pub(crate) const WIDTH: f64 = 114.0;
pub(crate) const HEIGHT: f64 = 109.2;

/// What a polygon is, so tests and writers can tell them apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Part {
    Body,
    Sticker,
    Halo,
    Arrow,
    Mark,
    Label,
    Number,
    Frame,
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

    #[cfg(test)]
    pub(crate) fn with_opacity(mut self, opacity: f64) -> Shape {
        self.opacity = opacity;
        self
    }
}
