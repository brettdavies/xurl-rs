//! What a step keeps of the run it answers.
//!
//! Every runnable command runs for real, in process, over each store shape,
//! as a real run, a dry run, and a run that names an app. Each `next_step` in
//! each document it prints is held to the rules in
//! `crates/xurl-cli/src/cli/rerun.rs`: a step that runs `xr` keeps the output
//! format the caller asked for (help text excepted), a step that runs the
//! invocation again keeps `--dry-run`, and a step that signs in names the app
//! the failed run used. The command list comes from clap, so a command added
//! later is checked without an edit here.
//!
//! Two more walks read the declarations the rules are written against: clap's
//! global flags, each of which the rules carry or drop with a reason, and the
//! schema's list of actions, each of which has a row in the format policy.

mod common;

use std::collections::BTreeSet;

use clap::CommandFactory;
use serde_json::Value;
use xdk::testing::MockX;
use xurl::cli::Cli;
use xurl::cli::rerun::{CARRIED_GLOBALS, DROPPED_GLOBALS, policy_actions};

use common::invoke::{Ran, answering_mock, arguments_for, run_xr, runnable_paths, walk};
use common::shapes::{OTHER, Shape, shapes};

/// How many tests the walk is split over, each with its own time limit.
const CHUNKS: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Real,
    Dry,
    NamedApp,
}

impl Kind {
    const ALL: [Self; 3] = [Self::Real, Self::Dry, Self::NamedApp];

    fn label(self) -> &'static str {
        match self {
            Self::Real => "a real run",
            Self::Dry => "a dry run",
            Self::NamedApp => "a run under --app",
        }
    }
}

/// One run: a command, a store shape, and how it is invoked. `args` stand in
/// for the arguments clap's declarations give the command.
struct Case {
    path: String,
    shape: usize,
    kind: Kind,
    args: Option<Vec<String>>,
}

/// A step as a document carries it.
struct Seen {
    action: String,
    text: String,
}

fn steps_in(document: &Value) -> Vec<Seen> {
    let step = &document["next_step"];
    let Some(action) = step["action"].as_str() else {
        return Vec::new();
    };
    ["command", "template"]
        .into_iter()
        .filter_map(|key| step[key].as_str())
        .map(|text| Seen {
            action: action.to_string(),
            text: text.to_string(),
        })
        .collect()
}

/// The words of `text` from its `xr` word on, or `None` when it names none.
fn after_program(text: &str) -> Option<Vec<&str>> {
    let words: Vec<&str> = text.split(' ').collect();
    let at = words.iter().position(|word| *word == "xr")?;
    Some(words[at + 1..].to_vec())
}

fn names_json_output(words: &[&str]) -> bool {
    words.windows(2).any(|pair| pair == ["--output", "json"])
        || words
            .iter()
            .any(|word| *word == "--output=json" || *word == "--json")
}

fn names(words: &[&str], flag: &str) -> bool {
    words.iter().any(|word| {
        *word == flag
            || word
                .strip_prefix(flag)
                .is_some_and(|rest| rest.starts_with('='))
    })
}

/// Whether `words` run the command at `path`: its words appear in order.
fn runs(words: &[&str], path: &str) -> bool {
    let wanted: Vec<&str> = path.split(' ').collect();
    words.windows(wanted.len()).any(|window| window == wanted)
}

/// What one run answered with: the faults in its steps, and the action of
/// each step it carried.
struct Walked {
    faults: Vec<String>,
    actions: Vec<String>,
}

async fn visit(mock: &MockX, shapes: &[Shape], case: &Case) -> Walked {
    let shape = &shapes[case.shape];
    let mut args = case
        .args
        .clone()
        .unwrap_or_else(|| arguments_for(&case.path));
    if case.kind == Kind::NamedApp {
        args.extend(["--app".to_string(), OTHER.to_string()]);
    }
    let ran = run_xr(
        &mock.base_url(),
        &shape.seed,
        &case.path,
        &args,
        case.kind == Kind::Dry,
    )
    .await;
    let what = format!("{} over {}, {}", case.path, shape.name, case.kind.label());
    let mut walked = Walked {
        faults: Vec::new(),
        actions: Vec::new(),
    };
    for stream in [&ran.stderr, &ran.stdout] {
        let document = Ran::document(stream);
        for step in steps_in(&document) {
            walked.faults.extend(check(&what, case, &document, &step));
            walked.actions.push(step.action);
        }
    }
    walked
}

