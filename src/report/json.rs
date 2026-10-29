//! A small JSON writer: compact output, keys in the order the caller writes
//! them, strings escaped as RFC 8259 asks.

/// Appends `text` as a JSON string: quotes, backslash and control characters
/// are escaped, everything else stays as it is.
pub fn write_string(out: &mut String, text: &str) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if u32::from(c) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", u32::from(c)));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// An object under construction: `{` ... `}` with the keys in call order.
pub struct Object<'a> {
    out: &'a mut String,
    first: bool,
}

impl<'a> Object<'a> {
    /// Opens an object.
    pub fn open(out: &'a mut String) -> Object<'a> {
        out.push('{');
        Object { out, first: true }
    }

    fn key(&mut self, name: &str) {
        if !self.first {
            self.out.push(',');
        }
        self.first = false;
        write_string(self.out, name);
        self.out.push(':');
    }

    /// A string member.
    pub fn string(&mut self, name: &str, value: &str) {
        self.key(name);
        write_string(self.out, value);
    }

    /// A number member.
    pub fn number(&mut self, name: &str, value: impl std::fmt::Display) {
        self.key(name);
        self.out.push_str(&value.to_string());
    }

    /// A member written by `write`, which appends one JSON value.
    pub fn value(&mut self, name: &str, write: impl FnOnce(&mut String)) {
        self.key(name);
        write(self.out);
    }

    /// Closes the object.
    pub fn close(self) {
        self.out.push('}');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn escaped(text: &str) -> String {
        let mut out = String::new();
        write_string(&mut out, text);
        out
    }

    #[test]
    fn strings_are_escaped() {
        assert_eq!(escaped("R U'"), "\"R U'\"");
        assert_eq!(escaped("a\"b\\c"), r#""a\"b\\c""#);
        assert_eq!(escaped("x\ny\tz\r"), r#""x\ny\tz\r""#);
        assert_eq!(escaped("\u{1}\u{1f}\u{8}\u{c}"), r#""\u0001\u001f\b\f""#);
        assert_eq!(escaped("é · ✓"), "\"é · ✓\"");
    }

    #[test]
    fn keys_keep_the_order_written() {
        let mut out = String::new();
        let mut object = Object::open(&mut out);
        object.string("z", "1");
        object.number("a", 2);
        object.value("m", |out| out.push_str("[]"));
        object.close();
        assert_eq!(out, r#"{"z":"1","a":2,"m":[]}"#);
    }

    #[test]
    fn an_empty_object_is_braces() {
        let mut out = String::new();
        Object::open(&mut out).close();
        assert_eq!(out, "{}");
    }
}
