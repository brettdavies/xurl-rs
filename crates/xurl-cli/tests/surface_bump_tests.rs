//! `scripts/check-surface-bump.sh` fails a pull request whose `xr` surface
//! grows while its `## Changelog (xurl-rs)` block names no minor-or-higher
//! section. The head tree in every case is the repository itself, so the
//! parser runs against the committed completions, schemas, and PR template
//! rather than hand-written copies of their formats.
#![cfg(unix)]

mod common;

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::Value;
use tempfile::TempDir;

const SCRIPT: &str = "scripts/check-surface-bump.sh";

struct Outcome {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(args: &[&Path]) -> Outcome {
    let root = common::workspace_root();
    let script = root.join(SCRIPT);
    assert!(script.exists(), "{SCRIPT} is missing");
    let output = Command::new("bash")
        .arg(&script)
        .args(args)
        .current_dir(root)
        .env_remove("GITHUB_ACTIONS")
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn {SCRIPT}: {e}"));
    Outcome {
        code: output
            .status
            .code()
            .expect("the script exits rather than dying on a signal"),
        stdout: String::from_utf8(output.stdout).expect("stdout is UTF-8"),
        stderr: String::from_utf8(output.stderr).expect("stderr is UTF-8"),
    }
}

/// A copy of the surface files `check` reads, for a base tree to diverge from.
fn base_tree() -> TempDir {
    let root = common::workspace_root();
    let dir = TempDir::new().expect("tempdir");
    fs::create_dir_all(dir.path().join("completions")).expect("completions dir");
    fs::copy(
        root.join("completions/xr.bash"),
        dir.path().join("completions/xr.bash"),
    )
    .expect("copy completions");
    copy_dir(&root.join("schema"), &dir.path().join("schema"));
    dir
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create schema dir");
    for entry in fs::read_dir(from).expect("read schema dir") {
        let path = entry.expect("dir entry").path();
        let dest = to.join(path.file_name().expect("file name"));
        if path.is_dir() {
            copy_dir(&path, &dest);
        } else {
            fs::copy(&path, &dest).expect("copy schema file");
        }
    }
}

/// The base tree as it would be before `examples` became a command.
fn base_without_examples() -> TempDir {
    let base = base_tree();
    let path = base.path().join("completions/xr.bash");
    let text = fs::read_to_string(&path).expect("read completions");
    assert!(
        text.contains(" examples validate help\""),
        "the root opts line no longer lists `examples validate help`; pick another command to remove"
    );
    fs::write(
        &path,
        text.replace(" examples validate help\"", " validate help\""),
    )
    .expect("write completions");
    base
}

/// Edit the base tree's envelope schema through `edit`.
fn edit_envelope(base: &TempDir, edit: impl FnOnce(&mut Value)) {
    let path = base.path().join("schema/output.schema.json");
    let mut schema: Value = serde_json::from_str(&fs::read_to_string(&path).expect("read schema"))
        .expect("parse schema");
    edit(&mut schema);
    fs::write(
        &path,
        serde_json::to_string_pretty(&schema).expect("render schema"),
    )
    .expect("write schema");
}

fn body(dir: &TempDir, text: &str) -> std::path::PathBuf {
    let path = dir.path().join("body.md");
    fs::write(&path, text).expect("write body");
    path
}

fn changelog(section: &str) -> String {
    format!(
        "## Summary\n\nA change.\n\n## Changelog (xurl-rs)\n\n### {section}\n\n- An entry.\n\n## Changelog (xdk-rs)\n"
    )
}

fn check(base: &TempDir, body: &Path) -> Outcome {
    run(&[
        Path::new("check"),
        base.path(),
        common::workspace_root(),
        body,
    ])
}

#[test]
fn surface_reads_commands_flags_values_and_schema_leaves() {
    let out = run(&[Path::new("surface"), common::workspace_root()]);
    assert_eq!(out.code, 0, "surface failed: {}", out.stderr);
    for expected in [
        "cli\txr auth\tstatus",
        "cli\txr\t--output",
        "cli\txr\t--output=json",
        "cli\txr\t--color=always",
        "schema\tschema/output.schema.json\t$defs.NextAction.oneOf.[].const=\"show-help\"",
    ] {
        assert!(
            out.stdout.lines().any(|line| line == expected),
            "the surface is missing {expected:?}"
        );
    }
    assert!(
        !out.stdout.contains(".description="),
        "a description is prose, not surface"
    );
}

#[test]
fn a_new_command_filed_under_changed_fails() {
    let base = base_without_examples();
    let out = check(&base, &body(&base, &changelog("Changed")));
    assert_eq!(
        out.code, 1,
        "stdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
    assert!(
        out.stderr.contains("xr\texamples"),
        "the failure names the new command: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("### Added"),
        "the failure names the section: {}",
        out.stderr
    );
}

#[test]
fn a_new_command_filed_under_a_minor_or_higher_section_passes() {
    let base = base_without_examples();
    for section in ["Added", "Deprecated", "Breaking changes"] {
        let out = check(&base, &body(&base, &changelog(section)));
        assert_eq!(out.code, 0, "{section}: {}", out.stderr);
    }
}

#[test]
fn a_new_envelope_action_filed_under_changed_fails() {
    let base = base_tree();
    edit_envelope(&base, |schema| {
        schema["$defs"]["NextAction"]["oneOf"]
            .as_array_mut()
            .expect("NextAction is a oneOf")
            .retain(|member| member["const"] != "show-help");
    });
    let out = check(&base, &body(&base, &changelog("Changed")));
    assert_eq!(
        out.code, 1,
        "stdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
    assert!(out.stderr.contains("show-help"), "{}", out.stderr);
}

#[test]
fn a_reworded_description_is_not_growth() {
    let base = base_tree();
    edit_envelope(&base, |schema| {
        schema["$defs"]["NextAction"]["description"] = Value::from("Older wording.");
    });
    let out = check(&base, &body(&base, &changelog("Changed")));
    assert_eq!(out.code, 0, "{}", out.stderr);
}

#[test]
fn a_reordered_list_is_not_growth() {
    let base = base_tree();
    edit_envelope(&base, |schema| {
        schema["$defs"]["NextAction"]["oneOf"]
            .as_array_mut()
            .expect("NextAction is a oneOf")
            .reverse();
    });
    let out = check(&base, &body(&base, &changelog("Changed")));
    assert_eq!(out.code, 0, "{}", out.stderr);
}

/// The real template keeps its guidance in HTML comments and leaves a bare `-`
/// under every heading; neither is an entry, and an `### Added` bullet in the
/// library's block does not speak for the binary.
#[test]
fn template_guidance_and_the_library_block_do_not_count() {
    let template =
        fs::read_to_string(common::workspace_root().join(".github/pull_request_template.md"))
            .expect("read the PR template");
    let (binary, library) = template
        .split_once("## Changelog (xdk-rs)")
        .expect("the template has a library block");
    let filled_binary = binary.replacen("### Changed\n\n-\n", "### Changed\n\n- An entry.\n", 1);
    let filled_library =
        library.replacen("### Added\n\n-\n", "### Added\n\n- A library entry.\n", 1);
    assert_ne!(
        filled_binary, binary,
        "no Changed placeholder in the binary block"
    );
    assert_ne!(
        filled_library, library,
        "no Added placeholder in the library block"
    );
    let filled = format!("{filled_binary}## Changelog (xdk-rs){filled_library}");
    let base = base_without_examples();
    let out = check(&base, &body(&base, &filled));
    assert_eq!(
        out.code, 1,
        "stdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
}

/// A body edited in the browser arrives with CRLF line endings.
#[test]
fn a_crlf_body_is_read() {
    let base = base_without_examples();
    let crlf = changelog("Added").replace('\n', "\r\n");
    let out = check(&base, &body(&base, &crlf));
    assert_eq!(out.code, 0, "{}", out.stderr);
}

#[test]
fn a_removal_passes_with_a_warning_naming_breaking_changes() {
    let base_has_more = base_tree();
    let path = base_has_more.path().join("completions/xr.bash");
    let text = fs::read_to_string(&path).expect("read completions");
    fs::write(
        &path,
        text.replace(
            " examples validate help\"",
            " examples validate retired help\"",
        ),
    )
    .expect("write completions");
    let out = check(&base_has_more, &body(&base_has_more, &changelog("Fixed")));
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(out.stderr.contains("xr\tretired"), "{}", out.stderr);
    assert!(
        out.stderr.contains("### Breaking changes"),
        "{}",
        out.stderr
    );
}

/// A change list larger than a pipe buffer is still listed and counted,
/// rather than ending the script when the truncated listing stops reading.
#[test]
fn a_removal_too_large_for_a_pipe_still_passes() {
    let base = base_tree();
    let fields: serde_json::Map<String, Value> = (0..4000)
        .map(|i| {
            (
                format!("field_{i:05}_with_a_long_enough_name"),
                serde_json::json!({ "type": "string", "format": "x" }),
            )
        })
        .collect();
    fs::write(
        base.path().join("schema/responses/retired.schema.json"),
        serde_json::json!({ "properties": fields }).to_string(),
    )
    .expect("write the retired schema");
    let out = check(&base, &body(&base, &changelog("Fixed")));
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(
        out.stderr.contains("more"),
        "the listing is truncated with a count: {}",
        out.stderr
    );
}

/// A parser that cannot run has no verdict to give, so the check reports an
/// error instead of claiming the block lacks a section.
#[test]
fn a_parser_that_cannot_run_is_an_error_not_a_verdict() {
    let base = base_without_examples();
    let head = base_tree();
    fs::create_dir_all(head.path().join("scripts")).expect("scripts dir");
    fs::write(
        head.path().join("scripts/generate-changelog.py"),
        "raise SystemExit('this generator cannot be loaded')\n",
    )
    .expect("write the broken generator");
    fs::create_dir_all(head.path().join("crates/xurl-cli")).expect("crate dir");
    fs::copy(
        common::workspace_root().join("crates/xurl-cli/Cargo.toml"),
        head.path().join("crates/xurl-cli/Cargo.toml"),
    )
    .expect("copy the manifest");
    let out = run(&[
        Path::new("check"),
        base.path(),
        head.path(),
        &body(&base, &changelog("Added")),
    ]);
    assert_eq!(
        out.code, 2,
        "stdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
    assert!(
        out.stderr.contains("generate-changelog.py"),
        "the error names the parser: {}",
        out.stderr
    );
}

#[test]
fn an_unchanged_surface_passes_with_no_changelog() {
    let base = base_tree();
    let out = check(&base, &body(&base, "## Summary\n\nTests only.\n"));
    assert_eq!(out.code, 0, "{}", out.stderr);
}
