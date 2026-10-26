//! The options as one JSON object, in a fixed field order, for the
//! fingerprint that `cubacle photo` writes into its PNG files.

use super::options::{Logo, Options, Preset};

/// A JSON number; `null` for a value JSON cannot hold.
fn number(value: f64) -> String {
    if value.is_finite() {
        format!("{value}")
    } else {
        "null".to_owned()
    }
}

/// A JSON string; every non-ASCII character is escaped, so the text is
/// valid Latin-1 as well as UTF-8.
fn string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 || (c as u32) > 0x7E => {
                let mut units = [0u16; 2];
                for unit in c.encode_utf16(&mut units) {
                    out.push_str(&format!("\\u{unit:04x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn object(fields: Vec<(&str, String)>) -> String {
    let body: Vec<String> = fields
        .into_iter()
        .map(|(key, value)| format!("\"{key}\":{value}"))
        .collect();
    format!("{{{}}}", body.join(","))
}

fn logo(logo: &Logo) -> String {
    let mut fields = vec![
        ("row", logo.row.to_string()),
        ("col", logo.col.to_string()),
        ("color", string(&logo.color.to_string())),
    ];
    if let Some(size) = logo.size {
        fields.push(("size", number(size)));
    }
    object(fields)
}

/// The fingerprint of one photo as a JSON object, fields in this order:
/// `v` (1), `renderer` (`rust/v1`), `preset`, `seed`, `face`, `size`,
/// `options` (see [`Options::to_json`]) and `app` (`{"version": ...}`).
pub fn fingerprint_json(
    preset: Preset,
    seed: i32,
    face: &str,
    size: usize,
    options: &Options,
    app_version: &str,
) -> String {
    format!(
        "{{\"v\":1,\"renderer\":\"rust/v1\",\"preset\":{},\"seed\":{seed},\"face\":{},\"size\":{size},\"options\":{},\"app\":{{\"version\":{}}}}}",
        string(preset.name()),
        string(face),
        options.to_json(),
        string(app_version),
    )
}

impl Options {
    /// The options that are set, as a JSON object with the keys of the
    /// specification in this order: `scatter`, `lightness`, `palette`
    /// (letters sorted, each `[r, g, b]`), `logo`, `noise`, `blur`,
    /// `brightness`, `shadow` (`strength`, `shape`, `softness`) and
    /// `geometry` (`margin`, `gap`, `radius`). Options that are not set are
    /// left out, so the default options give `{}`.
    pub fn to_json(&self) -> String {
        let mut fields: Vec<(&str, String)> = Vec::new();
        let mut number_field = |key, value: Option<f64>| {
            if let Some(value) = value {
                fields.push((key, number(value)));
            }
        };
        number_field("scatter", self.scatter);
        number_field("lightness", self.lightness);
        if !self.palette.is_empty() {
            let colors = self
                .palette
                .iter()
                .map(|(letter, c)| {
                    (
                        string(&letter.to_string()),
                        format!("[{},{},{}]", number(c.r), number(c.g), number(c.b)),
                    )
                })
                .map(|(key, value)| format!("{key}:{value}"))
                .collect::<Vec<_>>();
            fields.push(("palette", format!("{{{}}}", colors.join(","))));
        }
        if let Some(l) = &self.logo {
            fields.push(("logo", logo(l)));
        }
        let mut number_field = |key, value: Option<f64>| {
            if let Some(value) = value {
                fields.push((key, number(value)));
            }
        };
        number_field("noise", self.noise);
        number_field("blur", self.blur);
        number_field("brightness", self.brightness);
        if let Some(shadow) = &self.shadow {
            let mut inner = Vec::new();
            if let Some(v) = shadow.strength {
                inner.push(("strength", number(v)));
            }
            if let Some(shape) = shadow.shape {
                inner.push(("shape", string(shape.name())));
            }
            if let Some(v) = shadow.softness {
                inner.push(("softness", number(v)));
            }
            fields.push(("shadow", object(inner)));
        }
        let g = &self.geometry;
        let mut inner = Vec::new();
        for (key, value) in [("margin", g.margin), ("gap", g.gap), ("radius", g.radius)] {
            if let Some(v) = value {
                inner.push((key, number(v)));
            }
        }
        if !inner.is_empty() {
            fields.push(("geometry", object(inner)));
        }
        object(fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synthetic::{Rgb, Shadow, ShadowShape};

    #[test]
    fn default_options_are_an_empty_object() {
        assert_eq!(Options::default().to_json(), "{}");
    }

    #[test]
    fn fields_come_in_a_fixed_order() {
        let options = Options {
            noise: Some(12.0),
            scatter: Some(0.5),
            palette: [('W', Rgb::new(200.0, 40.0, 40.0))].into(),
            shadow: Some(Shadow {
                strength: Some(0.3),
                shape: Some(ShadowShape::Half),
                softness: None,
            }),
            ..Options::default()
        };
        assert_eq!(
            options.to_json(),
            r#"{"scatter":0.5,"palette":{"W":[200,40,40]},"noise":12,"shadow":{"strength":0.3,"shape":"half"}}"#
        );
    }

    #[test]
    fn the_fingerprint_has_a_fixed_shape() {
        let options = Options {
            noise: Some(12.0),
            ..Options::default()
        };
        assert_eq!(
            fingerprint_json(Preset::Glare, 5, "u", 64, &options, "0.1.0"),
            r#"{"v":1,"renderer":"rust/v1","preset":"glare","seed":5,"face":"u","size":64,"options":{"noise":12},"app":{"version":"0.1.0"}}"#
        );
    }

    #[test]
    fn non_ascii_is_escaped() {
        let text: String = ['a', '"', '\u{e9}', '\u{1F600}'].iter().collect();
        assert_eq!(
            string(&text),
            [r#""a\""#, "\\u00e9", "\\ud83d", "\\ude00", "\""].concat()
        );
    }
}
