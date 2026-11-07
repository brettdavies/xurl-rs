//! `xr skill update` replaces only what `xr skill install` wrote, and only
//! once the clone that replaces it exists.
//!
//! Every run points `XURL_SKILL_HOME` at a scratch directory and puts a `git`
//! shim first on `PATH`, so nothing reaches the network. The shim is a
//! `#!/bin/sh` script, and on Windows `Command::new("git")` runs only a
//! `git.exe`, so the file is Unix-only.

#![cfg(unix)]

mod common;

use std::collections::BTreeMap;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use serde_json::Value;
use tempfile::TempDir;

const HOST: &str = "claude_code";
const LEGACY_HOST: &str = "codex";

/// A `git` that refuses every clone with a fixed message.
const REFUSING_GIT: &str = "echo 'fatal: refused by the test' >&2\nexit 128\n";
/// A `git` that exits 0 and writes nothing, so the clone is never there to
/// swap in.
const EMPTY_GIT: &str = "exit 0\n";
/// A `git` that writes what a real clone writes where the marker is concerned:
/// `origin` as the URL it was given, in the target's `.git/config`.
const CLONING_GIT: &str = r#"mkdir -p "$target/.git"
printf '[remote "origin"]\n\turl = %s\n' "$url" > "$target/.git/config"
echo fresh > "$target/SKILL.md"
"#;

fn skill_manifest() -> Value {
    let path = common::workspace_root().join("crates/xurl-cli/src/cli/skill_install/skill.json");
    let text = std::fs::read_to_string(&path).expect("skill.json must be readable");
    serde_json::from_str(&text).expect("skill.json must parse")
}

fn bundle_url() -> String {
    skill_manifest()["source"]["url"]
        .as_str()
        .expect("source.url")
        .to_string()
}

/// A `.git/config` of the shape `git clone --depth 1` writes.
fn git_config(origin: &str) -> String {
    format!(
        "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\
         \tlogallrefupdates = true\n[remote \"origin\"]\n\turl = {origin}\n\
         \tfetch = +refs/heads/main:refs/remotes/origin/main\n[branch \"main\"]\n\
         \tremote = origin\n\tmerge = refs/heads/main\n"
    )
}

/// What a clone of `origin` leaves at `dir`, with a file the clone did not
/// bring.
fn write_clone_of(dir: &Path, origin: &str) {
    std::fs::create_dir_all(dir.join(".git")).expect("create the clone");
    std::fs::write(dir.join(".git").join("config"), git_config(origin)).expect("git config");
    std::fs::write(dir.join("SKILL.md"), "---\nname: xurl-rs\n---\nold\n").expect("SKILL.md");
    std::fs::write(dir.join("kept.txt"), "kept").expect("kept file");
}

fn write_bundle(dir: &Path) {
    write_clone_of(dir, &bundle_url());
}

/// Every entry under `dir`, by relative path: a file's bytes, a directory's
/// marker, a link's target.
fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).expect("read_dir") {
            let path = entry.expect("entry").path();
            let name = path
                .strip_prefix(root)
                .expect("under the root")
                .display()
                .to_string();
            let kind = std::fs::symlink_metadata(&path).expect("lstat");
            if kind.is_symlink() {
                let target = std::fs::read_link(&path).expect("readlink");
                out.insert(name, format!("-> {}", target.display()).into_bytes());
            } else if kind.is_dir() {
                out.insert(name, b"<dir>".to_vec());
                walk(root, &path, out);
            } else {
                out.insert(name, std::fs::read(&path).expect("read"));
            }
        }
    }
    let mut out = BTreeMap::new();
    if dir.exists() {
        walk(dir, dir, &mut out);
    }
    out
}

/// The names beside `dir`, so a staging directory left behind shows.
fn siblings(dir: &Path) -> Vec<String> {
    let parent = dir.parent().expect("a parent");
    let mut names: Vec<String> = std::fs::read_dir(parent)
        .expect("read_dir")
        .map(|entry| entry.expect("entry").file_name().to_string_lossy().into())
        .collect();
    names.sort();
    names
}

