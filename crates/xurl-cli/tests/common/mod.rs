//! Helpers shared across the integration suite: the `xr` spawn seam and the
//! guard tests' source scanner. Each test crate uses a subset.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use assert_cmd::Command;

/// Store path for spawns that never touch credentials. Its parent directory
/// does not exist, so a read loads an empty store and a write fails loudly
/// instead of landing in a file another test could see.
fn unwritable_store() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(concat!(env!("CARGO_CRATE_NAME"), "-no-store"))
        .join(".xurl")
}

/// The built `xr` binary with `XURL_TOKEN_STORE` pointed at an unwritable
/// scratch path. Use [`xr_with_store`] when the test reads or writes a store.
pub fn xr() -> Command {
    xr_with_store(&unwritable_store())
}

/// The built `xr` binary with `XURL_TOKEN_STORE` set to `store`.
pub fn xr_with_store(store: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xr"));
    hermetic(&mut cmd, store);
    cmd
}

/// [`xr`] as a `std::process::Command`, for a test that needs the child
/// handle itself (piped stdio, signals).
pub fn xr_std() -> std::process::Command {
    xr_std_at(xr_bin())
}

/// [`xr_std`] for the binary at `program`, so a harness that compares builds
/// keeps the same isolation for whichever `xr` it runs.
pub fn xr_std_at(program: &str) -> std::process::Command {
    xr_std_with_store_at(program, &unwritable_store())
}

/// [`xr_std_at`] with `XURL_TOKEN_STORE` set to `store`, for a harness that
/// seeds a store and still chooses which `xr` build it runs.
pub fn xr_std_with_store_at(program: &str, store: &Path) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    hermetic(&mut cmd, store);
    cmd
}

/// Path of the built `xr` binary, for a harness that spawns it by path
/// through [`xr_std_at`].
pub fn xr_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xr")
}

/// Strips every variable `xr` reads from the inherited environment, then
/// points the child at `store`, so a test sees only what it sets itself.
fn hermetic<C: EnvBuilder>(cmd: &mut C, store: &Path) {
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("XURL_") {
            cmd.remove(&key);
        }
    }
    for key in [
        "CLIENT_ID",
        "CLIENT_SECRET",
        "REDIRECT_URI",
        "AUTH_URL",
        "TOKEN_URL",
        "API_BASE_URL",
        "INFO_URL",
        "NO_COLOR",
    ] {
        cmd.remove(std::ffi::OsStr::new(key));
    }
    cmd.set("XURL_TOKEN_STORE", store);
}

/// The two command types share the environment-editing surface the seam needs.
trait EnvBuilder {
    fn remove(&mut self, key: &std::ffi::OsStr);
    fn set(&mut self, key: &str, value: &Path);
}

impl EnvBuilder for Command {
    fn remove(&mut self, key: &std::ffi::OsStr) {
        self.env_remove(key);
    }
    fn set(&mut self, key: &str, value: &Path) {
        self.env(key, value);
    }
}

impl EnvBuilder for std::process::Command {
    fn remove(&mut self, key: &std::ffi::OsStr) {
        self.env_remove(key);
    }
    fn set(&mut self, key: &str, value: &Path) {
        self.env(key, value);
    }
}

/// The workspace root, from the `[env]` entry in `.cargo/config.toml`.
/// Repo-root assets (`schema/`, `scripts/`) and the other member's tree are
/// reached from here, never from `CARGO_MANIFEST_DIR`.
pub fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_WORKSPACE_DIR"))
}

/// Every member crate directory under `crates/`, sorted.
pub fn member_dirs() -> Vec<PathBuf> {
    let crates = workspace_root().join("crates");
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&crates)
        .expect("crates/ must be readable")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();
    dirs
}

/// Every `.rs` file under `dir`, recursively, sorted.
pub fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{} must be readable: {e}", dir.display()))
        {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

/// Every `.rs` file under each member's `src/`: the whole production tree
/// a source-scanning guard must cover, so neither crate escapes it.
pub fn workspace_sources() -> Vec<PathBuf> {
    member_dirs()
        .iter()
        .flat_map(|member| rust_files(&member.join("src")))
        .collect()
}

/// A test a source-scanning guard exempts, with the reason it is exempt.
/// `file` is workspace-relative.
pub struct Allowed {
    pub file: &'static str,
    pub test: &'static str,
    pub reason: &'static str,
}

/// The entries of `allowlist` naming a test that no longer exists, rendered
/// for the assertion message. A stale exemption silently widens what a guard
/// permits, so every guard checks its own allowlist through here.
pub fn stale_allowlist_entries(allowlist: &[Allowed]) -> Vec<String> {
    allowlist
        .iter()
        .filter(|entry| {
            let path = workspace_root().join(entry.file);
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            !source.contains(&format!("fn {}(", entry.test))
        })
        .map(|entry| format!("{}::{} ({})", entry.file, entry.test, entry.reason))
        .collect()
}

/// Returns the name of the `fn` a given byte offset falls inside.
pub fn enclosing_test(source: &str, offset: usize) -> String {
    source[..offset]
        .rmatch_indices("fn ")
        .find_map(|(i, _)| {
            let rest = &source[i + 3..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            (!name.is_empty()).then_some(name)
        })
        .unwrap_or_else(|| "<unknown>".to_string())
}
