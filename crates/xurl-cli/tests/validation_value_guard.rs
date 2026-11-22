//! A validation refusal for one malformed input carries that input as `value`.
//!
//! Every runnable command runs in process, under `--dry-run` and for real,
//! once per argument it takes, with that one argument replaced by a
//! malformed value and every other argument as `arguments_for` gives it.
//! A refusal that answers `validation` was caused by the one argument that
//! changed, so it must carry that argument as `value`. The commands and
//! their arguments are read from clap, so a command or an argument added
//! later is checked without an edit here.

mod common;

use std::collections::BTreeSet;

use serde_json::Value;
use xdk::testing::MockX;

use common::invoke::{
    Ran, Seed, answering_mock, arguments_for, command_at, every_kind_with_the_login_expired,
    nothing, run_xr, runnable_paths, walk,
};

/// A value past every length limit a validator states, and in no form it
/// states for an id, a key, a handle, a URL, a time, or a language code: it
/// holds `!`.
fn malformed() -> String {
    "x!".repeat(1_000)
}

/// One run: `path` with `args`, in which `label` names the argument that
/// holds `token`.
struct Probe {
    path: String,
    label: String,
    args: Vec<String>,
    token: String,
}

/// Every probe of the command at `path`: each value in its arguments, and
/// each option that takes a value but is not among them, given the token.
fn probes_for(path: &str) -> Vec<Probe> {
    let base = arguments_for(path);
    let options: Vec<String> = command_at(path)
        .get_arguments()
        .filter(|arg| {
            !arg.is_global_set()
                && !arg.is_positional()
                && arg.get_action().takes_values()
                && arg.get_possible_values().is_empty()
        })
        .filter_map(|arg| arg.get_long().map(|long| format!("--{long}")))
        .filter(|flag| !base.contains(flag))
        .collect();

    let mut probes = Vec::new();
    let token = malformed();
    let probe = |label: String, args: Vec<String>| Probe {
        path: path.to_string(),
        label,
        args,
        token: token.clone(),
    };
    for (index, arg) in base.iter().enumerate() {
        if arg.starts_with("--") || arg == "{file}" {
            continue;
        }
        let label = match index.checked_sub(1).map(|before| &base[before]) {
            Some(flag) if flag.starts_with("--") => flag.clone(),
            _ => format!("the argument {arg}"),
        };
        let mut args = base.clone();
        args[index] = token.clone();
        probes.push(probe(label, args));
    }
    for flag in &options {
        let mut args = base.clone();
        args.extend([flag.clone(), token.clone()]);
        probes.push(probe(flag.clone(), args));
    }
    probes
}

/// The paths whose own arguments are already refused as invalid, so a
/// refusal of one of their probes proves nothing about the probe's token.
async fn refused_as_given(mock: &MockX, seed: &Seed, path: &str) -> Option<String> {
    let ran = run_xr(&mock.base_url(), seed, path, &arguments_for(path), true).await;
    (Ran::document(&ran.stdout)["reason"] == "validation").then(|| path.to_string())
}

/// What is wrong with how `probe`'s command refuses its malformed argument.
async fn faults(mock: &MockX, seed: &Seed, probe: &Probe) -> Vec<String> {
    let origin = mock.base_url();
    let mut found = Vec::new();
    for dry_run in [true, false] {
        let ran = run_xr(&origin, seed, &probe.path, &probe.args, dry_run).await;
        let document = Ran::document(if dry_run { &ran.stdout } else { &ran.stderr });
        if document["reason"] != "validation" {
            break;
        }
        if document["value"] != probe.token {
            let how = if dry_run { "dry run" } else { "real run" };
            found.push(format!(
                "{} ({how}): {} was given a malformed value and refused with rule {} and \
                 `value` {}\n  \
                 Cause: the command builds the refusal from the rule alone, so the input it \
                 refused is left out.\n  \
                 Fix: build the refusal with the input it names, `Invalid::naming(rule, input)` \
                 or `checked(validator, input)`, in the command's module under \
                 crates/xurl-cli/src/cli/commands/.",
                probe.path,
                probe.label,
                document["rule"],
                if document["value"].is_null() {
                    "absent".to_string()
                } else {
                    document["value"].to_string().chars().take(40).collect()
                },
            ));
        }
    }
    found
}