/// A scratch skill home and a directory holding the `git` shim.
struct Home {
    dir: TempDir,
    shim: TempDir,
}

impl Home {
    /// The shim runs `body` with `$url` and `$target` set from its last two
    /// arguments, after logging the target and whether `$PROBE` exists.
    fn new(body: &str) -> Self {
        let shim = TempDir::new().expect("tempdir");
        let git = shim.path().join("git");
        let script = format!(
            "#!/bin/sh\nfor arg; do url=$target; target=$arg; done\n\
             echo \"$target\" >> \"$(dirname \"$0\")/clones\"\n\
             if [ -n \"$PROBE\" ] && [ -e \"$PROBE\" ]; then touch \"$(dirname \"$0\")/probe-found\"; fi\n\
             {body}"
        );
        std::fs::write(&git, script).expect("git shim");
        std::fs::set_permissions(&git, std::fs::Permissions::from_mode(0o755)).expect("executable");
        Self {
            dir: TempDir::new().expect("tempdir"),
            shim,
        }
    }

    fn path_var(&self) -> std::ffi::OsString {
        let inherited = std::env::var_os("PATH").unwrap_or_default();
        std::env::join_paths(
            std::iter::once(self.shim.path().to_path_buf())
                .chain(std::env::split_paths(&inherited)),
        )
        .expect("PATH")
    }

    fn dest(&self, host: &str) -> PathBuf {
        let manifest = skill_manifest();
        let template = manifest["install"][host]
            .as_str()
            .expect("install command")
            .split_whitespace()
            .last()
            .expect("a destination");
        self.dir
            .path()
            .join(template.strip_prefix("~/").expect("a ~/ template"))
    }

    fn legacy(&self, host: &str) -> PathBuf {
        let manifest = skill_manifest();
        let template = manifest["legacy_destinations"][host]["path"]
            .as_str()
            .expect("a legacy path");
        self.dir
            .path()
            .join(template.strip_prefix("~/").expect("a ~/ path"))
    }

    fn run_raw(&self, args: &[&str], probe: Option<&Path>) -> std::process::Output {
        let mut cmd = common::xr();
        cmd.env("XURL_SKILL_HOME", self.dir.path())
            .env("PATH", self.path_var());
        if let Some(probe) = probe {
            cmd.env("PROBE", probe);
        }
        cmd.args(args).output().expect("run xr")
    }

    fn update(&self, host: &str) -> (i32, Value) {
        self.update_with(&["skill", "update", host, "--output", "json"], None)
    }

    fn dry_run(&self, host: &str) -> (i32, Value) {
        self.update_with(
            &["skill", "update", host, "--dry-run", "--output", "json"],
            None,
        )
    }

    fn update_with(&self, args: &[&str], probe: Option<&Path>) -> (i32, Value) {
        let out = self.run_raw(args, probe);
        let value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
            panic!(
                "stdout is not a JSON envelope ({e}): {}",
                String::from_utf8_lossy(&out.stdout)
            )
        });
        (out.status.code().expect("an exit code"), value)
    }

    /// The target of each clone the shim was asked for.
    fn clones(&self) -> Vec<PathBuf> {
        std::fs::read_to_string(self.shim.path().join("clones"))
            .unwrap_or_default()
            .lines()
            .map(PathBuf::from)
            .collect()
    }

    fn probe_found(&self) -> bool {
        self.shim.path().join("probe-found").exists()
    }
}

