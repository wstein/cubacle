//! The package layout (plan section 3.5): which files go into `cubacle-wasm-<version>.tgz`.
//!
//! The JavaScript files (`index.mjs`, `worker.mjs`, `index.d.ts` in `crates/wasm/js/`) arrive
//! with the glue slice. Until a file exists it is skipped and `exports` names only what is
//! packed; no placeholder is created.

use std::fs;
use std::path::Path;

use crate::tarball::Entry;

/// Packed when present, skipped otherwise.
pub const OPTIONAL_JS: [&str; 3] = ["index.d.ts", "index.mjs", "worker.mjs"];

/// `package.json` text: sorted keys, no scripts. `has` tells which optional files are packed.
pub fn package_json(version: &str, has: &dyn Fn(&str) -> bool) -> String {
    let mut exports: Vec<String> = Vec::new();
    if has("index.mjs") {
        let types = if has("index.d.ts") {
            "\"types\": \"./index.d.ts\", "
        } else {
            ""
        };
        exports.push(format!(
            "    \".\": {{ {types}\"default\": \"./index.mjs\" }}"
        ));
    }
    exports.push("    \"./cubacle_wasm.wasm\": \"./cubacle_wasm.wasm\"".to_string());
    exports.push("    \"./package.json\": \"./package.json\"".to_string());
    if has("worker.mjs") {
        exports.push("    \"./worker\": \"./worker.mjs\"".to_string());
    }
    exports.sort();
    format!(
        "{{\n  \"exports\": {{\n{}\n  }},\n  \"license\": \"AGPL-3.0-or-later\",\n  \"name\": \"cubacle-wasm\",\n  \"type\": \"module\",\n  \"version\": \"{version}\"\n}}\n",
        exports.join(",\n")
    )
}

/// The entries of the package, and the optional files that were skipped.
pub fn entries(
    repo: &Path,
    dist: &Path,
    js_dir: &Path,
    version: &str,
) -> Result<(Vec<Entry>, Vec<String>), String> {
    let read = |path: &Path, name: &str| {
        fs::read(path)
            .map_err(|error| format!("{name}: {error}"))
            .map(|data| Entry {
                name: name.to_string(),
                data,
                mode: 0o644,
            })
    };
    let present = |name: &str| js_dir.join(name).is_file();
    let mut entries = vec![
        read(&dist.join(crate::wasm::MODULE), crate::wasm::MODULE)?,
        read(&dist.join(crate::wasm::MANIFEST), crate::wasm::MANIFEST)?,
        read(&repo.join("LICENSE"), "LICENSE")?,
        read(&repo.join("LICENSE-MIT"), "LICENSE-MIT")?,
        read(&repo.join("NOTICE.md"), "NOTICE.md")?,
        read(&repo.join("crates/wasm/README.md"), "README.md")?,
        Entry {
            name: "package.json".to_string(),
            data: package_json(version, &present).into_bytes(),
            mode: 0o644,
        },
    ];
    let mut skipped = Vec::new();
    for name in OPTIONAL_JS {
        if present(name) {
            entries.push(read(&js_dir.join(name), name)?);
        } else {
            skipped.push(name.to_string());
        }
    }
    Ok((entries, skipped))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_the_glue_exports_name_no_javascript() {
        let json = package_json("0.1.0", &|_| false);
        assert!(!json.contains(".mjs") && !json.contains(".d.ts"));
        assert!(json.contains("\"./cubacle_wasm.wasm\""));
        assert!(!json.contains("scripts"));
    }

    #[test]
    fn with_the_glue_exports_name_it() {
        let json = package_json("0.1.0", &|_| true);
        assert!(
            json.contains("\".\": { \"types\": \"./index.d.ts\", \"default\": \"./index.mjs\" }")
        );
        assert!(json.contains("\"./worker\": \"./worker.mjs\""));
        assert!(json.contains("\"license\": \"AGPL-3.0-or-later\""));
        assert!(json.contains("\"type\": \"module\""));
    }

    #[test]
    fn the_package_json_keys_are_in_byte_order() {
        let json = package_json("1.2.3", &|_| true);
        let keys: Vec<&str> = json
            .lines()
            .filter(|line| line.starts_with("  \""))
            .map(|line| line.trim().split('"').nth(1).unwrap())
            .collect();
        assert_eq!(keys, ["exports", "license", "name", "type", "version"]);
    }

    /// A scratch layout: module, manifest and an (empty or filled) glue directory.
    fn scratch(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!("cubacle-xtask-package-{name}"));
        let _ = fs::remove_dir_all(&root);
        let (dist, js) = (root.join("dist"), root.join("js"));
        fs::create_dir_all(&dist).unwrap();
        fs::create_dir_all(&js).unwrap();
        fs::write(dist.join(crate::wasm::MODULE), b"w").unwrap();
        fs::write(dist.join(crate::wasm::MANIFEST), b"{}").unwrap();
        (dist, js)
    }

    fn repo() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
    }

    #[test]
    fn without_the_glue_files_the_three_are_skipped_and_seven_files_are_packed() {
        let (dist, js) = scratch("none");
        let (entries, skipped) = entries(repo(), &dist, &js, "0.1.0").unwrap();
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(skipped, ["index.d.ts", "index.mjs", "worker.mjs"]);
        for name in [
            "package.json",
            "cubacle_wasm.wasm",
            "cubacle-wasm.manifest.json",
            "LICENSE",
            "LICENSE-MIT",
            "NOTICE.md",
            "README.md",
        ] {
            assert!(names.contains(&name), "{name} is packed");
        }
        assert_eq!(names.len(), 7);
        assert!(entries.iter().all(|e| e.mode == 0o644));
    }

    #[test]
    fn a_glue_file_that_exists_is_packed_and_named_in_exports() {
        let (dist, js) = scratch("some");
        fs::write(js.join("index.mjs"), b"export {};").unwrap();
        let (entries, skipped) = entries(repo(), &dist, &js, "0.1.0").unwrap();
        assert_eq!(skipped, ["index.d.ts", "worker.mjs"]);
        assert!(entries.iter().any(|e| e.name == "index.mjs"));
        let json = entries.iter().find(|e| e.name == "package.json").unwrap();
        assert!(String::from_utf8_lossy(&json.data).contains("./index.mjs"));
    }
}
