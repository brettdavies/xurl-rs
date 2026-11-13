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
//!
//! Raw requests have no row. Each is held to what the auth matrix lists for
//! its method and path, over one endpoint for every distinct set of schemes
//! the matrix declares and one path it does not list.

mod common;

use std::collections::BTreeMap;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

use regex::Regex;
use serde_json::Value;
use xdk::api::auth_matrix::{AUTH_MATRIX, WireScheme};
use xdk::store::TokenStore;
use xdk::testing::MockX;

use common::invoke::{
    Fill, Ran, SAMPLE, Seed, answering_mock, arguments_for, bearer_only, command_at,
    live_oauth2_only, nothing, oauth1_only, run_xr, runnable_paths, takes_a_credential, walk,
};

const OTHER: &str = "other-app";

/// The setup command a message or step names, and the scheme it stores.
static SETUP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"xr (?:--app \S+ )?auth (oauth2|oauth1|app)\b").expect("a pattern")
});
static TAIL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Add credentials with: (xr auth .+)\.$").expect("a pattern"));
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

/// One thing a store can hold: what the active app stores (`holds`), and the
/// scheme the other registered app stores, if any.
struct Shape {
    name: &'static str,
    seed: Seed,
    holds: &'static [WireScheme],
}

fn shapes() -> Vec<Shape> {
    let fills: [(&str, Fill, &[WireScheme]); 7] = [
        ("an empty app", nothing, &[]),
        ("a login", live_oauth2_only, &[WireScheme::OAuth2]),
        ("OAuth1 keys", oauth1_only, &[WireScheme::OAuth1]),
        ("a bearer token", bearer_only, &[WireScheme::App]),
        (
            "nothing, another app with a bearer token",
            other_with_bearer,
            &[],
        ),
        (
            "nothing, another app with OAuth1 keys",
            other_with_oauth1,
            &[],
        ),
        ("nothing, another app with a login", other_with_login, &[]),
    ];
    fills
        .into_iter()
        .map(|(name, fill, holds)| Shape {
            name,
            seed: Seed::of(fill),
            holds,
        })
        .collect()
}

/// What a run exercises: how it is invoked, the schemes its requests
/// accept, the arguments that stand for it without a scheme, and whether it
/// also runs with a named login.
struct Target {
    name: String,
    args: Vec<String>,
    accepted: Vec<WireScheme>,
    walks_login: bool,
}

/// One run: a target, a store shape, and the scheme the caller named.
struct Case {
    target: Target,
    shape: usize,
    named: Option<&'static str>,
    login: bool,
}

