//! A `confirmation-required` refusal names what its dry run names.
//!
//! Every command whose dry run says it would stop to ask runs for real,
//! in process, with `--no-interactive` and no `--force`, over a store
//! holding every kind of credential. The refusal has to carry each key
//! of the command's context that the dry run carries, with the same value,
//! so a caller that reads the refusal learns which post, list, or app the
//! command would have destroyed. It also has to name, as `app` and in its
//! `confirm` template, the app the dry run's credential belongs to, so the
//! template acts on that app wherever it runs. The command list and each
//! command's arguments are read from clap, so a command added later is
//! checked without an edit here.

mod common;

use std::path::{Path, PathBuf};

use serde_json::Value;
use tempfile::TempDir;
use xdk::store::TokenStore;
use xdk::testing::MockX;

use common::invoke::{
    Ran, SAMPLE, Seed, answering_mock, arguments_for, bearer_only, live_oauth2_only, oauth1_only,
    run_xr, runnable_paths, walk,
};

/// The keys a dry run's answer carries of its own, which no refusal repeats.
const DRY_RUN_ENVELOPE: &[&str] = &[
    "status",
    "would_succeed",
    "exit_code",
    "response_schema",
    "auth",
    "confirmation_required",
    "reason",
    "rule",
    "value",
    "message",
    "next_step",
    "endpoint",
    "method",
    "requested",
    "supported",
    "available_in_app",
    "other_apps_with_creds",
];

fn every_kind(ts: &mut TokenStore) {
    live_oauth2_only(ts);
    oauth1_only(ts);
    bearer_only(ts);
}

/// Invocations that set the optional flags a command's context carries.
const WITH_OPTIONS: &[(&str, &[&str])] = &[(
    "calls scheduled cancel",
    &[
        SAMPLE,
        "--no-notify",
        "--scope",
        "this",
        "--occurrence",
        "2026-11-02T16:00:00Z",
    ],
)];

/// A command and the arguments it is run with.
struct Invocation {
    path: String,
    args: Vec<String>,
}

/// What is missing from the refusal of `invocation`, compared with its dry
/// run.
async fn missing_from_refusal(
    mock: &MockX,
    seed: &Seed,
    Invocation { path, args }: &Invocation,
) -> Vec<String> {
    let origin = mock.base_url();
    let asking: Vec<String> = args
        .iter()
        .filter(|arg| *arg != "--force")
        .cloned()
        .collect();

    let dry = run_xr(&origin, seed, path, &asking, true).await;
    let planned = Ran::document(&dry.stdout);
    if planned["confirmation_required"] != true {
        return Vec::new();
    }

    let mut real = asking.clone();
    real.push("--no-interactive".to_string());
    let ran = run_xr(&origin, seed, path, &real, false).await;
    let refusal = Ran::document(&ran.stderr);
    if refusal["reason"] != "confirmation-required" {
        return vec![format!(
            "{path}: its dry run would stop to ask, and the real run answered {refusal}\n  \
             Cause: the real run met a refusal the dry run did not.\n  \
             Fix: make the dry run answer the refusal the real run meets first."
        )];
    }

    let mut found = unpinned_app(path, &planned, &refusal);
    let Value::Object(planned) = planned else {
        return found;
    };
    found.extend(
        planned
            .iter()
            .filter(|(key, _)| !DRY_RUN_ENVELOPE.contains(&key.as_str()))
            .filter(|(key, value)| &refusal[key.as_str()] != *value)
            .map(|(key, value)| {
                format!(
                    "{path}: its dry run carries {key}: {value}, its refusal carries {}\n  \
                 Cause: the confirmation-required body copies a fixed set of the command's \
                 context keys.\n  \
                 Fix: declare `{key}` on ErrorBody in crates/xurl-cli/src/cli/envelope.rs, copy it \
                 in print_confirmation_required in crates/xurl-cli/src/cli/output/mod.rs, and \
                 regenerate schema/output.schema.json.",
                    refusal[key.as_str()]
                )
            }),
    );
    found
}

