//! `cubacle patterns update`: download the URL sources of the config into the
//! cache directory, and the same step for `auto_fetch`.
//!
//! Only URLs the user listed in the config are fetched. A download goes to
//! `NAME.part`, is read with the table reader and replaces the old file only
//! if it gives patterns; any failure leaves the old file as it was. The
//! validators of the server (ETag, Last-Modified) sit in `NAME.meta` next to
//! the file, so the next update can be a `304 Not Modified`.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::fetch::{self, Fetched, Validators};
use crate::patterns_config::Config;

#[derive(Clone, Copy, Default)]
pub struct Mode {
    /// Download even if the server says nothing changed.
    pub force: bool,
    /// Never touch the network.
    pub offline: bool,
}

pub enum Outcome {
    Downloaded { bytes: usize, patterns: usize },
    UpToDate,
}

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name: OsString = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

fn read_meta(path: &Path) -> (Validators, Option<u64>) {
    let mut have = Validators::default();
    let mut when = None;
    let Ok(text) = std::fs::read_to_string(sibling(path, ".meta")) else {
        return (have, when);
    };
    for line in text.lines() {
        match line.split_once(": ") {
            Some(("etag", v)) => have.etag = Some(v.to_owned()),
            Some(("last-modified", v)) => have.last_modified = Some(v.to_owned()),
            Some(("fetched", v)) => when = v.parse().ok(),
            _ => {}
        }
    }
    (have, when)
}

fn write_meta(path: &Path, url: &str, have: &Validators) -> std::io::Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let mut text = format!("url: {url}\nfetched: {now}\n");
    if let Some(v) = &have.etag {
        text.push_str(&format!("etag: {v}\n"));
    }
    if let Some(v) = &have.last_modified {
        text.push_str(&format!("last-modified: {v}\n"));
    }
    std::fs::write(sibling(path, ".meta"), text)
}

/// `YYYY-MM-DD` (UTC) of a day count since 1970-01-01.
fn civil(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

/// The day a download was made: from its `.meta`, else the file date.
pub fn downloaded_on(path: &Path) -> String {
    let secs = read_meta(path).1.or_else(|| {
        std::fs::metadata(path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
    });
    secs.map_or("at an unknown date".to_owned(), |s| {
        civil((s / 86_400) as i64)
    })
}

/// Fetches one URL source into the cache directory.
pub fn update_one(
    config: &Config,
    url: &str,
    size: Option<usize>,
    name_prefix: &str,
    mode: Mode,
) -> Result<Outcome, String> {
    if mode.offline {
        return Err("offline (--offline or CUBACLE_OFFLINE is set); nothing was fetched".into());
    }
    fetch::check_url(url)?;
    let path = config.download_path(url)?;
    let have = if path.is_file() && !mode.force {
        read_meta(&path).0
    } else {
        Validators::default()
    };
    let (bytes, validators) = match fetch::get(url, &have)? {
        Fetched::NotModified => return Ok(Outcome::UpToDate),
        Fetched::Body { bytes, validators } => (bytes, validators),
    };
    let dir = path.parent().ok_or("no cache directory")?;
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let part = sibling(&path, ".part");
    let result = (|| {
        std::fs::write(&part, &bytes).map_err(|e| format!("{}: {e}", part.display()))?;
        let (list, _) = config
            .read_file(&part, url, size, name_prefix)
            .map_err(|e| format!("not a pattern table, old file kept: {e}"))?;
        if list.is_empty() {
            return Err("the download holds no patterns, old file kept".to_owned());
        }
        std::fs::rename(&part, &path).map_err(|e| format!("{}: {e}", path.display()))?;
        // Without the sidecar the next run simply downloads again.
        let _ = write_meta(&path, url, &validators);
        Ok(list.len())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    result.map(|patterns| Outcome::Downloaded {
        bytes: bytes.len(),
        patterns,
    })
}

/// `patterns update`: every URL source, one result line each. An error when
/// any of them failed.
pub fn update(config: &Config, mode: Mode) -> Result<(), String> {
    let mut failed = 0;
    let mut total = 0;
    for (url, size, prefix) in config.url_sources() {
        total += 1;
        match update_one(config, url, size, prefix, mode) {
            Ok(Outcome::Downloaded { bytes, patterns }) => {
                println!("{url}: downloaded, {bytes} bytes, {patterns} patterns");
            }
            Ok(Outcome::UpToDate) => println!("{url}: up to date"),
            Err(reason) => {
                failed += 1;
                println!("{url}: failed: {reason}");
                eprintln!("error: {url}: {reason}");
            }
        }
    }
    if total == 0 {
        println!("no http(s) sources in the config; list a URL under [patterns] files");
    }
    match failed {
        0 => Ok(()),
        n => Err(format!("{n} of {total} sources failed")),
    }
}

/// `auto_fetch = true`: download the URL sources that are not in the cache.
/// A failure is a warning; the command goes on.
pub fn auto_fetch(config: &Config, mode: Mode) {
    if !config.auto_fetch || mode.offline {
        return;
    }
    for (url, size, prefix) in config.url_sources() {
        if config.download_path(url).is_ok_and(|p| p.is_file()) {
            continue;
        }
        eprintln!("downloading {url} ...");
        if let Err(reason) = update_one(config, url, size, prefix, mode) {
            eprintln!("warning: {url}: {reason}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_become_dates() {
        assert_eq!(civil(0), "1970-01-01");
        assert_eq!(civil(11_016), "2000-02-29");
        assert_eq!(civil(20_751), "2026-10-25");
    }

    #[test]
    fn offline_never_fetches() {
        let config = Config::default();
        let mode = Mode {
            force: false,
            offline: true,
        };
        let error = update_one(&config, "https://example.org/x", None, "", mode)
            .err()
            .unwrap();
        assert!(error.contains("offline"));
    }
}
