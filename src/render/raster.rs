//! A small anti-aliased polygon filler: exact horizontal coverage, four
//! sub-scanlines per pixel, even-odd fill, premultiplied accumulation.

use super::scene::{Shape, HEIGHT, WIDTH};
use super::style::Rgb;

/// Straight (non-premultiplied) RGBA pixels, row by row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raster {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

const SUB: usize = 4;

pub(crate) fn rasterise(shapes: &[Shape], height: u32, background: Option<Rgb>) -> Raster {
    let k = f64::from(height) / HEIGHT;
    let width = (WIDTH * k).round() as u32;
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
}