#[test]
fn a_failed_clone_leaves_the_bundle_exactly_as_it_was() {
    let home = Home::new(REFUSING_GIT);
    let dest = home.dest(HOST);
    write_bundle(&dest);
    let before = tree(&dest);

    let (code, v) = home.update(HOST);

    assert_eq!(code, 128, "{v}");
    assert_eq!(v["reason"], "git-clone-failed", "{v}");
    assert_eq!(v["next_step"]["action"], "retry", "{v}");
    assert_eq!(tree(&dest), before, "the failed clone touched the bundle");
    assert_eq!(siblings(&dest), ["xurl-rs"], "staging is cleaned up");
}

#[test]
fn a_failed_clone_leaves_the_legacy_copy_in_place() {
    let home = Home::new(REFUSING_GIT);
    let legacy = home.legacy(LEGACY_HOST);
    write_bundle(&legacy);
    let before = tree(&legacy);

    let (code, v) = home.update(LEGACY_HOST);

    assert_eq!(code, 128, "{v}");
    assert_eq!(v["reason"], "git-clone-failed", "{v}");
    assert_eq!(
        v["legacy_install_dir"].as_str(),
        Some(legacy.to_string_lossy().as_ref())
    );
    assert_eq!(tree(&legacy), before, "the failed clone removed the copy");
}

#[test]
fn an_existing_bundle_is_replaced_once_its_clone_exists() {
    let home = Home::new(CLONING_GIT);
    let dest = home.dest(HOST);
    write_bundle(&dest);

    let (code, v) = home.update_with(
        &["skill", "update", HOST, "--output", "json"],
        Some(&dest.join("kept.txt")),
    );

    assert_eq!(code, 0, "{v}");
    assert_eq!(v["status"], "ok", "{v}");
    assert_eq!(v["action"], "skill-update", "{v}");
    assert_eq!(v["destination_status"], "non-empty-dir", "{v}");

    assert!(
        home.probe_found(),
        "the old bundle must still be there while the clone runs"
    );
    let clones = home.clones();
    assert_eq!(clones.len(), 1, "{clones:?}");
    assert!(
        clones[0].starts_with(dest.parent().expect("a parent")) && !clones[0].starts_with(&dest),
        "the clone goes beside the destination, not into it: {clones:?}"
    );

    let after = tree(&dest);
    assert_eq!(
        after.get("SKILL.md").map(Vec::as_slice),
        Some(&b"fresh\n"[..])
    );
    assert!(!after.contains_key("kept.txt"), "{after:?}");
    assert_eq!(siblings(&dest), ["xurl-rs"], "staging is cleaned up");

    let (code, v) = home.update(HOST);
    assert_eq!((code, &v["status"]), (0, &"ok".into()), "{v}");
}

/// A person who cloned the bundle by hand has the real bundle, whichever form
/// of the URL they typed.
#[rstest::rstest]
#[case::https_without_the_suffix("https://github.com/brettdavies/xurl-rs-skill")]
#[case::scp_form("git@github.com:brettdavies/xurl-rs-skill.git")]
#[case::scp_form_without_the_suffix("git@github.com:brettdavies/xurl-rs-skill")]
#[case::ssh_form("ssh://git@github.com/brettdavies/xurl-rs-skill.git")]
fn a_hand_made_clone_of_the_bundle_is_replaced(#[case] origin: &str) {
    let home = Home::new(CLONING_GIT);
    let dest = home.dest(HOST);
    write_clone_of(&dest, origin);

    let (code, v) = home.update(HOST);

    assert_eq!(code, 0, "{v}");
    assert_eq!(v["status"], "ok", "{v}");
    assert!(!tree(&dest).contains_key("kept.txt"));
    assert!(
        !home.clones()[0].starts_with(&dest),
        "the clone goes beside the destination"
    );
}

#[test]
fn a_clone_that_cannot_be_swapped_in_puts_the_bundle_back() {
    let home = Home::new(EMPTY_GIT);
    let dest = home.dest(HOST);
    write_bundle(&dest);
    let before = tree(&dest);

    let (code, v) = home.update(HOST);

    assert_eq!(code, 1, "{v}");
    assert_eq!(v["reason"], "remove-failed", "{v}");
    assert_eq!(v["next_step"]["action"], "fix-input", "{v}");
    assert_eq!(tree(&dest), before, "the bundle is back where it was");
    assert_eq!(siblings(&dest), ["xurl-rs"], "staging is cleaned up");
}

