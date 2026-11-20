//! `scripts/check-surface-bump.sh` fails a pull request whose
//! `## Changelog (xurl-rs)` block has a heading outside the six, whose `xr`
//! surface grows while the block names no minor-or-higher section (growth in
//! the schemas alone also passes with a `### Fixed` bullet), or whose surface
//! shrinks without a `### Breaking changes` bullet starting "Remove". The head
//! tree in every case is the repository itself, so the parser runs against the
//! committed completions, schemas, and PR template rather than hand-written
//! copies of their formats.
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

/// A PR body whose binary block holds `content` and whose library block holds
/// `library`.
fn body_with_blocks(content: &str, library: &str) -> String {
    format!(
        "## Summary\n\nA change.\n\n## Changelog (xurl-rs)\n\n{content}\n\n## Changelog (xdk-rs)\n\n{library}\n"
    )
}

fn changelog_with(section: &str, bullet: &str) -> String {
    body_with_blocks(&format!("### {section}\n\n- {bullet}"), "")
}

fn changelog(section: &str) -> String {
    changelog_with(section, "An entry.")
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
        "schema\tschema/output.schema.json\t$defs.NextAction=declared",
    ] {
        assert!(
            out.stdout.lines().any(|line| line == expected),
            "the surface is missing {expected:?}"
        );
    }
    assert!(
        !out.stdout.contains(".description=\""),
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
    without_a_next_action(&base);
    let out = check(&base, &body(&base, &changelog("Changed")));
    assert_eq!(
        out.code, 1,
        "stdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
    assert!(out.stderr.contains("show-help"), "{}", out.stderr);
    assert!(
        out.stderr.contains("### Fixed"),
        "the failure names the section a catching-up schema goes under: {}",
        out.stderr
    );
}

/// Drops `show-help` from the base tree's envelope schema, so the head tree's
/// schema has grown by that action.
fn without_a_next_action(base: &TempDir) {
    edit_envelope(base, |schema| {
        schema["$defs"]["NextAction"]["oneOf"]
            .as_array_mut()
            .expect("NextAction is a oneOf")
            .retain(|member| member["const"] != "show-help");
    });
}

#[test]
fn schema_growth_alone_filed_under_fixed_passes() {
    let base = base_tree();
    without_a_next_action(&base);
    let out = check(&base, &body(&base, &changelog("Fixed")));
    assert_eq!(
        out.code, 0,
        "stdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
    assert!(
        out.stdout.contains(
            "check-surface-bump: only the xr schemas grew and ### Fixed has a bullet; confirm each entry describes output the binary already prints and its docs already describe:"
        ),
        "{}",
        out.stdout
    );
    assert!(out.stderr.contains("show-help"), "{}", out.stderr);
}

#[test]
fn command_growth_filed_under_fixed_fails() {
    let base = base_without_examples();
    let out = check(&base, &body(&base, &changelog("Fixed")));
    assert_eq!(
        out.code, 1,
        "stdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
    assert!(out.stderr.contains("xr\texamples"), "{}", out.stderr);
}

#[test]
fn command_growth_with_schema_growth_filed_under_fixed_fails() {
    let base = base_without_examples();
    without_a_next_action(&base);
    let out = check(&base, &body(&base, &changelog("Fixed")));
    assert_eq!(
        out.code, 1,
        "stdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
    assert!(out.stderr.contains("xr\texamples"), "{}", out.stderr);
}

#[test]
fn a_schema_id_is_not_surface() {
    let base = base_tree();
    edit_envelope(&base, |schema| {
        schema["$id"] = Value::from("https://example.invalid/schema/v1/output.schema.json");
    });
    let out = check(&base, &body(&base, &changelog("Changed")));
    assert_eq!(out.code, 0, "{}", out.stderr);
}

#[test]
fn examples_are_not_surface() {
    let base = base_tree();
    edit_envelope(&base, |schema| {
        schema["$defs"]["NextAction"]["examples"] =
            serde_json::json!([{ "action": "show-help", "command": "xr --help" }]);
    });
    let out = check(&base, &body(&base, &changelog("Changed")));
    assert_eq!(out.code, 0, "{}", out.stderr);
}

/// A key declared under `$defs` whose only content is prose has no leaf of its
/// own, so the declaration is the surface.
#[test]
fn a_declared_key_with_only_prose_is_surface() {
    let base = base_tree();
    edit_envelope(&base, |schema| {
        schema["$defs"]["Retired"] = serde_json::json!({ "description": "Only prose." });
    });
    let out = check(&base, &body(&base, &changelog("Changed")));
    assert_eq!(
        out.code, 1,
        "stdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
    assert!(
        out.stderr.contains("$defs.Retired=declared"),
        "the failure names the declaration: {}",
        out.stderr
    );
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

/// A base tree whose `xr` has one more command than the repository, so the
/// head tree has lost it.
fn base_with_a_retired_command() -> TempDir {
    let base = base_tree();
    let path = base.path().join("completions/xr.bash");
    let text = fs::read_to_string(&path).expect("read completions");
    let edited = text.replace(
        " examples validate help\"",
        " examples validate retired help\"",
    );
    assert_ne!(
        edited, text,
        "the completions no longer list `examples validate help`"
    );
    fs::write(&path, edited).expect("write completions");
    base
}

#[test]
fn a_removal_filed_under_changed_fails_naming_the_remove_bullet() {
    let base = base_with_a_retired_command();
    let out = check(&base, &body(&base, &changelog("Changed")));
    assert_eq!(
        out.code, 1,
        "stdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
    assert!(out.stderr.contains("xr\tretired"), "{}", out.stderr);
    assert!(
        out.stderr.contains("### Breaking changes") && out.stderr.contains("Remove"),
        "{}",
        out.stderr
    );
    assert!(
        !out.stderr.contains("### Fixed"),
        "a removal is not filed under ### Fixed: {}",
        out.stderr
    );
}

#[test]
fn a_removal_with_a_breaking_changes_remove_bullet_passes() {
    let base = base_with_a_retired_command();
    let text = changelog_with("Breaking changes", "Remove `xr retired`.");
    let out = check(&base, &body(&base, &text));
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(
        out.stdout
            .contains("### Breaking changes has a bullet starting \"Remove\""),
        "{}",
        out.stdout
    );
}

#[test]
fn a_removal_filed_under_fixed_fails() {
    let base = base_with_a_retired_command();
    let text = changelog_with("Fixed", "Remove `xr retired`.");
    let out = check(&base, &body(&base, &text));
    assert_eq!(
        out.code, 1,
        "stdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
    assert!(out.stderr.contains("xr\tretired"), "{}", out.stderr);
}

#[test]
fn a_breaking_changes_bullet_not_starting_with_remove_does_not_cover_a_removal() {
    let base = base_with_a_retired_command();
    let text = changelog_with("Breaking changes", "Rework the retired command.");
    let out = check(&base, &body(&base, &text));
    assert_eq!(
        out.code, 1,
        "stdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
    assert!(out.stderr.contains("xr\tretired"), "{}", out.stderr);
}

/// A base tree whose `xr` has `retired` where the repository has `examples`.
fn base_with_a_renamed_command() -> TempDir {
    let base = base_tree();
    let path = base.path().join("completions/xr.bash");
    let text = fs::read_to_string(&path).expect("read completions");
    let edited = text.replace(" examples validate help\"", " retired validate help\"");
    assert_ne!(
        edited, text,
        "the completions no longer list `examples validate help`"
    );
    fs::write(&path, edited).expect("write completions");
    base
}

#[test]
fn a_rename_reads_as_a_removal_and_an_addition() {
    let base = base_with_a_renamed_command();
    let rename = changelog_with("Breaking changes", "Rename `xr retired` to `xr examples`.");
    let out = check(&base, &body(&base, &rename));
    assert_eq!(
        out.code, 1,
        "stdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
    assert!(out.stderr.contains("xr\tretired"), "{}", out.stderr);

    let removal = changelog_with(
        "Breaking changes",
        "Remove `xr retired`, replaced by `xr examples`.",
    );
    let out = check(&base, &body(&base, &removal));
    assert_eq!(out.code, 0, "{}", out.stderr);
}

/// A change list larger than a pipe buffer is still listed and counted,
/// rather than ending the script when the truncated listing stops reading.
#[test]
fn a_removal_too_large_for_a_pipe_still_fails_with_a_count() {
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
    let out = check(&base, &body(&base, &changelog("Changed")));
    assert_eq!(
        out.code, 1,
        "stdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
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

#[test]
fn every_one_of_the_six_headings_passes_on_an_unchanged_surface() {
    let base = base_tree();
    for section in [
        "Breaking changes",
        "Added",
        "Changed",
        "Deprecated",
        "Fixed",
        "Documentation",
    ] {
        let out = check(&base, &body(&base, &changelog(section)));
        assert_eq!(out.code, 0, "{section}: {}", out.stderr);
    }
}

/// The release's generator files an unknown `###` heading as a patch and reads
/// neither a deeper nor an indented one, so a removal under any of them would
/// ship without its major.
#[test]
fn a_heading_outside_the_six_fails_whatever_the_surface_did() {
    for (content, named) in [
        ("### Misc\n\n- An entry.", "### Misc"),
        (
            "### Added\n\n- An entry.\n\n#### Notes\n\nMore.",
            "#### Notes",
        ),
        (
            "### Added\n\n- An entry.\n\n  ### Notes\n\nMore.",
            "### Notes",
        ),
    ] {
        let unchanged = base_tree();
        let out = check(
            &unchanged,
            &body(&unchanged, &body_with_blocks(content, "")),
        );
        assert_eq!(
            out.code, 1,
            "{named}\nstdout: {}\nstderr: {}",
            out.stdout, out.stderr
        );
        assert!(
            out.stderr.contains("outside the six") && out.stderr.contains(named),
            "{named}: {}",
            out.stderr
        );

        let grown = base_without_examples();
        let out = check(&grown, &body(&grown, &body_with_blocks(content, "")));
        assert_eq!(
            out.code, 1,
            "{named}, grown\nstdout: {}\nstderr: {}",
            out.stdout, out.stderr
        );
        assert!(out.stderr.contains(named), "{named}: {}", out.stderr);
    }
}

#[test]
fn a_heading_outside_the_six_in_the_library_block_does_not_count() {
    let base = base_tree();
    let text = body_with_blocks("### Added\n\n- An entry.", "### Misc\n\n- A library entry.");
    let out = check(&base, &body(&base, &text));
    assert_eq!(out.code, 0, "{}", out.stderr);
}

/// A schema key holding a newline would otherwise start a CI log line of its
/// own.
#[test]
fn a_schema_key_holding_a_newline_stays_on_one_line() {
    let tree = base_tree();
    fs::write(
        tree.path().join("schema/responses/odd.schema.json"),
        serde_json::json!({ "properties": { "a\nb": { "type": "string" } } }).to_string(),
    )
    .expect("write the odd schema");
    let out = run(&[Path::new("surface"), tree.path()]);
    assert_eq!(out.code, 0, "surface failed: {}", out.stderr);
    for line in out.stdout.lines() {
        assert!(
            line.starts_with("cli\t") || line.starts_with("schema\t"),
            "a line that belongs to no entry: {line:?}"
        );
    }
    let expected = "schema\tschema/responses/odd.schema.json\tproperties.a\\nb.type=\"string\"";
    assert!(
        out.stdout.lines().any(|line| line == expected),
        "the surface is missing {expected:?}"
    );
}

#[test]
fn help_prints_the_usage_and_exits_zero() {
    let out = run(&[Path::new("--help")]);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(
        out.stdout.contains("Usage:")
            && out
                .stdout
                .contains("check-surface-bump.sh check <base-tree> <head-tree> <pr-body-file>"),
        "{}",
        out.stdout
    );
}
