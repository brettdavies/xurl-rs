//! Under `--dry-run` no command sends a request or changes what is stored.
//!
//! Every command clap parses is run with the flag, in-process, against a
//! server that records whatever reaches it. The API, the authorize, the
//! token, and the identity endpoints all point there, and the store holds an
//! `OAuth2` login that has expired, so a command that would send its request,
//! resolve a user id first, or refresh the login to do either is caught. The
//! store's bytes and the skill directory are compared before and after.
//!
//! The command list and each command's arguments are read from clap, so a
//! command added later is checked without an edit here. It has to answer the
//! dry-run envelope too, unless it is one of the [`LOCAL_READS`].
//!
//! A command that takes `--auth` sends its request under a credential, and
//! its dry run says which: the scheme, and for an `OAuth2` login the user
//! and whether the token has expired. The second test holds that report to
//! the real run, against a server that answers every endpoint: over each
//! kind of stored credential, the dry run fails for the reason the real run
//! fails for, or names the scheme the real run sent.

mod common;

use serde_json::{Value, json};
use wiremock::MockServer;
use xdk::testing::MockX;

use common::invoke::{
    Fill, OVERRIDES, Ran, SAMPLE, arguments_for, bearer_only, every_kind_with_the_login_expired,
    live_oauth2_only, nothing, oauth1_only, run_xr, runnable_paths, takes_a_credential,
};

/// The commands that only read local state or print what the binary knows.
/// The flag has nothing to stop on them, so they answer as they always do.
/// Every other command answers the dry-run envelope.
const LOCAL_READS: &[&str] = &[
    "auth status",
    "auth apps list",
    "auth apps redirect-uri get",
    "schema",
    "completions",
    "version",
    "examples",
    "validate",
];

/// Raw mode is not a clap command; each row is a request shape.
const RAW_INVOCATIONS: &[(&str, &[&str])] = &[
    ("raw GET", &["/2/users/me"]),
    (
        "raw POST",
        &["-X", "POST", "/2/tweets", "-d", "{\"text\":\"x\"}"],
    ),
    ("raw DELETE", &["-X", "DELETE", "/2/tweets/1234567890"]),
    ("raw stream", &["-s", "/2/tweets/search/stream"]),
    (
        "raw media append",
        &[
            "-X",
            "POST",
            "/2/media/upload/1234567890/append",
            "-F",
            "{file}",
        ],
    ),
];

/// What one dry run did that it must not do, as lines for the failure
/// message, and the document it answered with.
async fn violations(server: &MockServer, name: &str, args: &[String]) -> (Vec<String>, Value) {
    server.reset().await;
    let ran = run_xr(
        &server.uri(),
        every_kind_with_the_login_expired,
        name,
        args,
        true,
    )
    .await;
    let Some(code) = ran.code else {
        return (
            vec![format!("{name}: did not return within 10 seconds")],
            Value::Null,
        );
    };

    let mut found = Vec::new();
    let document = Ran::document(&ran.stdout);
    let answered_dry_run = document["status"] == "dry_run" && code == 0;
    if !LOCAL_READS.contains(&name) && !answered_dry_run {
        found.push(format!(
            "{name}: answered exit {code} with no dry-run envelope for {args:?}: {} {}",
            String::from_utf8_lossy(&ran.stdout).trim(),
            String::from_utf8_lossy(&ran.stderr).trim()
        ));
    }
    if code == 2 {
        found.push(format!(
            "{name}: the arguments {args:?} are a usage error; give the command an OVERRIDES row: {}",
            String::from_utf8_lossy(&ran.stderr).trim()
        ));
    }
    let sent = server.received_requests().await.expect("recording is on");
    if !sent.is_empty() {
        let requests: Vec<String> = sent
            .iter()
            .map(|request| format!("{} {}", request.method, request.url.path()))
            .collect();
        found.push(format!("{name}: sent {requests:?}"));
    }
    if ran.store_changed {
        found.push(format!("{name}: changed the token store"));
    }
    if ran.installed != 0 {
        found.push(format!(
            "{name}: wrote {} entry(ies) into the skill home",
            ran.installed
        ));
    }
    (found, document)
}

/// What is wrong with the credential a dry run over
/// [`every_kind_with_the_login_expired`] reports, if anything.
fn credential_report_fault(document: &Value) -> Option<String> {
    let auth = &document["auth"];
    if auth["app"] != SAMPLE {
        return Some(format!(
            "does not name the app whose credential it would send: {document}"
        ));
    }
    match auth["scheme"].as_str() {
        Some("oauth2") if auth["username"] == SAMPLE && auth["token_expired"] == true => None,
        Some("oauth1" | "app") => None,
        _ => Some(format!(
            "names no credential, or not the stored login as expired: {document}"
        )),
    }
}

/// A row that names no command would silently stop applying.
#[test]
fn every_row_names_a_command() {
    let runnable = runnable_paths();
    let stale: Vec<&str> = OVERRIDES
        .iter()
        .map(|(name, _)| *name)
        .chain(LOCAL_READS.iter().copied())
        .filter(|name| !runnable.contains(*name))
        .collect();
    assert!(
        stale.is_empty(),
        "OVERRIDES or LOCAL_READS rows naming no command: {stale:?}"
    );
}

