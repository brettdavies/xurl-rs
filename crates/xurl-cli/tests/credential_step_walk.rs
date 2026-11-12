//! A credential refusal's step, held to what each command's endpoints accept.
//!
//! Every command that takes `--auth` runs for real, in process, against a
//! server that answers every endpoint, over each shape a store can have
//! (nothing, one scheme, another app holding one) and with each scheme the
//! caller can name. When a run is refused for its credential, the refusal
//! carries a step, every scheme its step names and every setup command its
//! message names is one the command's row accepts, and running the step's
//! command gets past the refusal. The command list and each command's arguments are read from
//! clap, so a command added later is checked without an edit here.

mod common;

use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

use regex::Regex;
use serde_json::Value;
use xdk::api::auth_matrix::WireScheme;
use xdk::store::TokenStore;
use xdk::testing::MockX;

use common::invoke::{
    Fill, Ran, SAMPLE, Seed, answering_mock, arguments_for, bearer_only, live_oauth2_only, nothing,
    oauth1_only, run_xr, runnable_paths, takes_a_credential, walk,
};

const OTHER: &str = "other-app";

/// The setup command a message or step names, and the scheme it stores.
static SETUP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"xr (?:--app \S+ )?auth (oauth2|oauth1|app)\b").expect("a pattern")
});
static AUTH_FLAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"--auth (oauth2|oauth1|app)\b").expect("a pattern"));
static APP_FLAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"--app (\S+)").expect("a pattern"));

fn register_other(ts: &mut TokenStore) {
    ts.add_app(OTHER, "CLIENT-ID", "CLIENT-SECRET")
        .expect("add app");
}

fn other_with_bearer(ts: &mut TokenStore) {
    register_other(ts);
    ts.save_bearer_token_for_app(OTHER, "BEARER")
        .expect("save bearer");
}

fn other_with_oauth1(ts: &mut TokenStore) {
    register_other(ts);
    ts.save_oauth1_tokens_for_app(OTHER, "ACCESS", "SECRET", "KEY", "CONSUMER")
        .expect("save oauth1");
}

fn other_with_login(ts: &mut TokenStore) {
    register_other(ts);
    let in_an_hour = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock is past the epoch")
        .as_secs()
        + 3600;
    ts.save_oauth2_token_for_app(OTHER, SAMPLE, "LIVE-ACCESS", "REFRESH", in_an_hour)
        .expect("save oauth2");
}

/// One thing a store can hold: what the active app stores, and the scheme the
/// other registered app stores, if any.
struct Shape {
    name: &'static str,
    seed: Seed,
}

fn shapes() -> Vec<Shape> {
    let fills: [(&str, Fill); 7] = [
        ("an empty app", nothing),
        ("a login", live_oauth2_only),
        ("OAuth1 keys", oauth1_only),
        ("a bearer token", bearer_only),
        (
            "nothing, another app with a bearer token",
            other_with_bearer,
        ),
        ("nothing, another app with OAuth1 keys", other_with_oauth1),
        ("nothing, another app with a login", other_with_login),
    ];
    fills
        .into_iter()
        .map(|(name, fill)| Shape {
            name,
            seed: Seed::of(fill),
        })
        .collect()
}

/// One run: a command, a store shape, and the scheme the caller named.
struct Case {
    path: String,
    shape: usize,
    named: Option<&'static str>,
}

fn is_credential_refusal(document: &Value) -> bool {
    matches!(
        document["reason"].as_str(),
        Some("auth-method-mismatch" | "auth-required")
    )
}

/// The schemes `text` names that `accepted` lacks: those of a setup command
/// and, when `flags` is set, those of an `--auth` flag.
fn refused_schemes(text: &str, accepted: &[WireScheme], flags: bool) -> Vec<String> {
    let flagged = AUTH_FLAG.captures_iter(text).filter(|_| flags);
    SETUP
        .captures_iter(text)
        .chain(flagged)
        .map(|found| found[1].to_string())
        .filter(|name| WireScheme::from_wire(name).is_none_or(|scheme| !accepted.contains(&scheme)))
        .collect()
}

