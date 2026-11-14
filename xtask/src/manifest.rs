//! The manifest of a wasm build: `cubacle-wasm.manifest.json`.
//!
//! Keys are written in byte order, nothing depends on time, so one input gives one text.

use std::collections::BTreeMap;
use std::fmt::Write;

pub const FORMAT: &str = "cubacle.wasm-manifest";
pub const FORMAT_VERSION: u32 = 1;

/// Size and SHA-256 (lowercase hex) of one file.
pub struct FileEntry {
    pub bytes: u64,
    pub sha256: String,
}

pub struct Manifest {
    pub version: String,
    pub commit: String,
    /// The output of `rustc -Vv`.
    pub rustc: String,
    pub files: BTreeMap<String, FileEntry>,
    /// Hashes of the serialized tables; empty until the table slice (L3) fills it.
    pub table_sha256: BTreeMap<String, String>,
}

/// JSON string literal: quotes, backslash and control characters are escaped.
fn quote(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => write!(out, "\\u{:04x}", c as u32).unwrap(),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

impl Manifest {
    /// Two-space indented JSON, keys sorted at every level, one trailing newline.
    pub fn to_json(&self) -> String {
        let mut out = String::from("{\n");
        writeln!(out, "  \"commit\": {},", quote(&self.commit)).unwrap();
        out.push_str("  \"files\": {");
        for (index, (name, file)) in self.files.iter().enumerate() {
            let comma = if index == 0 { "" } else { "," };
            write!(
                out,
                "{comma}\n    {}: {{\n      \"bytes\": {},\n      \"sha256\": {}\n    }}",
                quote(name),
                file.bytes,
                quote(&file.sha256)
            )
            .unwrap();
        }
        out.push_str(if self.files.is_empty() {
            "},\n"
        } else {
            "\n  },\n"
        });
        writeln!(out, "  \"format\": {},", quote(FORMAT)).unwrap();
        writeln!(out, "  \"formatVersion\": {FORMAT_VERSION},").unwrap();
        writeln!(out, "  \"rustc\": {},", quote(&self.rustc)).unwrap();
        out.push_str("  \"tableSha256\": {");
        for (index, (name, hash)) in self.table_sha256.iter().enumerate() {
            let comma = if index == 0 { "" } else { "," };
            write!(out, "{comma}\n    {}: {}", quote(name), quote(hash)).unwrap();
        }
        out.push_str(if self.table_sha256.is_empty() {
            "},\n"
        } else {
            "\n  },\n"
        });
        writeln!(out, "  \"version\": {}", quote(&self.version)).unwrap();
        out.push_str("}\n");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Manifest {
        let mut files = BTreeMap::new();
        files.insert(
            "cubacle_wasm.wasm".to_string(),
            FileEntry {
                bytes: 3,
                sha256: "ab".repeat(32),
            },
        );
        Manifest {
            version: "0.1.0".into(),
            commit: "c".repeat(40),
            rustc: "rustc 1.99.1\nrelease: 1.99.1".into(),
            files,
            table_sha256: BTreeMap::new(),
        }
    }

    #[test]
    fn top_level_keys_are_in_byte_order() {
        let json = sample().to_json();
        let keys: Vec<&str> = json
            .lines()
            .filter(|line| line.starts_with("  \""))
            .map(|line| line.trim().split('"').nth(1).unwrap())
            .collect();
        assert_eq!(
            keys,
            [
                "commit",
                "files",
                "format",
                "formatVersion",
                "rustc",
                "tableSha256",
                "version"
            ]
        );
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        assert_eq!(keys, sorted);
    }

    #[test]
    fn file_names_are_sorted_and_the_table_hashes_start_empty() {
        let mut manifest = sample();
        manifest.files.insert(
            "a.bin".into(),
            FileEntry {
                bytes: 1,
                sha256: "00".repeat(32),
            },
        );
        let json = manifest.to_json();
        assert!(json.find("\"a.bin\"").unwrap() < json.find("\"cubacle_wasm.wasm\"").unwrap());
        assert!(json.contains("\"tableSha256\": {},"));
    }

    #[test]
    fn text_is_escaped_and_has_no_time_stamp() {
        let json = sample().to_json();
        assert!(json.contains("rustc 1.99.1\\nrelease: 1.99.1"));
        assert!(json.ends_with("}\n"));
        assert!(!json.to_lowercase().contains("time"));
    }

    #[test]
    fn the_exact_text_has_the_values_escapes_and_commas() {
        let mut manifest = sample();
        manifest.files.insert(
            "a\"b\\c\u{1}.bin".into(),
            FileEntry {
                bytes: 7,
                sha256: "01".repeat(32),
            },
        );
        manifest.table_sha256.insert("t1".into(), "d1".repeat(32));
        manifest.table_sha256.insert("t2".into(), "d2".repeat(32));
        manifest.rustc = "q\"r\ts\u{1f}".into();
        let expected = format!(
            "{{\n  \"commit\": \"{c}\",\n  \"files\": {{\n    \"a\\\"b\\\\c\\u0001.bin\": {{\n      \"bytes\": 7,\n      \"sha256\": \"{h1}\"\n    }},\n    \"cubacle_wasm.wasm\": {{\n      \"bytes\": 3,\n      \"sha256\": \"{h2}\"\n    }}\n  }},\n  \"format\": \"cubacle.wasm-manifest\",\n  \"formatVersion\": 1,\n  \"rustc\": \"q\\\"r\\ts\\u001f\",\n  \"tableSha256\": {{\n    \"t1\": \"{t1}\",\n    \"t2\": \"{t2}\"\n  }},\n  \"version\": \"0.1.0\"\n}}\n",
            c = "c".repeat(40),
            h1 = "01".repeat(32),
            h2 = "ab".repeat(32),
            t1 = "d1".repeat(32),
            t2 = "d2".repeat(32),
        );
        assert_eq!(manifest.to_json(), expected);
    }

    #[test]
    fn the_same_input_gives_the_same_text() {
        assert_eq!(sample().to_json(), sample().to_json());
    }
}
