//! `cargo xtask wasm`: builds the browser module from a clean tree with fixed flags
//! and writes the manifest next to it.
//!
//! Output: `target/wasm-dist/cubacle_wasm.wasm` and
//! `target/wasm-dist/cubacle-wasm.manifest.json`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use sha2::{Digest, Sha256};

use crate::manifest::{FileEntry, Manifest};

/// The toolchain this build is pinned to (decision D5): `rustc -Vv` must name this
/// release and this commit. Change it on purpose, in a pull request of its own.
pub const PINNED_RELEASE: &str = "1.99.1";
pub const PINNED_COMMIT_HASH: &str = "58cf8f9c35b355199727627d07ae34a4e2846d67";

pub const MODULE: &str = "cubacle_wasm.wasm";
pub const MANIFEST: &str = "cubacle-wasm.manifest.json";

/// `git status --porcelain` output: empty means a clean tree (untracked files count).
pub fn check_clean(porcelain: &str) -> Result<(), String> {
    if porcelain.trim().is_empty() {
        Ok(())
    } else {
        Err(format!(
            "the working tree is not clean; commit or remove:\n{}",
            porcelain.trim_end()
        ))
    }
}

fn field<'a>(rustc_vv: &'a str, key: &str) -> Option<&'a str> {
    rustc_vv
        .lines()
        .find_map(|line| line.strip_prefix(key)?.strip_prefix(": "))
}

/// `rustc -Vv` output: refuses any release or commit other than the pinned one.
pub fn check_toolchain(rustc_vv: &str) -> Result<(), String> {
    let release = field(rustc_vv, "release");
    let hash = field(rustc_vv, "commit-hash");
    if release == Some(PINNED_RELEASE) && hash == Some(PINNED_COMMIT_HASH) {
        Ok(())
    } else {
        Err(format!(
            "toolchain mismatch: pinned rustc {PINNED_RELEASE} ({PINNED_COMMIT_HASH}), found {} ({})",
            release.unwrap_or("unknown"),
            hash.unwrap_or("unknown")
        ))
    }
}

/// The flags of the build, joined for `CARGO_ENCODED_RUSTFLAGS`. A plain `RUSTFLAGS`
/// or this variable replaces the target `rustflags` of `.cargo/config.toml`, so the
/// `getrandom` backend flag is repeated here.
pub fn encoded_rustflags(repo: &Path, cargo_home: &Path) -> String {
    [
        "--cfg".to_string(),
        "getrandom_backend=\"custom\"".to_string(),
        format!("--remap-path-prefix={}=/cubacle", repo.display()),
        format!("--remap-path-prefix={}=/cargo", cargo_home.display()),
    ]
    .join("\u{1f}")
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// `RUSTC` when it names a compiler, otherwise `rustc` on `PATH`.
/// Cargo uses the same variable, so the pin check has to follow it.
fn compiler_program(rustc_env: Option<&str>) -> &str {
    match rustc_env.map(str::trim).filter(|value| !value.is_empty()) {
        Some(program) => program,
        None => "rustc",
    }
}

fn commit_args() -> [&'static str; 2] {
    ["rev-parse", "HEAD"]
}

fn file_entry(bytes: &[u8]) -> FileEntry {
    FileEntry {
        bytes: bytes.len() as u64,
        sha256: hex(&Sha256::digest(bytes)),
    }
}

/// Arguments and environment of the `cargo build` this task runs.
///
/// `RUSTC` and `CARGO_BUILD_RUSTC` are the compiler that passed the pin check.
/// An empty `CARGO_BUILD_RUSTC_WRAPPER` overrides `build.rustc-wrapper` in a
/// Cargo config file. `RUSTC_WRAPPER` is a different variable and is removed,
/// because an empty config value does not clear it. The same applies to the
/// workspace wrapper. `RUSTFLAGS` is removed so it cannot replace the encoded flags.
#[derive(Debug)]
struct CargoInvocation {
    args: Vec<String>,
    env: Vec<(String, String)>,
    removed: &'static [&'static str],
}

impl CargoInvocation {
    #[cfg(test)]
    fn env(&self, key: &str) -> Option<&str> {
        self.env
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    }
}

