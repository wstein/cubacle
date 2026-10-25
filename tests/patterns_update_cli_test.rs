//! `patterns update` and `auto_fetch` against a local HTTP server on
//! 127.0.0.1 and an invented table. No test uses the internet, and every one
//! points HOME, the config and the cache to its own temporary directory.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::{response, response_announcing, table, Server};

struct Env {
    dir: PathBuf,
}

impl Env {
    fn new(name: &str, urls: &[String], extra: &str) -> Env {
        let dir =
            std::env::temp_dir().join(format!("cubacle-update-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let list: Vec<String> = urls.iter().map(|u| format!("\"{u}\"")).collect();
        std::fs::write(
            dir.join("config.toml"),
            format!(
                "[patterns]\nfiles = [\"builtin\", {}]\n{extra}\n",
                list.join(", ")
            ),
        )
        .unwrap();
        Env { dir }
    }

    fn run(&self, args: &[&str], envs: &[(&str, &str)]) -> (bool, String, String) {
        let mut command = Command::new(env!("CARGO_BIN_EXE_cubacle"));
        command
            .args(args)
            .current_dir(&self.dir)
            .env("HOME", &self.dir)
            .env("XDG_CONFIG_HOME", self.dir.join("xdg"))
            .env("XDG_CACHE_HOME", self.dir.join("xdg-cache"))
            .env("CUBACLE_CACHE_DIR", self.dir.join("cache"))
            .env("CUBACLE_CONFIG", self.dir.join("config.toml"))
            .env_remove("CUBACLE_OFFLINE");
        for name in [
            "HTTPS_PROXY",
            "https_proxy",
            "ALL_PROXY",
            "all_proxy",
            "HTTP_PROXY",
            "http_proxy",
        ] {
            command.env_remove(name);
        }
        for (k, v) in envs {
            command.env(k, v);
        }
        let out = command.output().expect("cubacle runs");
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }

    fn downloads(&self) -> PathBuf {
        self.dir.join("cache/patterns/downloads")
    }