/// What sits at the destination in each case below: anything `skill install`
/// did not write.
fn plain_files(dir: &Path) {
    std::fs::create_dir_all(dir).expect("dir");
    std::fs::write(dir.join("notes.md"), "mine").expect("file");
}

fn clone_of_another_repo(dir: &Path) {
    write_clone_of(dir, "https://github.com/someone/other-skill.git");
}

fn clone_of_a_fork(dir: &Path) {
    write_clone_of(dir, "https://github.com/someone/xurl-rs-skill.git");
}

fn clone_of_the_source_repo(dir: &Path) {
    write_clone_of(dir, "git@github.com:brettdavies/xurl-rs.git");
}

fn skill_named_xurl_rs_without_a_clone(dir: &Path) {
    std::fs::create_dir_all(dir).expect("dir");
    std::fs::write(dir.join("SKILL.md"), "---\nname: xurl-rs\n---\nmine\n").expect("file");
}

fn worktree_of_the_bundle(dir: &Path) {
    std::fs::create_dir_all(dir).expect("dir");
    std::fs::write(dir.join(".git"), "gitdir: /elsewhere/.git/worktrees/x\n").expect("file");
    std::fs::write(dir.join("SKILL.md"), "mine").expect("file");
}

#[rstest::rstest]
#[case::plain_files(plain_files)]
#[case::another_repo(clone_of_another_repo)]
#[case::fork(clone_of_a_fork)]
#[case::source_repo(clone_of_the_source_repo)]
#[case::skill_without_a_clone(skill_named_xurl_rs_without_a_clone)]
#[case::worktree(worktree_of_the_bundle)]
fn a_directory_that_is_not_a_bundle_is_refused_and_left_untouched(#[case] occupy: fn(&Path)) {
    let home = Home::new(CLONING_GIT);
    let dest = home.dest(HOST);
    occupy(&dest);
    let before = tree(&dest);

    let (code, v) = home.update(HOST);

    assert_eq!(code, 1, "{v}");
    assert_eq!(v["status"], "error", "{v}");
    assert_eq!(v["reason"], "destination-not-empty", "{v}");
    assert_eq!(v["destination_status"], "non-empty-dir", "{v}");
    assert_eq!(v["next_step"]["action"], "fix-input", "{v}");
    assert_eq!(v["next_step"].get("command"), None, "{v}");
    assert_eq!(
        v["refused_dir"].as_str(),
        Some(dest.to_string_lossy().as_ref()),
        "{v}"
    );
    let message = v["message"].as_str().expect("a message");
    assert!(message.contains(&*dest.to_string_lossy()), "{message}");
    assert_eq!(tree(&dest), before, "the refusal touched the directory");
    assert!(home.clones().is_empty(), "a refusal never clones");
    assert_eq!(siblings(&dest), ["xurl-rs"]);
}

#[test]
fn a_file_at_the_destination_is_refused_and_left_untouched() {
    let home = Home::new(CLONING_GIT);
    let dest = home.dest(HOST);
    std::fs::create_dir_all(dest.parent().expect("a parent")).expect("parent");
    std::fs::write(&dest, "mine").expect("file");

    let (code, v) = home.update(HOST);

    assert_eq!(code, 1, "{v}");
    assert_eq!(v["reason"], "destination-is-file", "{v}");
    assert_eq!(v["destination_status"], "file", "{v}");
    assert_eq!(v["next_step"]["action"], "fix-input", "{v}");
    assert_eq!(std::fs::read_to_string(&dest).expect("file"), "mine");
    assert!(home.clones().is_empty(), "a refusal never clones");
}