fn cargo_invocation(target_dir: &Path, compiler: &str, encoded_rustflags: &str) -> CargoInvocation {
    CargoInvocation {
        args: vec![
            "build".into(),
            "--release".into(),
            "--locked".into(),
            "-p".into(),
            "cubacle-wasm".into(),
            "--target".into(),
            "wasm32-unknown-unknown".into(),
            "--target-dir".into(),
            target_dir.display().to_string(),
        ],
        env: vec![
            ("RUSTC".into(), compiler.into()),
            ("CARGO_BUILD_RUSTC".into(), compiler.into()),
            ("CARGO_ENCODED_RUSTFLAGS".into(), encoded_rustflags.into()),
            ("CARGO_BUILD_RUSTC_WRAPPER".into(), String::new()),
            ("CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER".into(), String::new()),
        ],
        removed: &["RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"],
    }
}

/// What `build` runs after the checks: the cargo command and the `rustc` value of the manifest.
#[derive(Debug)]
struct BuildPlan {
    invocation: CargoInvocation,
    target_dir: PathBuf,
    rustc: String,
}

/// Every step from the checks to the cargo command, without running anything:
/// porcelain check, compiler choice, pin check, cargo spec. `rustc_vv_of` runs
/// `<compiler> -Vv` for the compiler it is given.
fn plan_build(
    porcelain: &str,
    rustc_env: Option<&str>,
    rustc_vv_of: impl FnOnce(&str) -> Result<String, String>,
    target_dir: &Path,
    encoded_rustflags: &str,
) -> Result<BuildPlan, String> {
    check_clean(porcelain)?;
    let compiler = compiler_program(rustc_env);
    let rustc_vv = rustc_vv_of(compiler)?;
    check_toolchain(&rustc_vv)?;
    Ok(BuildPlan {
        invocation: cargo_invocation(target_dir, compiler, encoded_rustflags),
        target_dir: target_dir.to_path_buf(),
        rustc: rustc_vv.trim_end().to_string(),
    })
}

/// The `cargo` command with the invocation applied: variables set and removed.
fn cargo_command(invocation: &CargoInvocation, repo: &Path) -> Command {
    let mut command = Command::new("cargo");
    command.args(&invocation.args).current_dir(repo);
    for (key, value) in &invocation.env {
        command.env(key, value);
    }
    for key in invocation.removed {
        command.env_remove(key);
    }
    command
}

/// Where cargo leaves the module for the target dir it was given.
fn module_path(target_dir: &Path) -> PathBuf {
    target_dir
        .join("wasm32-unknown-unknown/release")
        .join(MODULE)
}