/// What `refusal` leaves out of the app the dry run `planned` goes out
/// under: the `app` key, or `--app` with that name in the `confirm`
/// template.
fn unpinned_app(path: &str, planned: &Value, refusal: &Value) -> Vec<String> {
    let Some(app) = planned["auth"]["app"].as_str() else {
        return Vec::new();
    };
    let template = refusal["next_step"]["template"]
        .as_str()
        .unwrap_or_default();
    let words: Vec<&str> = template.split_whitespace().collect();
    let pinned = words.windows(2).any(|pair| pair == ["--app", app]);
    if refusal["app"] == app && pinned {
        return Vec::new();
    }
    vec![format!(
        "{path}: its dry run goes out under app {app}, its refusal carries app {} and the \
         template `{template}`\n  \
         Cause: the stop leaves out the app the credential belongs to, so the template acts on \
         whichever app the next run selects.\n  \
         Fix: confirm through `Confirm::gate` in crates/xurl-cli/src/cli/commands/mod.rs, which \
         adds the credential's app to the stop.",
        refusal["app"]
    )]
}

#[test]
fn a_confirmation_refusal_names_what_its_dry_run_names() {
    let seed = Seed::of(every_kind);
    let mut invocations: Vec<Invocation> = runnable_paths()
        .into_iter()
        .map(|path| Invocation {
            args: arguments_for(&path),
            path,
        })
        .collect();
    invocations.extend(WITH_OPTIONS.iter().map(|(path, args)| Invocation {
        path: (*path).to_string(),
        args: args.iter().map(ToString::to_string).collect(),
    }));

    let found: Vec<String> = walk(
        &invocations,
        answering_mock,
        async |mock, run: &Invocation| missing_from_refusal(mock, &seed, run).await,
    )
    .into_iter()
    .flatten()
    .collect();

    assert!(
        found.is_empty(),
        "these confirmation refusals leave out what their dry run names:\n{}",
        found.join("\n")
    );
}

/// A store with two apps, `first` the default, each holding an `OAuth2`
/// login.
fn two_app_store(dir: &Path) -> PathBuf {
    let store = dir.join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    for app in ["first", "second"] {
        ts.add_app(app, "CLIENT-ID", "CLIENT-SECRET")
            .expect("add app");
        ts.save_oauth2_token_for_app(app, "alice", "ACCESS", "REFRESH", 9_999_999_999)
            .expect("save oauth2");
    }
    ts.set_default_app("first").expect("set the default app");
    let _ = ts.remove_app("default");
    store
}

/// `XURL_APP` selects the app, and argv does not show it. The refusal names
/// the app, and its `confirm` template, run where the variable is not set,
/// goes out under the same app.
#[test]
fn the_confirm_template_acts_on_the_app_the_environment_selected() {
    let tmp = TempDir::new().expect("tempdir");
    let store = two_app_store(tmp.path());
    let output = common::xr_with_store(&store)
        .env("API_BASE_URL", common::CLOSED)
        .env("XURL_APP", "second")
        .args(["--output", "json", "--no-interactive", "delete", SAMPLE])
        .output()
        .expect("xr runs");
    let refused: Value = serde_json::from_slice(&output.stderr).expect("a document");
    assert_eq!(refused["reason"], "confirmation-required", "{refused}");
    assert_eq!(refused["app"], "second", "{refused}");
    let template = refused["next_step"]["template"]
        .as_str()
        .unwrap_or_else(|| panic!("no template: {refused}"));
    let words: Vec<&str> = template.split_whitespace().collect();

    let rehearsed = common::xr_with_store(&store)
        .env("API_BASE_URL", common::CLOSED)
        .arg("--dry-run")
        .args(&words[1..])
        .output()
        .expect("xr runs");
    let planned: Value = serde_json::from_slice(&rehearsed.stdout).expect("a document");

    assert_eq!(planned["auth"]["app"], "second", "{template}: {planned}");
}
