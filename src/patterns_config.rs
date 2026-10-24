//! `cubacle.toml`: which pattern files the `patterns` commands read.
//!
//! The file is looked up as `--config FILE`, `$CUBACLE_CONFIG`,
//! `./cubacle.toml`, then `~/.config/cubacle/cubacle.toml` (or
//! `$XDG_CONFIG_HOME/cubacle/`). Without one, the list is the built-in library.
//!
//! ```toml
//! [patterns]
//! files = ["builtin", "tables/3x3.txt", { path = "mine.txt", size = 3, name_prefix = "my-" }]
//! cache_dir = "~/.cache/cubacle/patterns"   # optional
//! auto_fetch = false                         # downloading is not built in
//! ```

use std::path::{Path, PathBuf};

use cubacle::patterns::{
    self, decode_text, import_tab, looks_imported, read_imported, ImportOptions, Pattern,
};

use crate::patterns_import_cmd::{default_data_dir, size_from_name};

/// One element of `files`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    Builtin,
    /// A local file, or a URL whose download sits in the cache directory.
    Source {
        location: String,
        size: Option<usize>,
        name_prefix: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// The file this came from; `None` for the default.
    pub path: Option<PathBuf>,
    pub files: Vec<Entry>,
    pub cache_dir: Option<PathBuf>,
    pub auto_fetch: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            path: None,
            files: vec![Entry::Builtin],
            cache_dir: None,
            auto_fetch: false,
        }
    }
}

/// The patterns the commands work on.
#[derive(Default)]
pub struct Library {
    pub builtin: bool,
    pub extra: Vec<Pattern>,
    /// One line per entry: what was read, or why not.
    pub notes: Vec<String>,
    /// The entries that could not be read.
    pub warnings: Vec<String>,
}

impl Library {
    pub fn all(&self) -> impl Iterator<Item = &Pattern> {
        let builtin = if self.builtin { patterns::all() } else { &[] };
        builtin.iter().chain(self.extra.iter())
    }

    pub fn find(&self, name: &str) -> Option<&Pattern> {
        self.all().find(|p| p.is_named(name))
    }
}

fn config_home() -> Option<PathBuf> {
    let var = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
    var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|d| d.is_absolute())
        .or_else(|| var("HOME").map(|h| PathBuf::from(h).join(".config")))
        .map(|d| d.join("cubacle"))
}

/// The config file to use, if any, in the documented order.
pub fn locate(explicit: Option<&Path>) -> Result<Option<PathBuf>, String> {
    if let Some(path) = explicit {
        return Ok(Some(path.to_owned()));
    }
    if let Some(path) = std::env::var_os("CUBACLE_CONFIG").filter(|v| !v.is_empty()) {
        return Ok(Some(PathBuf::from(path)));
    }
    let here = PathBuf::from("cubacle.toml");
    if here.is_file() {
        return Ok(Some(here));
    }
    Ok(config_home()
        .map(|d| d.join("cubacle.toml"))
        .filter(|p| p.is_file()))
}

/// 1-based line of a byte offset.
fn line_of(text: &str, offset: usize) -> usize {
    text[..offset.min(text.len())].matches('\n').count() + 1
}

fn expand_home(text: &str) -> PathBuf {
    match (text.strip_prefix("~/"), std::env::var_os("HOME")) {
        (Some(rest), Some(home)) => PathBuf::from(home).join(rest),
        _ => PathBuf::from(text),
    }
}

/// Reads a config file; errors name the file and the line.
pub fn parse(text: &str, path: &Path) -> Result<Config, String> {
    let shown = path.display();
    let table: toml::Table = text.parse().map_err(|e: toml::de::Error| {
        let line = e.span().map_or(0, |s| line_of(text, s.start));
        format!("{shown}:{line}: {}", e.message())
    })?;
    let bad = |m: String| format!("{shown}: {m}");
    let mut config = Config {
        path: Some(path.to_owned()),
        ..Config::default()
    };
    for (key, value) in &table {
        if key != "patterns" {
            return Err(bad(format!("unknown section [{key}]")));
        }
        let section = value
            .as_table()
            .ok_or_else(|| bad("patterns must be a table".into()))?;
        for (key, value) in section {
            match key.as_str() {
                "files" => config.files = entries(value).map_err(bad)?,
                "cache_dir" => {
                    let dir = value
                        .as_str()
                        .ok_or_else(|| bad("cache_dir is a string".into()))?;
                    config.cache_dir = Some(expand_home(dir));
                }
                "auto_fetch" => {
                    config.auto_fetch = value
                        .as_bool()
                        .ok_or_else(|| bad("auto_fetch is true or false".into()))?;
                }
                other => return Err(bad(format!("unknown key patterns.{other}"))),
            }
        }
    }
    Ok(config)
}