fn capture(program: &str, args: &[&str], dir: &Path) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|error| format!("cannot run {program}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{program} {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn package_version(repo: &Path) -> Result<String, String> {
    let text = fs::read_to_string(repo.join("crates/wasm/Cargo.toml"))
        .map_err(|error| format!("crates/wasm/Cargo.toml: {error}"))?;
    let table: toml::Table = text
        .parse()
        .map_err(|error| format!("crates/wasm/Cargo.toml: {error}"))?;
    table
        .get("package")
        .and_then(|package| package.get("version"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| "crates/wasm/Cargo.toml has no package version".to_string())
}

fn build(repo: &Path) -> Result<PathBuf, String> {
    let porcelain = capture(
        "git",
        &["status", "--porcelain", "--untracked-files=all"],
        repo,
    )?;
    let rustc_var = std::env::var("RUSTC").ok();
    let commit = capture("git", &commit_args(), repo)?.trim().to_string();
    let version = package_version(repo)?;
    let cargo_home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")))
        .ok_or("neither CARGO_HOME nor HOME is set")?;

    let target_dir = repo.join("target/wasm-build");
    let plan = plan_build(
        &porcelain,
        rustc_var.as_deref(),
        |compiler| capture(compiler, &["-Vv"], repo),
        &target_dir,
        &encoded_rustflags(repo, &cargo_home),
    )?;
    let status = cargo_command(&plan.invocation, repo)
        .status()
        .map_err(|error| format!("cannot run cargo build: {error}"))?;
    if !status.success() {
        return Err("cargo build failed".to_string());
    }

    let dist = repo.join("target/wasm-dist");
    fs::create_dir_all(&dist).map_err(|error| error.to_string())?;
    let module = fs::read(module_path(&plan.target_dir))
    .map_err(|error| format!("{MODULE}: {error}"))?;
    fs::write(dist.join(MODULE), &module).map_err(|error| error.to_string())?;
    let mut files = BTreeMap::new();
    files.insert(MODULE.to_string(), file_entry(&module));
    let manifest = Manifest {
        version,
        commit,
        rustc: plan.rustc,
        files,
        table_sha256: BTreeMap::new(),
    };
    fs::write(dist.join(MANIFEST), manifest.to_json()).map_err(|error| error.to_string())?;
    Ok(dist)
}

pub fn run() -> ExitCode {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repository root");
    match build(repo) {
        Ok(dist) => {
            println!("wasm: wrote {}", dist.display());
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("wasm: {message}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vv(release: &str, hash: &str) -> String {
        format!(
            "rustc {release}\nbinary: rustc\ncommit-hash: {hash}\nhost: x\nrelease: {release}\n"
        )
    }

    #[test]
    fn a_dirty_tree_is_refused() {
        assert!(check_clean("").is_ok());
        assert!(check_clean(" M src/lib.rs\n")
            .unwrap_err()
            .contains("not clean"));
        assert!(check_clean("?? new.txt\n").is_err());
    }

    #[test]
    fn the_pinned_toolchain_passes_and_another_is_refused() {
        assert!(check_toolchain(&vv(PINNED_RELEASE, PINNED_COMMIT_HASH)).is_ok());
        let other_release = check_toolchain(&vv("1.98.0", PINNED_COMMIT_HASH)).unwrap_err();
        assert!(other_release.contains("toolchain mismatch"));
        assert!(check_toolchain(&vv(PINNED_RELEASE, "0".repeat(40).as_str())).is_err());
        assert!(check_toolchain("garbage").is_err());
    }

    #[test]
    fn the_flags_keep_the_getrandom_backend_and_remap_both_roots() {
        let flags = encoded_rustflags(Path::new("/r"), Path::new("/h/.cargo"));
        let parts: Vec<&str> = flags.split('\u{1f}').collect();
        assert_eq!(parts[0], "--cfg");
        assert_eq!(parts[1], "getrandom_backend=\"custom\"");
        assert_eq!(parts[2], "--remap-path-prefix=/r=/cubacle");
        assert_eq!(parts[3], "--remap-path-prefix=/h/.cargo=/cargo");
    }

    #[test]
    fn hex_is_lowercase_and_padded() {
        assert_eq!(hex(&[0, 15, 255]), "000fff");
    }

    #[test]
    fn the_pin_check_uses_rustc_from_the_environment_when_it_is_set() {
        assert_eq!(compiler_program(None), "rustc");
        assert_eq!(compiler_program(Some("")), "rustc");
        assert_eq!(compiler_program(Some("  ")), "rustc");
        assert_eq!(
            compiler_program(Some("/opt/pinned/rustc")),
            "/opt/pinned/rustc"
        );
    }

    #[test]
    fn the_cargo_invocation_locks_the_checked_compiler_and_drops_wrappers() {
        let invocation = cargo_invocation(
            Path::new("target/wasm-build"),
            "/opt/pinned/rustc",
            "--cfg\u{1f}getrandom_backend=\"custom\"",
        );
        assert!(
            invocation.args.iter().any(|arg| arg == "--locked"),
            "{invocation:?}"
        );
        assert!(
            invocation
                .args
                .windows(2)
                .any(|pair| { pair[0] == "--target-dir" && pair[1] == "target/wasm-build" }),
            "{invocation:?}"
        );
        assert_eq!(invocation.env("RUSTC"), Some("/opt/pinned/rustc"));
        assert_eq!(
            invocation.env("CARGO_BUILD_RUSTC"),
            Some("/opt/pinned/rustc")
        );
        assert_eq!(
            invocation.env("CARGO_ENCODED_RUSTFLAGS"),
            Some("--cfg\u{1f}getrandom_backend=\"custom\"")
        );
        assert_eq!(invocation.env("CARGO_BUILD_RUSTC_WRAPPER"), Some(""));
        assert_eq!(
            invocation.env("CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER"),
            Some("")
        );
        assert!(invocation.env("RUSTFLAGS").is_none(), "{invocation:?}");
        for name in ["RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"] {
            assert!(
                invocation.removed.contains(&name),
                "missing {name} in {invocation:?}"
            );
        }
    }

    fn plan(
        porcelain: &str,
        rustc_env: Option<&str>,
        reply: &str,
        asked: &std::cell::RefCell<Vec<String>>,
    ) -> Result<BuildPlan, String> {
        plan_build(
            porcelain,
            rustc_env,
            |compiler| {
                asked.borrow_mut().push(compiler.to_string());
                Ok(reply.to_string())
            },
            Path::new("t"),
            "flags",
        )
    }

    #[test]
    fn the_plan_asks_and_runs_the_environment_compiler() {
        let asked = std::cell::RefCell::new(Vec::new());
        let reply = vv(PINNED_RELEASE, PINNED_COMMIT_HASH);
        let plan = plan("", Some("/opt/pinned/rustc"), &reply, &asked).unwrap();
        assert_eq!(*asked.borrow(), ["/opt/pinned/rustc"]);
        assert_eq!(plan.invocation.env("RUSTC"), Some("/opt/pinned/rustc"));
        assert_eq!(
            plan.invocation.env("CARGO_BUILD_RUSTC"),
            Some("/opt/pinned/rustc")
        );
        assert_eq!(
            plan.invocation.env("CARGO_ENCODED_RUSTFLAGS"),
            Some("flags")
        );
        assert_eq!(plan.rustc, reply.trim_end());
        assert!(!plan.rustc.is_empty());
        let asked = std::cell::RefCell::new(Vec::new());
        let plan = self::plan("", None, &reply, &asked).unwrap();
        assert_eq!(*asked.borrow(), ["rustc"]);
        assert_eq!(plan.invocation.env("RUSTC"), Some("rustc"));
    }

    #[test]
    fn the_plan_refuses_a_dirty_tree_and_a_pin_mismatch_before_cargo() {
        let asked = std::cell::RefCell::new(Vec::new());
        let reply = vv(PINNED_RELEASE, PINNED_COMMIT_HASH);
        let dirty = plan("?? new.txt\n", None, &reply, &asked).unwrap_err();
        assert!(dirty.contains("not clean"), "{dirty}");
        assert!(asked.borrow().is_empty(), "compiler asked on a dirty tree");
        let other = plan("", None, &vv("1.98.0", PINNED_COMMIT_HASH), &asked).unwrap_err();
        assert!(other.contains("toolchain mismatch"), "{other}");
        let other_hash = plan("", None, &vv(PINNED_RELEASE, &"0".repeat(40)), &asked);
        assert!(other_hash.is_err());
    }

    #[test]
    fn the_plan_passes_a_failing_rustc_on() {
        let result = plan_build(
            "",
            None,
            |_| Err("cannot run rustc".to_string()),
            Path::new("t"),
            "flags",
        );
        assert_eq!(result.unwrap_err(), "cannot run rustc");
    }

    #[test]
    fn the_cargo_command_sets_the_variables_and_removes_the_others() {
        let invocation = cargo_invocation(Path::new("t"), "/opt/pinned/rustc", "flags");
        let command = cargo_command(&invocation, Path::new("/r"));
        assert_eq!(command.get_program(), "cargo");
        assert_eq!(command.get_current_dir(), Some(Path::new("/r")));
        let args: Vec<_> = command.get_args().collect();
        assert_eq!(args.len(), invocation.args.len());
        assert_eq!(args[0], "build");
        let envs: Vec<_> = command.get_envs().collect();
        for name in ["RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"] {
            assert!(
                envs.iter()
                    .any(|(key, value)| *key == name && value.is_none()),
                "{name} not removed: {envs:?}"
            );
        }
        assert!(envs.iter().any(|(key, value)| {
            *key == "RUSTC" && *value == Some(std::ffi::OsStr::new("/opt/pinned/rustc"))
        }));
        assert!(envs.iter().any(|(key, value)| {
            *key == "CARGO_BUILD_RUSTC_WRAPPER" && *value == Some(std::ffi::OsStr::new(""))
        }));
    }

    #[test]
    fn the_commit_comes_from_head() {
        assert_eq!(commit_args(), ["rev-parse", "HEAD"]);
    }

    #[test]
    fn the_file_entry_is_the_length_and_sha256_of_those_bytes() {
        let entry = file_entry(b"abc");
        assert_eq!(entry.bytes, 3);
        assert_eq!(
            entry.sha256,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
