//! A small anti-aliased polygon filler: exact horizontal coverage, four
//! sub-scanlines per pixel, even-odd fill, premultiplied accumulation.

use super::scene::Shape;
use super::style::Rgb;

/// Straight (non-premultiplied) RGBA pixels, row by row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raster {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

const SUB: usize = 4;

/// The test canvas: 150 x 100 units, so units equal pixels at 100 px.
#[cfg(test)]
pub(crate) fn rasterise(shapes: &[Shape], height: u32, background: Option<Rgb>) -> Raster {
    rasterise_canvas(shapes, (150.0, 100.0), height, background)
}

pub(crate) fn rasterise_canvas(
    shapes: &[Shape],
    (canvas_width, canvas_height): (f64, f64),
    height: u32,
    background: Option<Rgb>,
) -> Raster {
    let k = f64::from(height) / canvas_height;
    let width = (canvas_width * k).round() as u32;
    let (w, h) = (width as usize, height as usize);
    let mut buf = vec![0.0_f32; w * h * 4];
    if let Some(Rgb(r, g, b)) = background {
        for px in buf.chunks_exact_mut(4) {
            px.copy_from_slice(&[
                f32::from(r) / 255.0,
                f32::from(g) / 255.0,
                f32::from(b) / 255.0,
                1.0,
            ]);
        }
    }
    for shape in shapes {
        let points: Vec<[f64; 2]> = shape.points.iter().map(|p| [p[0] * k, p[1] * k]).collect();
        fill(&mut buf, w, h, &points, shape.fill, shape.opacity as f32);
    }
    let mut data = Vec::with_capacity(w * h * 4);
    for px in buf.chunks_exact(4) {
        let a = px[3];
        let channel = |v: f32| {
            let straight = if a > 0.0 { v / a } else { 0.0 };
            (straight * 255.0).round().clamp(0.0, 255.0) as u8
        };
        data.extend_from_slice(&[
            channel(px[0]),
            channel(px[1]),
            channel(px[2]),
            (a * 255.0).round().clamp(0.0, 255.0) as u8,
        ]);
    }
    Raster {
        width,
        height,
        data,
    }
}

impl Raster {
    /// Draws a frame of `colour` along the border of the raster, over what is
    /// there. The thickness is 3 px at 109 px height and grows with the height.
    pub fn framed(mut self, colour: Rgb) -> Raster {
        let (w, h) = (self.width as usize, self.height as usize);
        let t = (h / 36).max(2).min(w / 2).min(h / 2);
        let px = [colour.0, colour.1, colour.2, 255];
        for y in 0..h {
            for x in 0..w {
                if x < t || y < t || x >= w - t || y >= h - t {
                    self.data[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&px);
                }
            }
        }
        self
    }

    /// Places `rasters` side by side, top-aligned, with `gap` transparent
    /// pixels between neighbours. The result is as tall as the tallest one;
    /// where a raster is shorter, the rest of its column is transparent.
    /// An empty list gives an empty raster.
    pub fn hstack(rasters: &[Raster], gap: u32) -> Raster {
        let height = rasters.iter().map(|r| r.height).max().unwrap_or(0);
        let gaps = u32::try_from(rasters.len().saturating_sub(1)).unwrap_or(u32::MAX);
        let width = rasters.iter().map(|r| r.width).sum::<u32>() + gap * gaps;
        let (w, h) = (width as usize, height as usize);
        let mut data = vec![0u8; w * h * 4];
        let mut x = 0usize;
        for raster in rasters {
            let rw = raster.width as usize;
            for (y, row) in raster.data.chunks_exact(rw * 4).enumerate() {
                let at = (y * w + x) * 4;
                data[at..at + rw * 4].copy_from_slice(row);
            }
            x += rw + gap as usize;
        }
        Raster {
            width,
            height,
            data,
        }
    }
}