/// `skill install` never writes a link, so one is someone's wiring even when
/// it leads to a bundle.
#[test]
fn a_link_at_the_destination_is_refused_even_when_it_leads_to_a_bundle() {
    let home = Home::new(CLONING_GIT);
    let dest = home.dest(HOST);
    let elsewhere = home.dir.path().join("checkout");
    write_bundle(&elsewhere);
    std::fs::create_dir_all(dest.parent().expect("a parent")).expect("parent");
    symlink(&elsewhere, &dest).expect("symlink");
    let before = tree(&elsewhere);

    let (code, v) = home.update(HOST);

    assert_eq!(code, 1, "{v}");
    assert_eq!(v["reason"], "destination-not-empty", "{v}");
    assert_eq!(v["next_step"]["action"], "fix-input", "{v}");
    assert_eq!(std::fs::read_link(&dest).expect("still a link"), elsewhere);
    assert_eq!(tree(&elsewhere), before, "the link's target is untouched");
    assert!(home.clones().is_empty(), "a refusal never clones");
}

#[test]
fn a_dangling_link_at_the_destination_is_refused() {
    let home = Home::new(CLONING_GIT);
    let dest = home.dest(HOST);
    let gone = home.dir.path().join("gone");
    std::fs::create_dir_all(dest.parent().expect("a parent")).expect("parent");
    symlink(&gone, &dest).expect("symlink");

    let (code, v) = home.update(HOST);

    assert_eq!(code, 1, "{v}");
    assert_eq!(v["reason"], "destination-is-file", "{v}");
    assert_eq!(std::fs::read_link(&dest).expect("still a link"), gone);
    assert!(home.clones().is_empty(), "a refusal never clones");
}

#[test]
fn a_legacy_copy_that_is_not_a_bundle_is_refused_before_anything_moves() {
    let home = Home::new(CLONING_GIT);
    let dest = home.dest(LEGACY_HOST);
    let legacy = home.legacy(LEGACY_HOST);
    write_bundle(&dest);
    plain_files(&legacy);
    let (dest_before, legacy_before) = (tree(&dest), tree(&legacy));

    let (code, v) = home.update(LEGACY_HOST);

    assert_eq!(code, 1, "{v}");
    assert_eq!(v["reason"], "destination-not-empty", "{v}");
    assert_eq!(v["next_step"]["action"], "fix-input", "{v}");
    assert_eq!(
        v["refused_dir"].as_str(),
        Some(legacy.to_string_lossy().as_ref()),
        "the legacy copy is what blocks: {v}"
    );
    assert_eq!(tree(&dest), dest_before);
    assert_eq!(tree(&legacy), legacy_before);
    assert!(home.clones().is_empty(), "a refusal never clones");
}

#[test]
fn a_legacy_bundle_is_removed_once_the_clone_is_in_place() {
    let home = Home::new(CLONING_GIT);
    let dest = home.dest(LEGACY_HOST);
    let legacy = home.legacy(LEGACY_HOST);
    write_bundle(&legacy);

    let (code, v) = home.update(LEGACY_HOST);

    assert_eq!(code, 0, "{v}");
    assert_eq!(v["status"], "ok", "{v}");
    assert_eq!(
        v["legacy_install_dir"].as_str(),
        Some(legacy.to_string_lossy().as_ref())
    );
    assert!(!legacy.exists(), "the copy is gone");
    assert!(dest.join("SKILL.md").exists(), "the clone is in place");
}

