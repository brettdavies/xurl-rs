//! `run_subcommand` routes a command to its group's module by variant, and
//! the module matches that variant again to run it. The compiler holds the
//! routing match exhaustive; this walk holds each module's arms equal to the
//! variants routed to it, so a variant on one side only fails here instead of
//! at run time.

mod common;

use std::collections::BTreeSet;

const COMMANDS_DIR: &str = "crates/xurl-cli/src/cli/commands";

fn source(file: &str) -> String {
    let path = common::workspace_root().join(COMMANDS_DIR).join(file);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// The `Commands` variants `text` names. The word boundary keeps the
/// family enums (`MediaCommands`, `UsageCommands`) out.
fn variants(text: &str) -> BTreeSet<String> {
    regex::Regex::new(r"\bCommands::([A-Z][A-Za-z]+)")
        .unwrap()
        .captures_iter(text)
        .map(|c| c[1].to_string())
        .collect()
}

/// Each group `run_subcommand` hands a whole command to, with the variants
/// its routing line names.
fn routed_groups() -> Vec<(String, BTreeSet<String>)> {
    let dispatcher = source("mod.rs");
    let start = dispatcher
        .find("async fn run_subcommand(")
        .expect("run_subcommand is declared");
    let end = start
        + dispatcher[start..]
            .find("\n}\n")
            .expect("run_subcommand has a closing brace");
    let arm = regex::Regex::new(
        r"((?:\s*\|?\s*Commands::\w+(?: \{ \.\. \})?)+)\s*=>\s*(\w+)::run\(cmd, run\)",
    )
    .unwrap();
    arm.captures_iter(&dispatcher[start..end])
        .map(|c| (c[2].to_string(), variants(&c[1])))
        .collect()
}

#[test]
fn each_group_matches_the_variants_routed_to_it() {
    let groups = routed_groups();
    assert!(
        groups.len() >= 5,
        "found only {} groups in run_subcommand; the scan is broken: {groups:?}",
        groups.len()
    );
    let mut drift = Vec::new();
    for (group, routed) in groups {
        let matched = variants(&source(&format!("{group}.rs")));
        let unhandled: Vec<&String> = routed.difference(&matched).collect();
        let unrouted: Vec<&String> = matched.difference(&routed).collect();
        if !unhandled.is_empty() || !unrouted.is_empty() {
            drift.push(format!(
                "{group}: routed with no arm {unhandled:?}; an arm with no route {unrouted:?}"
            ));
        }
    }
    assert!(
        drift.is_empty(),
        "these command groups and their routing lines disagree:\n{}\n\
         Cause: a command's variant was added to its group's module or to that group's line \
         in run_subcommand, and not to the other.\n\
         Fix: name the variant on the group's line in run_subcommand in \
         {COMMANDS_DIR}/mod.rs, and give it an arm in {COMMANDS_DIR}/<group>.rs.",
        drift.join("\n")
    );
}