/// A login no store holds, for a run that names one with `-u`.
const UNHELD_LOGIN: &str = "nobody-holds-this-login";

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
    let Target {
        name,
        args: base,
        accepted,
        ..
    } = &case.target;
    let mut args = base.clone();
    if let Some(scheme) = case.named {
        args.extend(["--auth".to_string(), scheme.to_string()]);
    }
    if case.login {
        args.extend(["-u".to_string(), UNHELD_LOGIN.to_string()]);
    }
    let ran = run_xr(&origin, &shape.seed, name, &args, false).await;
    let document = Ran::document(&ran.stderr);
    if !is_credential_refusal(&document) {
        return Vec::new();
    }

    let what = format!(
        "{} over {}{}",
        case.target.name,
        shape.name,
        case.named
            .map(|scheme| format!(", --auth {scheme}"))
            .unwrap_or_default()
            + if case.login { ", -u" } else { "" }
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
        for scheme in refused_schemes(text, accepted, flags) {
            found.push(format!(
                "{what}: the refusal names {scheme}, which the command or endpoint refuses \
                 ({accepted:?}): {text}\n  \
                 Cause: the message or step is built without the schemes the request accepts.\n  \
                 Fix: build it from the schemes it accepts, in crates/xurl-cli/src/cli/credential_step.rs."
            ));
        }
    }

    if let (Some(tail), Some(template)) = (
        TAIL.captures(message),
        step["template"].as_str().filter(|t| SETUP.is_match(t)),
    ) && &tail[1] != template
    {
        found.push(format!(
            "{what}: the message says to run `{}` and the step says `{template}`\n  \
             Cause: the message tail and the step choose the scheme or the app separately.\n  \
             Fix: build both with setup_invocation in \
             crates/xurl-cli/src/cli/credential_step.rs.",
            &tail[1]
        ));
    }
    let binds = case.login && matches!(case.named, None | Some("oauth2"));
    if binds && accepted.contains(&WireScheme::OAuth2) {
        for (text, _) in &named[1..] {
            if let Some(other) = AUTH_FLAG
                .captures(text)
                .filter(|scheme| &scheme[1] != "oauth2")
            {
                found.push(format!(
                    "{what}: the refusal sends a caller who named a login to --auth {}: {text}\n  \
                     Cause: -u selects an OAuth2 login, and the step names a scheme that has none.\n  \
                     Fix: let only OAuth2 held by an app with the login serve it, in \
                     crates/xurl-cli/src/cli/credential_step.rs.",
                    &other[1]
                ));
            }
        }
    }

    if let Some(stored) = step["template"].as_str().and_then(|t| SETUP.captures(t)) {
        let template = step["template"].as_str().unwrap_or_default();
        let app_is_active = APP_FLAG
            .captures(template)
            .is_none_or(|app| &app[1] != OTHER);
        let scheme = WireScheme::from_wire(&stored[1]);
        if app_is_active && scheme.is_some_and(|scheme| shape.holds.contains(&scheme)) {
            found.push(format!(
                "{what}: the step `{template}` stores a credential the app already holds\n  \
                 Cause: a held scheme the request accepts was passed over, so the same refusal \
                 comes back after the credential is stored again.\n  \
                 Fix: let the held scheme serve, or take the sign-in, in \
                 crates/xurl-cli/src/cli/credential_step.rs."
            ));
        }
    }

    let followable = matches!(step["action"].as_str(), Some("fix-input" | "select-app"));
    if let (true, Some(command)) = (followable, step["command"].as_str()) {
        let mut rerun = base.clone();
        if let Some(scheme) = AUTH_FLAG.captures(command) {
            rerun.extend(["--auth".to_string(), scheme[1].to_string()]);
        }
        if let Some(app) = APP_FLAG.captures(command) {
            rerun.extend(["--app".to_string(), app[1].to_string()]);
        }
        if case.login && command.contains(UNHELD_LOGIN) {
            rerun.extend(["-u".to_string(), UNHELD_LOGIN.to_string()]);
        }
        let after = run_xr(&origin, &shape.seed, name, &rerun, false).await;
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

/// One test per way a caller names a scheme, so each has its own time limit
/// and a partitioned run can put them on different machines.
#[rstest::rstest]
#[case::unnamed(None)]
#[case::oauth2(Some("oauth2"))]
#[case::oauth1(Some("oauth1"))]
#[case::app(Some("app"))]
fn every_credential_refusal_names_a_step_its_command_accepts(#[case] named: Option<&'static str>) {
    let shapes = shapes();
    let paths: Vec<String> = runnable_paths()
        .into_iter()
        .filter(|path| takes_a_credential(path))
        .collect();
    let mut targets: Vec<Target> = paths
        .iter()
        .filter_map(|path| {
            Some(Target {
                name: path.clone(),
                args: arguments_for(path),
                accepted: xurl::cli::accepted_auth::accepted_by_command(path, false)?,
                walks_login: command_at(path)
                    .get_arguments()
                    .any(|arg| arg.get_id() == "username"),
            })
        })
        .collect();
    // A named login is served by the schemes the request accepts, the store,
    // and the scheme named, none of which the command changes, so one command
    // per set of accepted schemes runs with `-u`.
    let mut with_login: Vec<Vec<WireScheme>> = Vec::new();
    for target in &mut targets {
        if target.walks_login {
            target.walks_login = !with_login.contains(&target.accepted);
            if target.walks_login {
                with_login.push(target.accepted.clone());
            }
        }
    }

    let found = walked(&shapes, targets, &[named]);
    assert!(
        found.is_empty(),
        "these credential refusals send a caller somewhere the command does not accept:\n{}",
        found.join("\n")
    );
}

/// Every target over every store shape and each way of naming a scheme in
/// `namings`.
fn walked(shapes: &[Shape], targets: Vec<Target>, namings: &[Option<&'static str>]) -> Vec<String> {
    let mut cases = Vec::new();
    for target in targets {
        for shape in 0..shapes.len() {
            for &named in namings {
                for login in [false, true] {
                    if login && !target.walks_login {
                        continue;
                    }
                    cases.push(Case {
                        target: Target {
                            name: target.name.clone(),
                            args: target.args.clone(),
                            accepted: target.accepted.clone(),
                            walks_login: target.walks_login,
                        },
                        shape,
                        named,
                        login,
                    });
                }
            }
        }
    }
    walk(&cases, answering_mock, async |mock, case: &Case| {
        faults(mock, shapes, case).await
    })
    .into_iter()
    .flatten()
    .collect()
}

/// A path the matrix does not list, so a request to it is held to no scheme.
const UNLISTED: &str = "/2/never/heard/of";

/// One concrete endpoint for every distinct set of schemes the matrix
/// declares, spelled once as a literal path and once as a template filled
/// with an id, and one path it does not list. A filled template is held to
/// the schemes of the matrix entry it came from, whatever the path says.
fn raw_targets() -> Vec<Target> {
    type Endpoint = (String, String, Vec<WireScheme>);
    let mut by_set: BTreeMap<(Vec<&'static str>, bool), Endpoint> = BTreeMap::new();
    for (key, schemes) in AUTH_MATRIX.entries() {
        let Some((method, template)) = key.split_once('\0') else {
            continue;
        };
        let mut set: Vec<&'static str> = schemes
            .iter()
            .map(|scheme| scheme.wire().as_wire())
            .collect();
        set.sort_unstable();
        let accepted: Vec<WireScheme> = WireScheme::ALL_BY_PREFERENCE
            .into_iter()
            .filter(|scheme| schemes.iter().any(|entry| entry.wire() == *scheme))
            .collect();
        let templated = template.contains('{');
        let path = template
            .split('/')
            .map(|segment| {
                if segment.starts_with('{') {
                    SAMPLE
                } else {
                    segment
                }
            })
            .collect::<Vec<_>>()
            .join("/");
        by_set
            .entry((set, templated))
            .or_insert_with(|| (method.to_string(), path, accepted));
    }
    let mut endpoints: Vec<Endpoint> = by_set.into_values().collect();
    endpoints.push((
        "GET".to_string(),
        UNLISTED.to_string(),
        WireScheme::ALL_BY_PREFERENCE.to_vec(),
    ));
    endpoints
        .into_iter()
        .map(|(method, path, accepted)| Target {
            name: format!("raw {method} {path}"),
            args: vec!["-X".to_string(), method, path],
            accepted,
            walks_login: true,
        })
        .collect()
}

#[test]
fn every_raw_credential_refusal_names_a_step_its_endpoint_accepts() {
    let shapes = shapes();
    let targets = raw_targets();
    assert!(
        targets.len() > 3,
        "the matrix declares few distinct scheme sets: {}",
        targets.len()
    );

    let found = walked(
        &shapes,
        targets,
        &[None, Some("oauth2"), Some("oauth1"), Some("app")],
    );
    assert!(
        found.is_empty(),
        "these credential refusals send a raw request somewhere its endpoint does not accept:\n{}",
        found.join("\n")
    );
}