#[test]
fn update_all_refuses_one_host_and_refreshes_the_next() {
    let home = Home::new(CLONING_GIT);
    let refused = home.dest(HOST);
    let refreshed = home.dest("cursor");
    plain_files(&refused);
    write_bundle(&refreshed);
    let before = tree(&refused);

    let (code, v) = home.update_with(&["skill", "update", "--all", "--output", "json"], None);

    assert_eq!(code, 1, "{v}");
    assert_eq!(v["status"], "error", "{v}");
    let entry = |host: &str| {
        v["installations"]
            .as_array()
            .expect("installations")
            .iter()
            .find(|entry| entry["host"] == host)
            .expect("the host appears")
            .clone()
    };
    assert_eq!(entry(HOST)["reason"], "destination-not-empty", "{v}");
    assert_eq!(entry(HOST)["next_step"]["action"], "fix-input", "{v}");
    assert_eq!(entry("cursor")["status"], "ok", "{v}");
    assert_eq!(tree(&refused), before, "the refused host is untouched");
    assert!(!tree(&refreshed).contains_key("kept.txt"));
}

fn absent(_: &Home) {}

fn empty_dir(home: &Home) {
    std::fs::create_dir_all(home.dest(HOST)).expect("dir");
}

fn bundle(home: &Home) {
    write_bundle(&home.dest(HOST));
}

fn not_a_bundle(home: &Home) {
    plain_files(&home.dest(HOST));
}

fn a_file(home: &Home) {
    let dest = home.dest(HOST);
    std::fs::create_dir_all(dest.parent().expect("a parent")).expect("parent");
    std::fs::write(dest, "mine").expect("file");
}

fn legacy_bundle(home: &Home) {
    write_bundle(&home.legacy(LEGACY_HOST));
}

fn legacy_not_a_bundle(home: &Home) {
    plain_files(&home.legacy(LEGACY_HOST));
}

/// Each destination and legacy state, with the answer a dry run owes for it.
/// A dry run ends at exit 0 either way, as on every other command, and reports
/// in `exit_code` the code the real run ends on.
#[rstest::rstest]
#[case::absent(HOST, absent, true)]
#[case::empty_dir(HOST, empty_dir, true)]
#[case::bundle(HOST, bundle, true)]
#[case::not_a_bundle(HOST, not_a_bundle, false)]
#[case::file(HOST, a_file, false)]
#[case::legacy_bundle(LEGACY_HOST, legacy_bundle, true)]
#[case::legacy_not_a_bundle(LEGACY_HOST, legacy_not_a_bundle, false)]
fn the_dry_run_answers_what_the_real_run_does_and_touches_nothing(
    #[case] host: &str,
    #[case] arrange: fn(&Home),
    #[case] would_succeed: bool,
) {
    let planned = Home::new(CLONING_GIT);
    arrange(&planned);
    let before = tree(planned.dir.path());
    let (dry_code, dry) = planned.dry_run(host);
    assert_eq!(tree(planned.dir.path()), before, "the dry run wrote");
    assert!(planned.clones().is_empty(), "the dry run cloned");
    assert_eq!(dry_code, 0, "a dry run answers at exit 0: {dry}");
    assert_eq!(dry["status"], "dry_run", "{dry}");
    assert_eq!(dry["would_succeed"], would_succeed, "{dry}");

    let ran = Home::new(CLONING_GIT);
    arrange(&ran);
    let (real_code, real) = ran.update(host);
    assert_eq!(
        real["status"] == "ok",
        would_succeed,
        "the real run disagrees with the plan: {real}"
    );
    assert_eq!(dry["exit_code"], real_code, "{dry} / {real}");

    let strip = |envelope: &Value, home: &Home| {
        let text = envelope.to_string();
        text.replace(&*home.dir.path().to_string_lossy(), "<home>")
    };
    for key in [
        "action",
        "host",
        "exit_code",
        "reason",
        "next_step",
        "destination_status",
    ] {
        assert_eq!(dry[key], real[key], "{key}: {dry} / {real}");
    }
    for key in [
        "install_dir",
        "legacy_install_dir",
        "refused_dir",
        "message",
    ] {
        assert_eq!(
            strip(&dry[key], &planned),
            strip(&real[key], &ran),
            "{key}: {dry} / {real}"
        );
    }
}

