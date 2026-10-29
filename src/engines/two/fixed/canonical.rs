//! The specified ASCII wire normal form, without a JSON parser dependency in the engine.

use super::{Accept, Cell, Rule};

fn string(out: &mut String, value: &str) {
    use std::fmt::Write;
    out.push('"');
    for unit in value.encode_utf16() {
        match unit {
            0x22 => out.push_str("\\\""),
            0x5c => out.push_str("\\\\"),
            0x20..=0x7e => out.push(char::from_u32(u32::from(unit)).unwrap()),
            _ => write!(out, "\\u{unit:04x}").unwrap(),
        }
    }
    out.push('"');
}

fn vector(out: &mut String, value: [i8; 3]) {
    use std::fmt::Write;
    write!(out, "[{},{},{}]", value[0], value[1], value[2]).unwrap();
}

pub(super) fn rule(rule: &Rule) -> String {
    use std::fmt::Write;
    let mut out = String::from("{");
    match &rule.accept {
        Accept::Ids(ids) => {
            out.push_str("\"ids\":[");
            for (i, id) in ids.iter().enumerate() {
                if i != 0 {
                    out.push(',');
                }
                write!(out, "{id}").unwrap();
            }
            out.push(']');
        }
        Accept::Image(image) => {
            out.push_str("\"image\":");
            string(&mut out, image);
        }
    }
    if let Some(up) = rule.up {
        out.push_str(",\"up\":");
        vector(&mut out, up);
    }
    out.push('}');
    out
}

pub(super) fn display(cells: &[Cell]) -> String {
    let mut out = String::from("{\"cells\":[");
    for (i, cell) in cells.iter().enumerate() {
        if i != 0 {
            out.push(',');
        }
        out.push_str("{\"p\":");
        vector(&mut out, cell.p);
        out.push_str(",\"n\":");
        vector(&mut out, cell.n);
        out.push_str(",\"accept\":[");
        for (j, accepted) in cell.accept.iter().enumerate() {
            if j != 0 {
                out.push(',');
            }
            out.push_str(&rule(accepted));
        }
        out.push_str("]}");
    }
    out.push_str("]}");
    out
}