fn fill(buf: &mut [f32], w: usize, h: usize, points: &[[f64; 2]], colour: Rgb, opacity: f32) {
    if points.len() < 3 {
        return;
    }
    let min_y = points
        .iter()
        .map(|p| p[1])
        .fold(f64::MAX, f64::min)
        .floor()
        .max(0.0) as usize;
    let max_y = (points
        .iter()
        .map(|p| p[1])
        .fold(f64::MIN, f64::max)
        .ceil()
        .max(0.0) as usize)
        .min(h);
    let rgb = [colour.0, colour.1, colour.2].map(|c| f32::from(c) / 255.0);
    let mut coverage = vec![0.0_f32; w];
    let mut crossings: Vec<f64> = Vec::new();
    for y in min_y..max_y {
        coverage.iter_mut().for_each(|c| *c = 0.0);
        for sub in 0..SUB {
            let yy = y as f64 + (sub as f64 + 0.5) / SUB as f64;
            crossings.clear();
            for i in 0..points.len() {
                let (a, b) = (points[i], points[(i + 1) % points.len()]);
                if (a[1] <= yy) != (b[1] <= yy) {
                    crossings.push(a[0] + (yy - a[1]) / (b[1] - a[1]) * (b[0] - a[0]));
                }
            }
            crossings.sort_by(f64::total_cmp);
            for pair in crossings.chunks_exact(2) {
                let (x0, x1) = (pair[0].max(0.0), pair[1].min(w as f64));
                if x1 <= x0 {
                    continue;
                }
                let (first, last) = (x0.floor() as usize, (x1.floor() as usize).min(w - 1));
                for (px, slot) in coverage.iter_mut().enumerate().take(last + 1).skip(first) {
                    let overlap = x1.min(px as f64 + 1.0) - x0.max(px as f64);
                    if overlap > 0.0 {
                        *slot += (overlap / SUB as f64) as f32;
                    }
                }
            }
        }
        for (x, &c) in coverage.iter().enumerate() {
            if c <= 0.0 {
                continue;
            }
            let a = c.min(1.0) * opacity;
            let px = &mut buf[(y * w + x) * 4..][..4];
            for i in 0..3 {
                px[i] = rgb[i] * a + px[i] * (1.0 - a);
            }
            px[3] = a + px[3] * (1.0 - a);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::scene::Part;

    fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Shape {
        Shape::new(
            Part::Sticker,
            vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]],
            Rgb(255, 0, 0),
        )
    }

    /// Scene units equal pixels when the canvas is 100 px tall (150 x 100).
    fn pixel(raster: &Raster, x: usize, y: usize) -> [u8; 4] {
        let at = (y * raster.width as usize + x) * 4;
        raster.data[at..at + 4].try_into().unwrap()
    }

    #[test]
    fn a_rectangle_covers_the_expected_pixels() {
        let raster = rasterise(&[rect(10.0, 20.0, 30.5, 40.0)], 100, None);
        assert_eq!((raster.width, raster.height), (150, 100));
        assert_eq!(pixel(&raster, 10, 20), [255, 0, 0, 255]);
        assert_eq!(pixel(&raster, 29, 39), [255, 0, 0, 255]);
        assert_eq!(pixel(&raster, 30, 30), [255, 0, 0, 128]);
        assert_eq!(pixel(&raster, 31, 30)[3], 0);
        assert_eq!(pixel(&raster, 9, 30)[3], 0);
        assert_eq!(pixel(&raster, 20, 40)[3], 0);
        assert_eq!(pixel(&raster, 20, 19)[3], 0);
    }

    #[test]
    fn a_diagonal_edge_is_anti_aliased() {
        let triangle = Shape::new(
            Part::Sticker,
            vec![[0.0, 0.0], [10.0, 0.0], [0.0, 10.0]],
            Rgb(0, 0, 255),
        );
        let raster = rasterise(&[triangle], 100, None);
        let alpha: Vec<u8> = (0..10).map(|i| pixel(&raster, i, 9 - i)[3]).collect();
        assert!(alpha.iter().all(|&a| a > 100 && a < 155), "{alpha:?}");
        assert_eq!(pixel(&raster, 0, 0)[3], 255);
        assert_eq!(pixel(&raster, 9, 9)[3], 0);
    }

    #[test]
    fn opacity_and_background_blend() {
        let half = rect(0.0, 0.0, 150.0, 100.0).with_opacity(0.5);
        let raster = rasterise(&[half], 100, Some(Rgb(0, 0, 0)));
        assert_eq!(pixel(&raster, 5, 5), [128, 0, 0, 255]);
        let transparent = rasterise(&[rect(0.0, 0.0, 150.0, 100.0).with_opacity(0.5)], 100, None);
        assert_eq!(pixel(&transparent, 5, 5), [255, 0, 0, 128]);
    }

    #[test]
    fn output_scales_with_the_height() {
        let raster = rasterise(&[rect(0.0, 0.0, 75.0, 50.0)], 50, None);
        assert_eq!((raster.width, raster.height), (75, 50));
        assert_eq!(pixel(&raster, 36, 24)[3], 255);
        assert_eq!(pixel(&raster, 37, 24)[3], 128);
        assert_eq!(pixel(&raster, 38, 24)[3], 0);
    }

    #[test]
    fn hstack_joins_with_transparent_gaps() {
        let one = |v: u8, w: u32, h: u32| Raster {
            width: w,
            height: h,
            data: vec![v; (w * h * 4) as usize],
        };
        let joined = Raster::hstack(&[one(1, 2, 2), one(2, 1, 3)], 3);
        assert_eq!((joined.width, joined.height), (2 + 3 + 1, 3));
        assert_eq!(pixel(&joined, 1, 1), [1; 4]);
        assert_eq!(pixel(&joined, 2, 1), [0; 4]);
        assert_eq!(pixel(&joined, 5, 2), [2; 4]);
        assert_eq!(pixel(&joined, 0, 2), [0; 4]);
        let empty = Raster::hstack(&[], 5);
        assert_eq!((empty.width, empty.height, empty.data.len()), (0, 0, 0));
    }

    #[test]
    fn framed_paints_the_border_and_keeps_the_inside() {
        let plain = Raster {
            width: 40,
            height: 40,
            data: vec![7; 40 * 40 * 4],
        };
        let framed = plain.framed(Rgb(255, 176, 0));
        assert_eq!(pixel(&framed, 0, 20), [255, 176, 0, 255]);
        assert_eq!(pixel(&framed, 20, 39), [255, 176, 0, 255]);
        assert_eq!(pixel(&framed, 20, 20), [7; 4]);
    }
}
