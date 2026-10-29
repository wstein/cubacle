//! `cubacle.toml`: which pattern files the `patterns` commands read.
//!
//! The file is looked up as `--config FILE`, `$CUBACLE_CONFIG`,
//! `./cubacle.toml`, then the global `~/.config/cubacle/config.toml` (or
//! `$XDG_CONFIG_HOME/cubacle/config.toml`). When none exists, the global file
//! is created from `cubacle.example.toml` on the first run. URL sources are
//! downloaded only by `cubacle patterns update` (or by `auto_fetch = true`).
//!
//! The built-in library is always loaded; `files` adds sources and cannot drop it.
//!
//! ```toml
//! [patterns]
//! files = ["builtin", "tables/3x3.txt", { path = "mine.txt", size = 3, name_prefix = "my-" }]
//! cache_dir = "~/.cache/cubacle/patterns"   # optional
//! auto_fetch = false                         # true: download missing URL sources by itself
//! ```

use std::path::{Path, PathBuf};

use cubacle::patterns::{
    self, decode_text, import_tab, imported_is_stale, looks_imported, read_imported, ImportOptions,
    Pattern,
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

/// A source that is fetched rather than read from disk.
pub fn is_url(location: &str) -> bool {
    location.contains("://")
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
    /// URL sources that are not in the cache yet.
    pub missing: Vec<String>,
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
    Ok(global_path().filter(|p| p.is_file()))
}

fn global_path() -> Option<PathBuf> {
    config_home().map(|d| d.join("config.toml"))
}

/// The example, which is also the file written on the first run.
const EXAMPLE: &str = include_str!("../cubacle.example.toml");

/// Writes the global config from the example if it does not exist yet.
/// Returns the path when it was created; says on stderr what happened.
fn create_global() -> Option<PathBuf> {
    use std::io::Write;
    let path = global_path()?;
    let result = path
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|()| {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
        })
        .and_then(|mut f| f.write_all(EXAMPLE.as_bytes()));
    match result {
        Ok(()) => {
            eprintln!(
                "cubacle: created {} (built-in library only; third-party sources are commented out, read the notes inside)",
                path.display()
            );
            Some(path)
        }
        Err(e) => {
            eprintln!(
                "cubacle: could not create {}: {e}; continuing with the built-in library",
                path.display()
            );
            None
        }
    }
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
        None => Ok(create_global().map_or_else(Config::default, |path| Config {
            path: Some(path),
            ..Config::default()
        })),
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

    /// The `http(s)` sources of the config: location, size, name prefix.
    pub fn url_sources(&self) -> impl Iterator<Item = (&str, Option<usize>, &str)> {
        self.files.iter().filter_map(|entry| match entry {
            Entry::Source {
                location,
                size,
                name_prefix,
            } if is_url(location) => Some((location.as_str(), *size, name_prefix.as_str())),
            _ => None,
        })
    }

    /// Where the download of a URL sits in the cache directory.
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
        // A name of dots only would leave the directory.
        let name = if name.chars().all(|c| c == '.') {
            format!("_{name}")
        } else {
            name
        };
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
        // The built-in library is always loaded; the list only adds to it.
        let mut library = Library {
            builtin: true,
            ..Library::default()
        };
        library
            .notes
            .push("builtin: the built-in library".to_owned());
        for entry in &self.files {
            match entry {
                Entry::Builtin => {}
                Entry::Source {
                    location,
                    size,
                    name_prefix,
                } if is_url(location)
                    && self.download_path(location).is_ok_and(|p| !p.is_file()) =>
                {
                    library.missing.push(location.clone());
                    library.notes.push(format!("{location}: not downloaded"));
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
        if is_url(location) {
            let path = self.download_path(location)?;
            let (list, note) = self.read_file(&path, location, size, name_prefix)?;
            let bytes = std::fs::metadata(&path).map_or(0, |m| m.len());
            let when = crate::patterns_update::downloaded_on(&path);
            return Ok((list, format!("downloaded {when}, {bytes} bytes, {note}")));
        }
        self.read_file(
            &base.join(expand_home(location)),
            location,
            size,
            name_prefix,
        )
    }

    /// Reads one table file (also a `.part` that is being checked).
    pub fn read_file(
        &self,
        path: &Path,
        location: &str,
        size: Option<usize>,
        name_prefix: &str,
    ) -> Result<(Vec<Pattern>, String), String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let text = decode_text(&bytes);
        let label = location.rsplit('/').next().unwrap_or(location).to_owned();
        if looks_imported(&text) {
            let list =
                read_imported(&text, &label).map_err(|e| format!("{}: {e}", path.display()))?;
            if imported_is_stale(&text) {
                // Another patternness formula: write the file again with
                // fresh values (a read-only cache is left as it is).
                let _ = std::fs::write(path, patterns::write_imported(&list, &label));
            }
            let note = format!("{} patterns", list.len());
            return Ok((list, note));
        }
        let size = size
            .or_else(|| size_from_name(location))
            .ok_or("cannot tell the cube size: use { path = ..., size = N }")?;
        let kind = crate::input::kind_of(size)?;
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
    fn the_builtin_library_is_loaded_without_being_listed() {
        let config = parse_text("[patterns]\nfiles = []\n").unwrap();
        let library = config.library();
        assert!(library.builtin);
        assert_eq!(library.notes, vec!["builtin: the built-in library"]);
        assert!(library.all().count() > 0);
        let listed = parse_text("[patterns]\nfiles = [\"builtin\"]\n").unwrap();
        assert_eq!(listed.library().notes, library.notes);
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
