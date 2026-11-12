//! A post id that is not an id is refused before any request.
//!
//! Every command with a post id argument (`post_id`, `post_ids`, or `post`
//! in clap) runs in process, for real and under `--dry-run`, with a value
//! that is neither digits nor a post URL, against a server that records
//! whatever reaches it. Each must refuse with the `invalid-post-id` rule,
//! name the input, and send nothing. The command list and each command's
//! arguments are read from clap, so a command added later is checked
//! without an edit here.

mod common;

use xdk::testing::MockX;

use common::invoke::{
    Ran, SAMPLE, Seed, answering_mock, arguments_for, command_at,
    every_kind_with_the_login_expired, run_xr, runnable_paths, walk,
};

const NOT_A_POST: &str = "not-a-post";
const POST_ID_ARGS: &[&str] = &["post_id", "post_ids", "post"];

/// Whether the command at `path` takes a post id.
fn takes_a_post_id(path: &str) -> bool {
    command_at(path)
        .get_arguments()
        .any(|arg| POST_ID_ARGS.contains(&arg.get_id().as_str()))
}

/// What is wrong with how `path` refuses a post id that is not one.
async fn faults(mock: &MockX, seed: &Seed, path: &String) -> Vec<String> {
    let origin = mock.base_url();
    let args: Vec<String> = arguments_for(path)
        .into_iter()
        .map(|arg| {
            if arg == SAMPLE {
                NOT_A_POST.to_string()
            } else {
                arg
            }
        })
        .collect();
    let mut found = Vec::new();
    for dry_run in [true, false] {
        let seen = mock.requests().await.len();
        let ran = run_xr(&origin, seed, path, &args, dry_run).await;
        let sent = mock.requests().await.len() - seen;
        let document = if dry_run {
            Ran::document(&ran.stdout)
        } else {
            Ran::document(&ran.stderr)
        };
        let how = if dry_run { "dry run" } else { "real run" };
        if document["rule"] != "invalid-post-id" || document["value"] != NOT_A_POST {
            found.push(format!(
                "{path} ({how}): answered {document}\n  \
                 Cause: the command checks its post id with something other than \
                 valid_post_id, or checks another input first.\n  \
                 Fix: validate the post id with valid_post_id in \
                 crates/xurl-cli/src/cli/commands/mod.rs, before any other input."
            ));
        }
        if sent != 0 {
            found.push(format!(
                "{path} ({how}): sent {sent} request(s) with the post id {NOT_A_POST}\n  \
                 Cause: the command sends before it validates its post id.\n  \
                 Fix: validate the post id first, in crates/xurl-cli/src/cli/commands/."
            ));
        }
    }
    found
}

#[test]
fn a_post_id_that_is_not_one_is_refused_before_any_request() {
    let seed = Seed::of(every_kind_with_the_login_expired);
    let paths: Vec<String> = runnable_paths()
        .into_iter()
        .filter(|path| takes_a_post_id(path))
        .collect();
    assert!(
        paths.len() > 10,
        "the commands that take a post id: {paths:?}"
    );

    let found: Vec<String> = walk(&paths, answering_mock, async |mock, path: &String| {
        faults(mock, &seed, path).await
    })
    .into_iter()
    .flatten()
    .collect();

    assert!(
        found.is_empty(),
        "these commands accept a post id that is not one:\n{}",
        found.join("\n")
    );
}

/// A list that is too long and holds a bad id is refused for the id: the
/// post id is the first input a command checks.
#[test]
fn a_bad_id_is_refused_before_the_length_of_the_list() {
    let ids: Vec<String> = std::iter::once(NOT_A_POST.to_string())
        .chain((1..=100).map(|n| n.to_string()))
        .collect();
    let window = [
        "--start",
        "2026-10-01T00:00:00Z",
        "--end",
        "2026-10-02T00:00:00Z",
    ];
    for (command, extra) in [("posts", &window[..0]), ("analytics", &window[..])] {
        let output = common::xr()
            .args(["--output", "json", "--dry-run", command])
            .args(&ids)
            .args(extra)
            .output()
            .expect("xr runs");
        let document: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("a document");
        assert_eq!(document["rule"], "invalid-post-id", "{command}: {document}");
        assert_eq!(document["value"], NOT_A_POST, "{command}: {document}");
    }
}

#[test]
fn a_post_url_or_a_numeric_id_is_a_post_id() {
    for good in [
        "1234567890",
        "https://x.com/jack/status/20",
        "  1234567890  ",
    ] {
        assert_eq!(
            xdk::api::shortcuts::validate_post_id(good),
            Ok(()),
            "{good}"
        );
    }
    for (bad, rule) in [
        ("", "empty-post-id"),
        ("@jack", "invalid-post-id"),
        ("12a", "invalid-post-id"),
        ("https://x.com/jack", "invalid-post-id"),
        ("https://x.com/jack/status/abc", "invalid-post-id"),
        ("12345678901234567890", "invalid-post-id"),
    ] {
        assert_eq!(
            xdk::api::shortcuts::validate_post_id(bad),
            Err(rule),
            "{bad}"
        );
    }
}