    fn files(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(self.downloads())
            .map(|d| {
                d.filter_map(Result::ok)
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }

    fn the_file(&self) -> PathBuf {
        let name = self
            .files()
            .into_iter()
            .find(|n| n.ends_with(".txt"))
            .expect("a download");
        self.downloads().join(name)
    }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

fn plain_server() -> Server {
    Server::start(|_| response("200 OK", &[], table().as_bytes()))
}

#[test]
fn update_downloads_and_the_patterns_are_used() {
    let server = plain_server();
    let url = server.url("/3xT/t.txt");
    let env = Env::new("ok", std::slice::from_ref(&url), "");
    let (ok, out, err) = env.run(&["patterns", "sources"], &[]);
    assert!(ok, "{err}");
    assert!(out.contains(&format!("{url}: not downloaded")), "{out}");
    assert_eq!(server.hits(), 0, "sources fetches nothing");
    let (ok, _, err) = env.run(&["patterns", "--size", "3"], &[]);
    assert!(ok, "{err}");
    assert!(
        err.contains("1 source is not downloaded; run `cubacle patterns update`"),
        "{err}"
    );
    assert!(
        !err.contains("not downloaded: http"),
        "details only with -v: {err}"
    );
    let (_, _, err) = env.run(&["patterns", "-v", "--size", "3"], &[]);
    assert!(err.contains(&format!("not downloaded: {url}")), "{err}");
    let (ok, out, err) = env.run(&["patterns", "update"], &[]);
    assert!(ok, "{err}");
    assert!(
        out.contains("downloaded, ") && out.contains("2 patterns"),
        "{out}"
    );
    assert_eq!(read(&env.the_file()), table());
    assert!(env.files().iter().any(|n| n.ends_with(".txt.meta")));
    assert!(!env.files().iter().any(|n| n.ends_with(".part")));
    let (ok, out, err) = env.run(&["patterns", "--size", "3"], &[]);
    assert!(ok && !err.contains("not downloaded"), "{err}");
    assert!(out.contains("t1-01\tPair"), "{out}");
    let (_, out, _) = env.run(&["patterns", "sources"], &[]);
    assert!(
        out.contains("downloaded 20") && out.contains(" bytes, 2 patterns"),
        "{out}"
    );
    // The alias.
    let (ok, _, err) = env.run(&["patterns", "download", "--force"], &[]);
    assert!(ok, "{err}");
}

#[test]
fn an_unchanged_file_is_not_transferred_again() {
    let server = Server::start(|request| match request.headers.get("if-none-match") {
        Some(tag) if tag == "\"v1\"" => response("304 Not Modified", &[], b""),
        _ => response("200 OK", &[("ETag", "\"v1\"")], table().as_bytes()),
    });
    let env = Env::new("etag", &[server.url("/3xT/t.txt")], "");
    let (ok, out, _) = env.run(&["patterns", "update"], &[]);
    assert!(ok && out.contains("downloaded"), "{out}");
    let meta = std::fs::read_to_string(format!("{}.meta", env.the_file().display())).unwrap();
    assert!(meta.contains("etag: \"v1\""), "{meta}");
    let (ok, out, _) = env.run(&["patterns", "update"], &[]);
    assert!(ok && out.contains("up to date"), "{out}");
    let (ok, out, _) = env.run(&["patterns", "update", "--force"], &[]);
    assert!(ok && out.contains("downloaded"), "{out}");
    assert_eq!(server.hits(), 3);
}

#[test]
fn garbage_and_truncated_bodies_leave_the_old_file() {
    let good = plain_server();
    let env = Env::new("bad", &[good.url("/3xT/t.txt")], "");
    // Same URL path, different server: build the replacement servers on the
    // same port by rewriting the config instead.
    let (ok, _, err) = env.run(&["patterns", "update"], &[]);
    assert!(ok, "{err}");
    let old = read(&env.the_file());
    let cases: Vec<(&str, Server)> = vec![
        (
            "garbage",
            Server::start(|_| response("200 OK", &[], b"hello\nthis is not a table\n")),
        ),
        (
            "truncated",
            Server::start(|_| response_announcing(100_000, b"R2")),
        ),
        ("empty", Server::start(|_| response("200 OK", &[], b""))),
    ];
    for (name, server) in cases {
        // Point the same download file at the bad server: the name derives
        // from the URL, so keep the old URL in the config and the file name.
        let url = server.url("/3xT/t.txt");
        let env2 = Env::new(&format!("bad-{name}"), &[url], "");
        std::fs::create_dir_all(env2.downloads()).unwrap();
        let target = env2.downloads().join(
            server
                .url("/3xT/t.txt")
                .replace("http://", "")
                .replace([':', '/'], "_"),
        );
        std::fs::write(&target, &old).unwrap();
        let (ok, out, err) = env2.run(&["patterns", "update", "--force"], &[]);
        assert!(!ok, "{name}: {out}");
        assert!(out.contains("failed"), "{name}: {out}");
        assert!(err.contains("error:"), "{name}: {err}");
        assert_eq!(read(&target), old, "{name}: the old file is kept");
        assert!(!env2.files().iter().any(|n| n.ends_with(".part")), "{name}");
    }
}

#[test]
fn an_oversize_file_is_refused() {
    let server = Server::start(|_| response_announcing(60_000_000, b"R2"));
    let env = Env::new("big", &[server.url("/3xT/t.txt")], "");
    let (ok, out, _) = env.run(&["patterns", "update"], &[]);
    assert!(!ok);
    assert!(out.contains("limit"), "{out}");
    assert!(env.files().is_empty(), "{:?}", env.files());
}

#[test]
fn a_redirect_to_plain_http_is_refused() {
    let server = Server::start(|_| {
        response(
            "302 Found",
            &[("Location", "http://example.org/3xT/t.txt")],
            b"",
        )
    });
    let env = Env::new("redir", &[server.url("/3xT/t.txt")], "");
    let (ok, out, _) = env.run(&["patterns", "update"], &[]);
    assert!(!ok);
    assert!(out.contains("redirect") && out.contains("refused"), "{out}");
    assert!(env.files().is_empty());
}

#[test]
fn a_redirect_on_the_same_loopback_host_is_followed() {
    let server = Server::start(|request| {
        if request.path == "/3xT/t.txt" {
            response("301 Moved", &[("Location", "/3xT/real.txt")], b"")
        } else {
            response("200 OK", &[], table().as_bytes())
        }
    });
    let env = Env::new("redir-ok", &[server.url("/3xT/t.txt")], "");
    let (ok, out, err) = env.run(&["patterns", "update"], &[]);
    assert!(ok, "{out}{err}");
    assert_eq!(server.hits(), 2);
}

#[test]
fn plain_http_to_another_host_is_refused_without_connecting() {
    let env = Env::new("http", &["http://example.org/3xT/t.txt".to_owned()], "");
    let (ok, out, _) = env.run(&["patterns", "update"], &[]);
    assert!(!ok);
    assert!(out.contains("plain http is refused"), "{out}");
}

#[test]
fn offline_never_touches_the_network() {
    let server = plain_server();
    let url = server.url("/3xT/t.txt");
    let env = Env::new("offline", &[url], "auto_fetch = true");
    let (ok, out, _) = env.run(&["patterns", "--offline", "update"], &[]);
    assert!(!ok && out.contains("offline"), "{out}");
    let (ok, _, err) = env.run(&["patterns"], &[("CUBACLE_OFFLINE", "1")]);
    assert!(ok, "{err}");
    assert!(!err.contains("downloading"), "{err}");
    assert!(err.contains("not downloaded"), "{err}");
    let (ok, _, _) = env.run(&["patterns", "update"], &[("CUBACLE_OFFLINE", "1")]);
    assert!(!ok);
    assert_eq!(server.hits(), 0);
}

#[test]
fn auto_fetch_downloads_what_is_missing_and_only_then() {
    let server = plain_server();
    let env = Env::new("auto", &[server.url("/3xT/t.txt")], "auto_fetch = true");
    let (ok, out, err) = env.run(&["patterns", "--size", "3"], &[]);
    assert!(ok, "{err}");
    assert!(err.contains("downloading http://127.0.0.1:"), "{err}");
    assert!(out.contains("t1-01\tPair"), "{out}");
    let (ok, _, err) = env.run(&["patterns", "--size", "3"], &[]);
    assert!(ok && !err.contains("downloading"), "{err}");
    assert_eq!(server.hits(), 1);
}

#[test]
fn without_auto_fetch_nothing_is_downloaded() {
    let server = plain_server();
    let env = Env::new("noauto", &[server.url("/3xT/t.txt")], "auto_fetch = false");
    let (ok, _, err) = env.run(&["patterns"], &[]);
    assert!(ok && err.contains("not downloaded"), "{err}");
    assert_eq!(server.hits(), 0);
}

#[test]
fn a_failed_auto_fetch_warns_and_goes_on() {
    let server = Server::start(|_| response("500 Server Error", &[], b"oops"));
    let env = Env::new(
        "auto-fail",
        &[server.url("/3xT/t.txt")],
        "auto_fetch = true",
    );
    let (ok, out, err) = env.run(&["patterns"], &[]);
    assert!(ok, "{err}");
    assert!(out.contains("checkerboard"), "{out}");
    assert!(
        err.contains("warning:") && err.contains("HTTP status 500"),
        "{err}"
    );
}

#[test]
fn update_with_nothing_to_download_says_so() {
    let env = Env::new("none", &[], "");
    let (ok, out, _) = env.run(&["patterns", "update"], &[]);
    assert!(ok && out.contains("no http(s) sources"), "{out}");
}