async fn faults(mock: &MockX, shapes: &[Shape], case: &Case) -> Vec<String> {
    let origin = mock.base_url();
    let shape = &shapes[case.shape];
    let Some(accepted) = xurl::cli::accepted_auth::accepted_by_command(&case.path, false) else {
        return Vec::new();
    };
    let mut args = arguments_for(&case.path);
    if let Some(scheme) = case.named {
        args.extend(["--auth".to_string(), scheme.to_string()]);
    }
    let ran = run_xr(&origin, &shape.seed, &case.path, &args, false).await;
    let document = Ran::document(&ran.stderr);
    if !is_credential_refusal(&document) {
        return Vec::new();
    }

    let what = format!(
        "{} over {}{}",
        case.path,
        shape.name,
        case.named
            .map(|scheme| format!(", --auth {scheme}"))
            .unwrap_or_default()
    );
    let mut found = Vec::new();
    let step = &document["next_step"];
    let message = document["message"].as_str().unwrap_or_default();
    let named = [
        (message, false),
        (step["command"].as_str().unwrap_or_default(), true),
        (step["template"].as_str().unwrap_or_default(), true),
    ];

    if step.is_null() {
        found.push(format!(
            "{what}: the refusal carries no next_step: {message}\n  \
             Cause: the refusal's recovery chain has no step for this store shape.\n  \
             Fix: answer it in crates/xurl-cli/src/cli/credential_step.rs."
        ));
    }
    for (text, flags) in named {
        for scheme in refused_schemes(text, &accepted, flags) {
            found.push(format!(
                "{what}: the refusal names {scheme}, which the command's row refuses \
                 ({accepted:?}): {text}\n  \
                 Cause: the message or step is built without the schemes the command accepts.\n  \
                 Fix: build it from the row in crates/xurl-cli/src/cli/credential_step.rs."
            ));
        }
    }

    let followable = matches!(step["action"].as_str(), Some("fix-input" | "select-app"));
    if let (true, Some(command)) = (followable, step["command"].as_str()) {
        let mut rerun = arguments_for(&case.path);
        if let Some(scheme) = AUTH_FLAG.captures(command) {
            rerun.extend(["--auth".to_string(), scheme[1].to_string()]);
        }
        if let Some(app) = APP_FLAG.captures(command) {
            rerun.extend(["--app".to_string(), app[1].to_string()]);
        }
        let after = run_xr(&origin, &shape.seed, &case.path, &rerun, false).await;
        let again = Ran::document(&after.stderr);
        if is_credential_refusal(&again) {
            found.push(format!(
                "{what}: running the step `{command}` is refused again: {}\n  \
                 Cause: the step names an app or a scheme that holds no credential the \
                 command accepts.\n  \
                 Fix: choose it from what the store holds, in \
                 crates/xurl-cli/src/cli/credential_step.rs.",
                again["message"].as_str().unwrap_or_default()
            ));
        }
    }
    found
}

#[test]
fn every_credential_refusal_names_a_step_its_command_accepts() {
    let shapes = shapes();
    let paths: Vec<String> = runnable_paths()
        .into_iter()
        .filter(|path| takes_a_credential(path))
        .collect();
    let mut cases = Vec::new();
    for path in &paths {
        for shape in 0..shapes.len() {
            for named in [None, Some("oauth2"), Some("oauth1"), Some("app")] {
                cases.push(Case {
                    path: path.clone(),
                    shape,
                    named,
                });
            }
        }
    }

    let found: Vec<String> = walk(&cases, answering_mock, async |mock, case: &Case| {
        faults(mock, &shapes, case).await
    })
    .into_iter()
    .flatten()
    .collect();

    assert!(
        found.is_empty(),
        "these credential refusals send a caller somewhere the command does not accept:\n{}",
        found.join("\n")
    );
}
