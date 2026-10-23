//! SVG output: one `<polygon>` per shape, two decimals, stable text.

use super::scene::Shape;
use super::style::Rgb;

/// A number with two decimals and no negative zero.
fn num(v: f64) -> String {
    let text = format!("{v:.2}");
    if text == "-0.00" {
        "0.00".to_owned()
    } else {
        text
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;")
}

pub(crate) fn write(
    shapes: &[Shape],
    label: &str,
    pixel_height: u32,
    background: Option<Rgb>,
    (canvas_width, canvas_height): (f64, f64),
) -> String {
    let width = (f64::from(pixel_height) * canvas_width / canvas_height).round() as u32;
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" width=\"{width}\" height=\"{pixel_height}\">\n<title>{}</title>\n",
        num(canvas_width),
        num(canvas_height),
        escape(label)
    );
    if let Some(colour) = background {
        out.push_str(&format!(
            "<rect width=\"{}\" height=\"{}\" fill=\"{}\"/>\n",
            num(canvas_width),
            num(canvas_height),
            colour.hex()
        ));
    }
    for shape in shapes {
        let points: Vec<String> = shape
            .points
            .iter()
            .map(|p| format!("{},{}", num(p[0]), num(p[1])))
            .collect();
        out.push_str(&format!(
            "<polygon points=\"{}\" fill=\"{}\"",
            points.join(" "),
            shape.fill.hex()
        ));
        if shape.opacity < 1.0 {
            out.push_str(&format!(" fill-opacity=\"{}\"", num(shape.opacity)));
        }
        out.push_str("/>\n");
    }
    out.push_str("</svg>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::scene::Part;

    #[test]
    fn writes_stable_text() {
        let shape = Shape::new(
            Part::Sticker,
            vec![[0.0, -0.001], [1.234, 2.0], [3.0, 4.005]],
            Rgb(255, 0, 16),
        )
        .with_opacity(0.5);
        let svg = write(&[shape], "R & U<", 100, Some(Rgb(0, 0, 0)), (150.0, 100.0));
        assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 150.00 100.00\" width=\"150\" height=\"100\">"));
        assert!(svg.contains("<title>R &amp; U&lt;</title>"));
        assert!(svg.contains(
            "points=\"0.00,0.00 1.23,2.00 3.00,4.00\" fill=\"#ff0010\" fill-opacity=\"0.50\"/>"
        ));
        assert!(svg.contains("<rect"));
    }
}