/// An `--all` run that fails on one host is an error like any other at its top
/// level, and the validator an agent runs accepts it.
#[rstest::rstest]
#[case::update("update", "fix-input")]
#[case::install("install", "run-command")]
fn an_all_hosts_failure_is_a_whole_error_envelope(#[case] verb: &str, #[case] action: &str) {
    let home = Home::new(CLONING_GIT);
    plain_files(&home.dest(HOST));

    let out = home.run_raw(&["skill", verb, "--all", "--output", "json"], None);

    assert_eq!(out.status.code(), Some(1));
    let v: Value = serde_json::from_slice(&out.stdout).expect("a JSON envelope");
    assert_eq!(v["status"], "error", "{v}");
    assert_eq!(v["exit_code"], 1, "{v}");
    assert_eq!(v["reason"], "destination-not-empty", "{v}");
    assert_eq!(v["next_step"]["action"], action, "{v}");
    let message = v["message"].as_str().expect("a message");
    assert!(message.contains(HOST), "it names the host: {message}");
    assert!(v["installations"].is_array(), "{v}");

    let validated = common::xr()
        .args(["--output", "json", "validate", "--schema", "envelope"])
        .write_stdin(out.stdout)
        .output()
        .expect("run xr");
    assert!(
        validated.status.success(),
        "{}",
        String::from_utf8_lossy(&validated.stderr)
    );
}

/// The worst exit code decides which host the top level speaks for.
#[test]
fn an_all_hosts_failure_speaks_for_the_worst_exit_code() {
    let home = Home::new(REFUSING_GIT);
    plain_files(&home.dest(HOST));
    write_bundle(&home.dest("cursor"));

    let (code, v) = home.update_with(&["skill", "update", "--all", "--output", "json"], None);

    assert_eq!(code, 128, "{v}");
    assert_eq!(v["exit_code"], 128, "{v}");
    assert_eq!(v["reason"], "git-clone-failed", "{v}");
    assert_eq!(v["next_step"]["action"], "retry", "{v}");
    let message = v["message"].as_str().expect("a message");
    assert!(
        message.contains(HOST) && message.contains("cursor"),
        "both failures are named: {message}"
    );
}

/// Each host's own entry says what went wrong with it too.
#[test]
fn a_failed_clone_says_the_install_is_as_it_was() {
    let home = Home::new(REFUSING_GIT);
    let dest = home.dest(HOST);
    write_bundle(&dest);

    let (_, v) = home.update(HOST);

    let message = v["message"].as_str().expect("a message");
    assert!(message.contains(&*dest.to_string_lossy()), "{message}");
    assert!(message.contains("128"), "it names git's code: {message}");
}

/// `skill install` refuses an occupied directory and a file, and its dry run
/// says so the same way.
#[rstest::rstest]
#[case::occupied(not_a_bundle, "destination-not-empty", "run-command")]
#[case::file(a_file, "destination-is-file", "fix-input")]
fn an_install_dry_run_that_would_be_refused_answers_at_exit_zero(
    #[case] arrange: fn(&Home),
    #[case] reason: &str,
    #[case] action: &str,
) {
    let home = Home::new(CLONING_GIT);
    arrange(&home);
    let before = tree(home.dir.path());

    let (code, v) = home.update_with(
        &["skill", "install", HOST, "--dry-run", "--output", "json"],
        None,
    );

    assert_eq!(code, 0, "{v}");
    assert_eq!(v["status"], "dry_run", "{v}");
    assert_eq!(v["would_succeed"], false, "{v}");
    assert_eq!(v["exit_code"], 1, "the real run's code: {v}");
    assert_eq!(v["reason"], reason, "{v}");
    assert_eq!(v["next_step"]["action"], action, "{v}");
    assert_eq!(tree(home.dir.path()), before, "the dry run wrote");
    assert!(home.clones().is_empty(), "the dry run cloned");

    let (real_code, real) = home.update_with(&["skill", "install", HOST, "--output", "json"], None);
    assert_eq!(real_code, 1, "{real}");
    assert!(v["message"].is_string(), "{v}");
    for key in [
        "reason",
        "exit_code",
        "message",
        "next_step",
        "destination_status",
    ] {
        assert_eq!(v[key], real[key], "{key}: {v} / {real}");
    }
}