#[test]
fn a_refusal_names_the_malformed_input() {
    let seed = Seed::of(every_kind_with_the_login_expired);
    let paths: Vec<String> = runnable_paths().into_iter().collect();
    let already_refused: BTreeSet<String> = walk(&paths, answering_mock, async |mock, path| {
        refused_as_given(mock, &seed, path).await
    })
    .into_iter()
    .flatten()
    .collect();
    let probes: Vec<Probe> = paths
        .iter()
        .filter(|path| !already_refused.contains(*path))
        .flat_map(|path| probes_for(path))
        .collect();
    assert!(
        probes.len() > 200,
        "the arguments the commands take: {}",
        probes.len()
    );

    let found: Vec<String> = walk(&probes, answering_mock, async |mock, probe: &Probe| {
        faults(mock, &seed, probe).await
    })
    .into_iter()
    .flatten()
    .collect();

    assert!(
        found.is_empty(),
        "these refusals leave out the input they refused:\n{}",
        found.join("\n")
    );
}

/// The documents `xr` answers `command` with: a dry run's on stdout and a
/// real run's on stderr. A refusal sends nothing, so no server answers.
async fn answers(command: &str, args: &[&str]) -> [Value; 2] {
    let seed = Seed::of(nothing);
    let args: Vec<String> = args.iter().map(ToString::to_string).collect();
    let dry = run_xr(common::CLOSED, &seed, command, &args, true).await;
    let real = run_xr(common::CLOSED, &seed, command, &args, false).await;
    [Ran::document(&dry.stdout), Ran::document(&real.stderr)]
}

/// Asserts each document of `command` with `args` is a validation refusal
/// under `rule` whose `value` is `value`, or absent when `value` is `None`.
async fn assert_refusal(command: &str, args: &[&str], rule: &str, value: Option<&str>) {
    for document in answers(command, args).await {
        assert_eq!(document["reason"], "validation", "{command}: {document}");
        assert_eq!(document["rule"], rule, "{command}: {document}");
        assert_eq!(
            document.get("value").and_then(Value::as_str),
            value,
            "{command}: {document}"
        );
    }
}

#[tokio::test]
async fn an_id_is_named_as_given() {
    assert_refusal("lists get", &["abc!"], "invalid-list-id", Some("abc!")).await;
    assert_refusal(
        "communities get",
        &["abc!"],
        "invalid-community-id",
        Some("abc!"),
    )
    .await;
}

#[tokio::test]
async fn a_batch_names_the_first_input_it_refused() {
    for (command, args, rule, value) in [
        (
            "spaces get",
            &["abc", "bad!", "def", "x!"][..],
            "invalid-space-id",
            "bad!",
        ),
        (
            "stream rules remove",
            &["1", "2", "x!", "y!"][..],
            "invalid-rule-id",
            "x!",
        ),
        (
            "users",
            &["jack", "bad!", "x!"][..],
            "invalid-username",
            "bad!",
        ),
        (
            "media get",
            &["3_1", "bad!", "3_2"][..],
            "invalid-media-key",
            "bad!",
        ),
    ] {
        assert_refusal(command, args, rule, Some(value)).await;
    }
}

#[tokio::test]
async fn a_refusal_of_the_batch_as_a_whole_names_no_input() {
    assert_refusal("users", &["123", "jack"], "mixed-ids-and-handles", None).await;
    let many: Vec<String> = (1..=101).map(|n| n.to_string()).collect();
    let many: Vec<&str> = many.iter().map(String::as_str).collect();
    assert_refusal("users", &many, "too-many-ids", None).await;
}

#[tokio::test]
async fn a_time_window_names_the_bound_outside_the_form() {
    let start = "2026-10-01T00:00:00Z";
    let later = "2026-10-02T00:00:00Z";
    assert_refusal(
        "analytics",
        &["1", "--start", start, "--end", "bad!"],
        "invalid-time-window",
        Some("bad!"),
    )
    .await;
    assert_refusal(
        "analytics",
        &["1", "--start", later, "--end", start],
        "invalid-time-window",
        None,
    )
    .await;
}

#[tokio::test]
async fn a_blank_input_has_nothing_to_name() {
    assert_refusal("lists get", &[""], "invalid-list-id", None).await;
    assert_refusal("follow", &[""], "empty-username", None).await;
}