fn check(what: &str, case: &Case, document: &Value, step: &Seen) -> Vec<String> {
    let Some(words) = after_program(&step.text) else {
        return vec![format!(
            "{what}: the {} step `{}` names no `xr` word\n  \
             Cause: a step that runs nothing the caller can follow.\n  \
             Fix: build it as a command or template that names `xr`.",
            step.action, step.text
        )];
    };
    let mut found = Vec::new();
    let reruns = runs(&words, &case.path);
    if step.action != "show-help" && !names_json_output(&words) {
        found.push(format!(
            "{what}: the {} step `{}` drops --output json\n  \
             Cause: the step reached the document without the run's output format.\n  \
             Fix: a step reaches a document through OutputConfig::carry, \
             crates/xurl-cli/src/cli/rerun.rs.",
            step.action, step.text
        ));
    }
    if case.kind == Kind::Dry && step.action != "show-help" && reruns != names(&words, "--dry-run")
    {
        found.push(format!(
            "{what}: the {} step `{}` {} --dry-run\n  \
             Cause: a step that runs the invocation again keeps the rehearsal, and one that \
             runs another command does not.\n  \
             Fix: build it from the invocation explicit_invocation writes, \
             crates/xurl-cli/src/cli/rerun.rs.",
            step.action,
            step.text,
            if reruns { "drops" } else { "carries" }
        ));
    }
    if case.kind != Kind::Dry && names(&words, "--dry-run") {
        found.push(format!(
            "{what}: the {} step `{}` carries --dry-run into a real run\n  \
             Cause: the step was built from a rehearsal's invocation.\n  \
             Fix: read the flag from the parsed Cli in explicit_invocation, \
             crates/xurl-cli/src/cli/rerun.rs.",
            step.action, step.text
        ));
    }
    if case.kind == Kind::NamedApp
        && document["reason"] != "token-store"
        && step.action == "sign-in"
        && !words.windows(2).any(|pair| pair == ["--app", OTHER])
    {
        found.push(format!(
            "{what}: the sign-in step `{}` does not name {OTHER}\n  \
             Cause: the step leaves the app to the caller's shell, which resolves another one.\n  \
             Fix: name the app the run used, the default one included, where the sign-in step is \
             chosen in crates/xurl-cli/src/cli/hints.rs.",
            step.text
        ));
    }
    found
}

fn cases(chunk: usize) -> Vec<Case> {
    let shapes = shapes();
    runnable_paths()
        .into_iter()
        .enumerate()
        .filter(|(index, _)| index % CHUNKS == chunk)
        .flat_map(|(_, path)| {
            (0..shapes.len()).flat_map(move |shape| {
                let path = path.clone();
                Kind::ALL.into_iter().map(move |kind| Case {
                    path: path.clone(),
                    shape,
                    kind,
                    args: None,
                })
            })
        })
        .collect()
}

fn walked(cases: &[Case]) -> Vec<Walked> {
    let shapes = shapes();
    walk(cases, answering_mock, async |mock, case: &Case| {
        visit(mock, &shapes, case).await
    })
}

#[rstest::rstest]
#[case::first(0)]
#[case::second(1)]
#[case::third(2)]
#[case::fourth(3)]
fn every_step_keeps_what_the_run_it_answers_asked_for(#[case] chunk: usize) {
    let cases = cases(chunk);
    assert!(!cases.is_empty(), "chunk {chunk} holds no command");
    let found: BTreeSet<String> = walked(&cases)
        .into_iter()
        .flat_map(|walked| walked.faults)
        .collect();
    assert!(
        found.is_empty(),
        "these steps drop what the run they answer asked for:\n{}",
        found.into_iter().collect::<Vec<_>>().join("\n")
    );
}

/// One run the walk makes to show it reaches an action.
struct Reach {
    action: &'static str,
    path: &'static str,
    shape: &'static str,
    kind: Kind,
    args: Option<&'static [&'static str]>,
}

impl Reach {
    fn new(
        action: &'static str,
        path: &'static str,
        shape: &'static str,
        kind: Kind,
        args: Option<&'static [&'static str]>,
    ) -> Self {
        Self {
            action,
            path,
            shape,
            kind,
            args,
        }
    }
}

/// A run that reaches each action the walk can reach, so a walk that stops
/// reaching one fails here instead of passing with nothing to check. The
/// shapes the walk runs over all hold an app with a client id, so the actions
/// that need an empty or unreadable store, or an answer from X, are held by
/// `rerun_env_tests.rs` and the unit tests of `carry`.
#[test]
fn the_walk_reaches_each_action_it_can() {
    let shapes = shapes();
    let at = |name: &str| {
        shapes
            .iter()
            .position(|shape| shape.name == name)
            .unwrap_or_else(|| panic!("no store shape named {name:?}"))
    };
    let reach = [
        Reach::new("sign-in", "whoami", "an empty app", Kind::Real, None),
        Reach::new(
            "select-app",
            "whoami",
            "nothing, another app with a login",
            Kind::Real,
            None,
        ),
        Reach::new("fix-input", "whoami", "an empty app", Kind::NamedApp, None),
        Reach::new(
            "show-help",
            "like",
            "an empty app",
            Kind::Dry,
            Some(&["not-an-id"]),
        ),
        Reach::new(
            "confirm",
            "auth clear",
            "an empty app",
            Kind::Real,
            Some(&["--all"]),
        ),
    ];
    let cases: Vec<Case> = reach
        .iter()
        .map(|run| Case {
            path: run.path.to_string(),
            shape: at(run.shape),
            kind: run.kind,
            args: run
                .args
                .map(|args| args.iter().map(|arg| (*arg).to_string()).collect()),
        })
        .collect();
    let mut missed = Vec::new();
    for (run, walked) in reach.iter().zip(walked(&cases)) {
        if !walked.actions.iter().any(|seen| seen == run.action) {
            missed.push(format!(
                "{} over {}, {}: no {} step; it answered {:?}\n  \
                 Cause: the walk no longer reaches this action, so it checks nothing for it.\n  \
                 Fix: name a run that reaches it in the reach list of this test.",
                run.path,
                run.shape,
                run.kind.label(),
                run.action,
                walked.actions
            ));
        }
        assert!(walked.faults.is_empty(), "{}", walked.faults.join("\n"));
    }
    assert!(missed.is_empty(), "{}", missed.join("\n"));
}