fn entries(value: &toml::Value) -> Result<Vec<Entry>, String> {
    let list = value.as_array().ok_or("files is a list")?;
    list.iter()
        .map(|item| match item {
            toml::Value::String(s) if s == "builtin" => Ok(Entry::Builtin),
            toml::Value::String(s) => Ok(Entry::Source {
                location: s.clone(),
                size: None,
                name_prefix: String::new(),
            }),
            toml::Value::Table(t) => {
                let location = t
                    .get("path")
                    .or_else(|| t.get("url"))
                    .and_then(toml::Value::as_str)
                    .ok_or("a files entry needs path or url")?;
                let size = match t.get("size") {
                    Some(v) => Some(
                        v.as_integer()
                            .and_then(|n| usize::try_from(n).ok())
                            .ok_or("size is a number")?,
                    ),
                    None => None,
                };
                let name_prefix = t
                    .get("name_prefix")
                    .and_then(toml::Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                Ok(Entry::Source {
                    location: location.to_owned(),
                    size,
                    name_prefix,
                })
            }
            _ => Err("a files entry is a string or a table".to_owned()),
        })
        .collect()
}

pub fn load(explicit: Option<&Path>) -> Result<Config, String> {
    match locate(explicit)? {
        None => Ok(Config::default()),
        Some(path) => {
            let text =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            parse(&text, &path)
        }
    }
}

impl Config {
    /// The data directory: `cache_dir`, else the default.
    pub fn data_dir(&self) -> Result<PathBuf, String> {
        match &self.cache_dir {
            Some(dir) => Ok(dir.clone()),
            None => default_data_dir(),
        }
    }

    /// Where the file for a URL has to be put by hand.
    pub fn download_path(&self, url: &str) -> Result<PathBuf, String> {
        let name: String = url
            .split("://")
            .last()
            .unwrap_or(url)
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '.' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        Ok(self.data_dir()?.join("downloads").join(name))
    }

    /// Reads every entry. A broken entry is a note, not an error.
    pub fn library(&self) -> Library {
        let base = self
            .path
            .as_deref()
            .and_then(Path::parent)
            .map(Path::to_owned)
            .unwrap_or_default();
        let mut library = Library::default();
        for entry in &self.files {
            match entry {
                Entry::Builtin => {
                    library.builtin = true;
                    library
                        .notes
                        .push("builtin: the built-in library".to_owned());
                }
                Entry::Source {
                    location,
                    size,
                    name_prefix,
                } => match self.read_source(&base, location, *size, name_prefix) {
                    Ok((list, note)) => {
                        library.extra.extend(list);
                        library.notes.push(format!("{location}: {note}"));
                    }
                    Err(error) => {
                        library.notes.push(format!("{location}: {error}"));
                        library.warnings.push(format!("{location}: {error}"));
                    }
                },
            }
        }
        library
    }

    fn read_source(
        &self,
        base: &Path,
        location: &str,
        size: Option<usize>,
        name_prefix: &str,
    ) -> Result<(Vec<Pattern>, String), String> {
        let path = if location.contains("://") {
            let path = self.download_path(location)?;
            if !path.is_file() {
                return Err(format!(
                    "not downloaded; cubacle does not fetch: curl -o '{}' --create-dirs '{location}'",
                    path.display()
                ));
            }
            path
        } else {
            base.join(expand_home(location))
        };
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let text = decode_text(&bytes);
        let label = location.rsplit('/').next().unwrap_or(location).to_owned();
        if looks_imported(&text) {
            let list =
                read_imported(&text, &label).map_err(|e| format!("{}: {e}", path.display()))?;
            let note = format!("{} patterns", list.len());
            return Ok((list, note));
        }
        let size = size
            .or_else(|| size_from_name(location))
            .ok_or("cannot tell the cube size: use { path = ..., size = N }")?;
        let kind = crate::kind_of(size)?;
        let got = import_tab(
            &text,
            &ImportOptions {
                size,
                name_prefix: name_prefix.to_owned(),
                origin: label,
                source: "ct".to_owned(),
            },
            &crate::patterns_import_cmd::reader(kind),
        );
        let note = format!(
            "{} patterns, {} of {} lines skipped (see `patterns import --dry-run`)",
            got.patterns.len(),
            got.report.problems.len(),
            got.report.lines
        );
        Ok((got.patterns, note))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_text(text: &str) -> Result<Config, String> {
        parse(text, Path::new("c.toml"))
    }

    #[test]
    fn default_is_the_builtin_library() {
        let config = Config::default();
        assert_eq!(config.files, vec![Entry::Builtin]);
        assert!(!config.auto_fetch);
    }

    #[test]
    fn reads_files_strings_and_tables() {
        let config = parse_text(
            "[patterns]\nfiles = [\"builtin\", \"a.txt\", { path = \"b.txt\", size = 4, name_prefix = \"p-\" }]\nauto_fetch = false\n",
        )
        .unwrap();
        assert_eq!(config.files.len(), 3);
        assert_eq!(
            config.files[2],
            Entry::Source {
                location: "b.txt".into(),
                size: Some(4),
                name_prefix: "p-".into()
            }
        );
    }

    #[test]
    fn errors_name_file_and_line() {
        let error = parse_text("[patterns]\nfiles = [\n").unwrap_err();
        assert!(error.starts_with("c.toml:"), "{error}");
        let error = parse_text("[patterns]\n\nfile = []\n").unwrap_err();
        assert!(error.contains("unknown key patterns.file"), "{error}");
        let error = parse_text("[patterns]\n\n\nfiles = 3\n").unwrap_err();
        assert!(error.contains("files is a list"), "{error}");
    }

    #[test]
    fn the_example_config_enables_only_the_builtin_library() {
        let text = include_str!("../cubacle.example.toml");
        let config = parse_text(text).unwrap();
        assert_eq!(config.files, vec![Entry::Builtin]);
        assert!(text.contains("# \"https://raw.githubusercontent.com/wrandelshofer/CubeTwister/"));
    }

    #[test]
    fn a_comment_only_file_is_the_default_list() {
        let config = parse_text("# nothing\n").unwrap();
        assert_eq!(config.files, vec![Entry::Builtin]);
    }

    #[test]
    fn a_url_is_found_in_the_cache_dir() {
        let config = Config {
            cache_dir: Some(PathBuf::from("/c")),
            ..Config::default()
        };
        let path = config.download_path("https://h/x/3xA/F.txt").unwrap();
        assert_eq!(path, PathBuf::from("/c/downloads/h_x_3xA_F.txt"));
    }
}