/// `--all` answers at exit 0 too, and names the worst code the real run would
/// end on, whichever host it comes from.
#[rstest::rstest]
#[case::update("update", false)]
#[case::install("install", false)]
fn an_all_hosts_dry_run_that_would_be_refused_answers_at_exit_zero(
    #[case] verb: &str,
    #[case] would_succeed: bool,
) {
    let home = Home::new(CLONING_GIT);
    plain_files(&home.dest(HOST));
    let before = tree(home.dir.path());

    let (code, v) = home.update_with(
        &["skill", verb, "--all", "--dry-run", "--output", "json"],
        None,
    );

    assert_eq!(code, 0, "{v}");
    assert_eq!(v["status"], "dry_run", "{v}");
    assert_eq!(v["would_succeed"], would_succeed, "{v}");
    assert_eq!(v["exit_code"], 1, "the real run's worst code: {v}");
    let refused = v["installations"]
        .as_array()
        .expect("installations")
        .iter()
        .find(|entry| entry["host"] == HOST)
        .expect("the refused host");
    assert_eq!(refused["status"], "dry_run", "{v}");
    assert_eq!(refused["would_succeed"], false, "{v}");
    assert_eq!(refused["exit_code"], 1, "{v}");
    assert_eq!(refused["reason"], "destination-not-empty", "{v}");
    assert_eq!(tree(home.dir.path()), before, "the dry run wrote");
    assert!(home.clones().is_empty(), "the dry run cloned");
}

#[rstest::rstest]
#[case::update("update")]
#[case::install("install")]
fn an_all_hosts_dry_run_that_would_succeed_says_so(#[case] verb: &str) {
    let home = Home::new(CLONING_GIT);

    let (code, v) = home.update_with(
        &["skill", verb, "--all", "--dry-run", "--output", "json"],
        None,
    );

    assert_eq!(code, 0, "{v}");
    assert_eq!(v["status"], "dry_run", "{v}");
    assert_eq!(v["would_succeed"], true, "{v}");
    assert_eq!(v["exit_code"], 0, "{v}");
}

/// In text a refused dry run says what the real run would do, and exits 0.
#[test]
fn a_refused_dry_run_in_text_says_the_real_run_would_fail() {
    let home = Home::new(CLONING_GIT);
    let dest = home.dest(HOST);
    plain_files(&dest);

    let out = home.run_raw(&["skill", "update", HOST, "--dry-run"], None);

    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("The real run would fail with exit code 1: destination-not-empty"),
        "{text}"
    );
    assert!(text.contains(&*dest.to_string_lossy()), "{text}");
    assert!(text.contains("xurl-rs skill bundle"), "{text}");
}

#[test]
fn text_output_says_what_was_refused_and_what_to_do() {
    let home = Home::new(CLONING_GIT);
    let dest = home.dest(HOST);
    plain_files(&dest);

    let out = home.run_raw(&["skill", "update", HOST], None);

    assert_eq!(out.status.code(), Some(1));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("error: destination-not-empty") && text.contains(&*dest.to_string_lossy()),
        "{text}"
    );
    assert!(text.contains("xurl-rs skill bundle"), "{text}");
    assert_eq!(tree(&dest).len(), 1, "the refusal touched the directory");
}

#[test]
fn a_dry_run_in_text_names_the_bundle_it_would_replace() {
    let home = Home::new(CLONING_GIT);
    let dest = home.dest(HOST);
    write_bundle(&dest);

    let out = home.run_raw(&["skill", "update", HOST, "--dry-run"], None);

    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains(&format!("replaces the bundle at {}", dest.display())),
        "{text}"
    );
}