#[tokio::test]
async fn a_dry_run_sends_nothing_and_stores_nothing() {
    let server = MockServer::start().await;
    let mut found = Vec::new();
    for path in runnable_paths() {
        let args = arguments_for(&path);
        let (violations_found, document) = violations(&server, &path, &args).await;
        found.extend(violations_found);
        if takes_a_credential(&path)
            && let Some(fault) = credential_report_fault(&document)
        {
            found.push(format!("{path}: {fault}"));
        }
        if !args.iter().any(|arg| arg == "--force") {
            continue;
        }
        // The flag wins over the confirmation a destructive command asks
        // for: without `--force` the dry run still answers, and says the
        // real run would stop for confirmation.
        if !document["confirmation_required"].is_null() {
            found.push(format!(
                "{path}: reports confirmation_required though --force was given: {document}"
            ));
        }
        let unforced: Vec<String> = args.into_iter().filter(|arg| arg != "--force").collect();
        let (violations_found, document) = violations(&server, &path, &unforced).await;
        found.extend(violations_found);
        if document["confirmation_required"] != true {
            found.push(format!(
                "{path}: without --force the dry run does not report confirmation_required: {document}"
            ));
        }
    }
    for (name, args) in RAW_INVOCATIONS {
        let args: Vec<String> = args.iter().map(|arg| (*arg).to_string()).collect();
        let (violations_found, document) = violations(&server, name, &args).await;
        found.extend(violations_found);
        if let Some(fault) = credential_report_fault(&document) {
            found.push(format!("{name}: {fault}"));
        }
    }
    assert!(
        found.is_empty(),
        "under --dry-run a command reached the network, changed local state, or left its \
         credential unreported:\n{}",
        found.join("\n")
    );
}

/// The scheme a request went out under, read from its `Authorization`
/// header and the token values the seeds store.
fn scheme_sent(headers: &[(String, String)]) -> Option<&'static str> {
    let (_, value) = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("authorization"))?;
    match value.as_str() {
        "Bearer LIVE-ACCESS" => Some("oauth2"),
        "Bearer BEARER" => Some("app"),
        signed if signed.starts_with("OAuth ") => Some("oauth1"),
        _ => None,
    }
}

#[tokio::test]
async fn a_dry_run_reports_the_credential_the_real_run_uses() {
    let mock = MockX::start().await;
    // The media fixtures report processing under way, which a real upload
    // waits on. A finished one has nothing to wait for.
    let done = json!({"data": {"id": SAMPLE, "processing_info": {"state": "succeeded"}}});
    mock.stub("POST", "/finalize$", 200, done.clone()).await;
    mock.stub("GET", "^/2/media/upload$", 200, done).await;
    let origin = mock.base_url();

    let seeds: [(&str, Fill); 4] = [
        ("no credential", nothing),
        ("an app-only bearer", bearer_only),
        ("an OAuth1 token", oauth1_only),
        ("a live OAuth2 login", live_oauth2_only),
    ];
    let mut invocations: Vec<(String, Vec<String>)> = runnable_paths()
        .into_iter()
        .filter(|path| takes_a_credential(path))
        .map(|path| {
            let args = arguments_for(&path);
            (path, args)
        })
        .collect();
    invocations.extend(RAW_INVOCATIONS.iter().map(|(name, args)| {
        let args = args.iter().map(|arg| (*arg).to_string()).collect();
        ((*name).to_string(), args)
    }));
    assert!(
        invocations.len() > RAW_INVOCATIONS.len(),
        "no command takes --auth; the filter reads the wrong argument id"
    );

    let mut found = Vec::new();
    for (label, fill) in seeds {
        for (name, args) in &invocations {
            let case = format!("{name} with {label}");
            let seen = mock.requests().await.len();
            let real = run_xr(&origin, fill, name, args, false).await;
            let sent = mock.requests().await.split_off(seen);
            let dry = run_xr(&origin, fill, name, args, true).await;
            if mock.requests().await.len() != seen + sent.len() {
                found.push(format!("{case}: the dry run sent a request"));
            }
            let Some(real_code) = real.code else {
                found.push(format!("{case}: the real run did not return"));
                continue;
            };
            let refusal = Ran::document(&real.stderr);
            let document = Ran::document(&dry.stdout);
            let stopped_on_a_credential = matches!(
                refusal["reason"].as_str(),
                Some("auth-required" | "auth-method-mismatch")
            );
            if stopped_on_a_credential {
                if document["would_succeed"] != false
                    || document["reason"] != refusal["reason"]
                    || document["exit_code"] != real_code
                {
                    found.push(format!(
                        "{case}: the real run stopped on {} at exit {real_code}, and the dry run \
                         answered {document}",
                        refusal["reason"]
                    ));
                }
                continue;
            }
            let Some(scheme) = sent
                .last()
                .and_then(|request| scheme_sent(&request.headers))
            else {
                found.push(format!(
                    "{case}: the real run exited {real_code} without stopping on a credential \
                     and without sending one: {}",
                    String::from_utf8_lossy(&real.stderr).trim()
                ));
                continue;
            };
            if document["would_succeed"] != true
                || document["auth"]["scheme"] != scheme
                || document["auth"]["app"] != SAMPLE
            {
                found.push(format!(
                    "{case}: the real run sent {scheme} from app {SAMPLE}, and the dry run answered \
                     {document}"
                ));
            }
        }
    }
    assert!(
        found.is_empty(),
        "a dry run and the real run disagree on the credential:\n{}",
        found.join("\n")
    );
}
