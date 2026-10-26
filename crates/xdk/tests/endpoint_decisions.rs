//! `endpoint-decisions.json` records what `xr` decided for every spec
//! operation that has no shortcut command: no command until a trigger fires
//! (`raw-only`, `not-supported`), or a command family to build (`planned`).
//! These tests hold it to the vendored spec and to the shortcut list, so a
//! decision never outlives the operations it names or contradicts a command
//! that exists, and no operation goes undecided.

mod common;

use std::collections::BTreeSet;

use serde::Deserialize;
use xdk::api::auth_matrix::SHORTCUT_TEMPLATES;

use common::load_spec;

const REGISTRY_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../endpoint-decisions.json");

const METHODS: [&str; 5] = ["get", "post", "put", "patch", "delete"];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    #[serde(rename = "$comment")]
    _comment: String,
    decisions: Vec<Decision>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Decision {
    id: String,
    operations: Vec<String>,
    status: Status,
    reason: String,
    decided: String,
    by: String,
    revisit_when: Option<Trigger>,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum Status {
    RawOnly,
    NotSupported,
    Planned,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Trigger {
    Never,
    GithubPrMerged {
        repo: String,
        number: u64,
    },
    CratePublished {
        #[serde(rename = "crate")]
        name: String,
    },
}

fn registry() -> Registry {
    let raw = std::fs::read_to_string(REGISTRY_PATH)
        .unwrap_or_else(|e| panic!("read {REGISTRY_PATH}: {e}"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {REGISTRY_PATH}: {e}"))
}

/// Every operation in the vendored spec, as `METHOD /path`.
fn spec_operations() -> BTreeSet<String> {
    let spec = load_spec();
    let paths = spec["paths"].as_object().expect("the spec has paths");
    paths
        .iter()
        .flat_map(|(path, item)| {
            METHODS
                .iter()
                .filter(|method| item.get(**method).is_some())
                .map(move |method| format!("{} {path}", method.to_uppercase()))
        })
        .collect()
}

/// Whether `pattern` (`METHOD /path`, `*` for any method, a trailing `/**`
/// for the path and everything below it) names `operation`.
fn covers(pattern: &str, operation: &str) -> bool {
    let (Some((want_method, want_path)), Some((method, path))) =
        (pattern.split_once(' '), operation.split_once(' '))
    else {
        return false;
    };
    let method_matches = want_method == "*" || want_method == method;
    let path_matches = match want_path.strip_suffix("/**") {
        Some(prefix) => path == prefix || path.starts_with(&format!("{prefix}/")),
        None => path == want_path,
    };
    method_matches && path_matches
}

#[test]
fn every_operation_a_decision_names_is_in_the_spec() {
    let operations = spec_operations();
    let stale: Vec<String> = registry()
        .decisions
        .iter()
        .flat_map(|decision| {
            decision
                .operations
                .iter()
                .filter(|pattern| !operations.iter().any(|op| covers(pattern, op)))
                .map(move |pattern| format!("{}: {pattern}", decision.id))
        })
        .collect();
    assert!(
        stale.is_empty(),
        "decisions name operations the vendored spec does not declare: {stale:?}\n\
         The spec renamed or removed them. Update the operations in endpoint-decisions.json, \
         or retire the decision when nothing it covered remains."
    );
}

#[test]
fn no_decision_covers_an_endpoint_with_a_shortcut() {
    let contradictions: Vec<String> = registry()
        .decisions
        .iter()
        .flat_map(|decision| {
            SHORTCUT_TEMPLATES
                .iter()
                .map(|(method, path)| format!("{method} {path}"))
                .filter(|op| {
                    decision
                        .operations
                        .iter()
                        .any(|pattern| covers(pattern, op))
                })
                .map(move |op| format!("{}: {op}", decision.id))
        })
        .collect();
    assert!(
        contradictions.is_empty(),
        "decisions cover endpoints that have a shortcut command: {contradictions:?}\n\
         A command now serves them. Narrow the decision's operations in endpoint-decisions.json, \
         or retire the decision."
    );
}

#[test]
fn no_operation_falls_under_two_decisions() {
    let decisions = registry().decisions;
    let doubled: Vec<String> = spec_operations()
        .into_iter()
        .filter_map(|op| {
            let ids: Vec<&str> = decisions
                .iter()
                .filter(|d| d.operations.iter().any(|pattern| covers(pattern, &op)))
                .map(|d| d.id.as_str())
                .collect();
            (ids.len() > 1).then(|| format!("{op}: {ids:?}"))
        })
        .collect();
    assert!(
        doubled.is_empty(),
        "operations fall under more than one decision: {doubled:?}"
    );
}

#[test]
fn every_decision_says_why_when_and_who() {
    let decisions = registry().decisions;
    let mut ids = BTreeSet::new();
    for decision in &decisions {
        let id = &decision.id;
        assert!(
            !id.is_empty()
                && id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
            "decision id {id:?} is not kebab-case"
        );
        assert!(ids.insert(id.clone()), "decision id {id} appears twice");
        assert!(!decision.operations.is_empty(), "{id} names no operation");
        assert!(!decision.reason.trim().is_empty(), "{id} gives no reason");
        assert!(
            !decision.by.trim().is_empty(),
            "{id} says no one decided it"
        );
        let date = decision.decided.as_bytes();
        let iso_date = date.len() == 10
            && date[4] == b'-'
            && date[7] == b'-'
            && date
                .iter()
                .enumerate()
                .all(|(at, b)| at == 4 || at == 7 || b.is_ascii_digit());
        assert!(iso_date, "{id} has no YYYY-MM-DD decided date");
        let trigger = match (&decision.status, &decision.revisit_when) {
            (Status::Planned, None) => continue,
            (Status::Planned, Some(_)) => {
                panic!(
                    "{id} is planned: it is retired when the command ships, so it takes no revisit_when"
                )
            }
            (_, None) => panic!("{id} has no revisit_when"),
            (_, Some(trigger)) => trigger,
        };
        match trigger {
            Trigger::Never => {}
            Trigger::GithubPrMerged { repo, number } => {
                assert!(
                    repo.split_once('/')
                        .is_some_and(|(o, r)| !o.is_empty() && !r.is_empty()),
                    "{id}: repo {repo:?} is not owner/name"
                );
                assert!(*number > 0, "{id}: no pull request number");
            }
            Trigger::CratePublished { name } => {
                assert!(!name.trim().is_empty(), "{id}: no crate name");
            }
        }
    }
}

#[test]
fn every_spec_operation_has_a_shortcut_or_a_decision() {
    let decisions = registry().decisions;
    let shortcuts: BTreeSet<String> = SHORTCUT_TEMPLATES
        .iter()
        .map(|(method, path)| format!("{method} {path}"))
        .collect();
    let undecided: Vec<String> = spec_operations()
        .into_iter()
        .filter(|op| !shortcuts.contains(op))
        .filter(|op| {
            !decisions
                .iter()
                .any(|d| d.operations.iter().any(|pattern| covers(pattern, op)))
        })
        .collect();
    assert!(
        undecided.is_empty(),
        "operations with neither a shortcut nor a decision: {undecided:?}\n\
         Build a command for each (AGENTS.md, \"Adding a command family\"), or propose an \
         endpoint-decisions.json entry for a person to decide."
    );
}

#[test]
fn a_pattern_covers_its_method_and_the_paths_below_a_double_star() {
    assert!(covers("* /2/chat/**", "GET /2/chat/conversations"));
    assert!(covers(
        "* /2/chat/**",
        "POST /2/chat/conversations/{id}/messages"
    ));
    assert!(!covers("* /2/chat/**", "GET /2/chats"));
    assert!(covers("POST /2/calls", "POST /2/calls"));
    assert!(!covers("POST /2/calls", "GET /2/calls/{id}"));
    assert!(!covers("POST /2/calls", "POST /2/calls/{id}/join"));
}