/// A structured document's instruction can name a command outside any
/// `next_step`, and nothing in the type of that string sends it through
/// `carry`, so the one that exists is held here.
#[test]
fn an_instruction_that_names_a_command_names_the_output_format() {
    let shapes = shapes();
    let ran = walk(&[0_usize], answering_mock, async |mock, shape: &usize| {
        run_xr(
            &mock.base_url(),
            &shapes[*shape].seed,
            "auth oauth2",
            &["--no-browser".to_string()],
            false,
        )
        .await
    })
    .remove(0);
    let document = Ran::document(&ran.stdout);
    let instructions = document["instructions"].as_str().unwrap_or_default();
    assert!(
        instructions.contains("xr --output json auth oauth2"),
        "the instruction drops --output json: {document}\n  \
         Cause: a command named in prose reaches the document without the run's output format.\n  \
         Fix: pass it through rerun::name_format, crates/xurl-cli/src/cli/rerun.rs."
    );
}

#[test]
fn every_global_flag_is_carried_or_dropped_with_a_reason() {
    let command = Cli::command();
    let globals: BTreeSet<String> = command
        .get_arguments()
        .filter(|arg| arg.is_global_set())
        .map(|arg| arg.get_id().to_string())
        .collect();
    let decided: BTreeSet<&str> = CARRIED_GLOBALS
        .iter()
        .copied()
        .chain(DROPPED_GLOBALS.iter().map(|(id, _)| *id))
        .collect();
    let undecided: Vec<&String> = globals
        .iter()
        .filter(|id| !decided.contains(id.as_str()))
        .collect();
    let stale: Vec<&&str> = decided
        .iter()
        .filter(|id| !globals.contains(**id))
        .collect();
    let unexplained: Vec<&str> = DROPPED_GLOBALS
        .iter()
        .filter(|(_, reason)| reason.trim().is_empty())
        .map(|(id, _)| *id)
        .collect();
    assert!(
        undecided.is_empty() && stale.is_empty() && unexplained.is_empty(),
        "global flags and the rules for what a step keeps of them disagree:\n  \
         undecided: {undecided:?}\n  stale: {stale:?}\n  no reason: {unexplained:?}\n  \
         Cause: a global flag was added, renamed, or dropped without deciding whether a step \
         keeps it.\n  \
         Fix: list its clap id in CARRIED_GLOBALS, or in DROPPED_GLOBALS with the reason, in \
         crates/xurl-cli/src/cli/rerun.rs."
    );
}

#[test]
fn every_action_has_a_row_in_the_format_policy() {
    let schema = serde_json::to_value(schemars::schema_for!(xurl::cli::hints::NextAction))
        .expect("a schema");
    let mut declared: BTreeSet<String> = BTreeSet::new();
    collect_names(&schema, &mut declared);
    assert!(
        declared.len() > 5,
        "the schema names few actions: {declared:?}"
    );
    let rows: BTreeSet<String> = policy_actions()
        .into_iter()
        .map(|action| {
            serde_json::to_value(action)
                .ok()
                .and_then(|name| name.as_str().map(str::to_string))
                .unwrap_or_default()
        })
        .collect();
    let missing: Vec<&String> = declared.difference(&rows).collect();
    let stale: Vec<&String> = rows.difference(&declared).collect();
    assert!(
        missing.is_empty() && stale.is_empty(),
        "the format policy and the actions disagree:\n  missing: {missing:?}\n  stale: {stale:?}\n  \
         Cause: an action was added or removed without deciding whether its step keeps the \
         output format.\n  \
         Fix: give it a row in FORMAT_POLICY, crates/xurl-cli/src/cli/rerun.rs."
    );
}

/// Every string a schema's `enum` or `const` names.
fn collect_names(schema: &Value, names: &mut BTreeSet<String>) {
    match schema {
        Value::Object(map) => {
            for (key, value) in map {
                match (key.as_str(), value) {
                    ("enum", Value::Array(items)) => {
                        names.extend(items.iter().filter_map(Value::as_str).map(str::to_string));
                    }
                    ("const", Value::String(name)) => {
                        names.insert(name.clone());
                    }
                    _ => collect_names(value, names),
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect_names(item, names)),
        _ => {}
    }
}
