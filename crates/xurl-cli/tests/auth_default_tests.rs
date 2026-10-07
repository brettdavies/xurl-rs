//! `xr auth default <app> [user]`: one answer for the whole change, and no
//! change at all when either name is wrong.

mod common;

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use xdk::store::TokenStore;

/// A store with two apps. `first` is the default and holds users `bob` (its
/// default) and `alice`; `second` holds `alice`.
fn two_app_store(tmp: &TempDir) -> PathBuf {
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    for (app, users) in [("first", &["bob", "alice"][..]), ("second", &["alice"][..])] {
        ts.add_app(app, "CLIENT-ID-VALUE", "SECRET-VALUE")
            .expect("add_app");
        for user in users {
            ts.save_oauth2_token_for_app(app, user, "user-at", "user-rt", 9_999_999_999)
                .expect("save_oauth2");
        }
    }
    ts.set_default_app("first").expect("set_default_app");
    ts.set_default_user("first", "bob")
        .expect("set_default_user");
    let _ = ts.remove_app("default");
    store
}

fn run(store: &Path, args: &[&str]) -> std::process::Output {
    common::xr_with_store(store)
        .args(args)
        .output()
        .expect("xr runs")
}

/// Every JSON document on `stdout`, in order.
fn documents(stdout: &[u8]) -> Vec<serde_json::Value> {
    serde_json::Deserializer::from_slice(stdout)
        .into_iter::<serde_json::Value>()
        .collect::<Result<_, _>>()
        .expect("stdout holds only JSON documents")
}

/// The default app and the default user of `app`, read back from the file.
fn defaults(store: &Path, app: &str) -> (String, String) {
    let ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    (
        ts.get_default_app().to_string(),
        ts.get_default_user(app).to_string(),
    )
}

/// Setting the app and the user together is one change and answers one
/// document naming both.
#[test]
fn an_app_and_a_user_together_answer_one_document() {
    let tmp = TempDir::new().expect("tempdir");
    let store = two_app_store(&tmp);

    let output = run(
        &store,
        &["--output", "json", "auth", "default", "second", "alice"],
    );

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let documents = documents(&output.stdout);
    assert_eq!(
        documents.len(),
        1,
        "one document for the one command: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(documents[0]["status"], "ok");
    let message = documents[0]["message"].as_str().expect("a message");
    assert!(
        message.contains("\"second\"") && message.contains("\"alice\""),
        "the message names the app and the user: {message}"
    );
    assert_eq!(
        defaults(&store, "second"),
        ("second".to_string(), "alice".to_string())
    );
}

/// The same change reads as one line to a human.
#[test]
fn an_app_and_a_user_together_print_one_line_of_text() {
    let tmp = TempDir::new().expect("tempdir");
    let store = two_app_store(&tmp);

    let output = run(&store, &["auth", "default", "second", "alice"]);

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.lines().count(), 1, "one line: {stdout}");
    assert!(
        stdout.contains("\"second\"") && stdout.contains("\"alice\""),
        "{stdout}"
    );
}

/// An app alone answers one document, as it always has.
#[test]
fn an_app_alone_answers_one_document() {
    let tmp = TempDir::new().expect("tempdir");
    let store = two_app_store(&tmp);

    let output = run(&store, &["--output", "json", "auth", "default", "second"]);

    assert_eq!(output.status.code(), Some(0));
    let documents = documents(&output.stdout);
    assert_eq!(documents.len(), 1);
    assert_eq!(documents[0]["status"], "ok");
    assert_eq!(defaults(&store, "first").0, "second");
}

/// A user the app does not hold fails the command before the default app
/// moves, with nothing on stdout.
#[test]
fn an_unknown_user_changes_nothing() {
    let tmp = TempDir::new().expect("tempdir");
    let store = two_app_store(&tmp);

    let output = run(
        &store,
        &["--output", "json", "auth", "default", "second", "nobody"],
    );

    assert_ne!(output.status.code(), Some(0));
    assert!(
        output.stdout.is_empty(),
        "no success document for a failed command: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(
        defaults(&store, "first"),
        ("first".to_string(), "bob".to_string()),
        "the default app and its default user are as they were"
    );
}

/// An app that is not registered fails the command, and the user named with
/// it is not applied to the default app in its place.
#[test]
fn an_unknown_app_changes_nothing() {
    let tmp = TempDir::new().expect("tempdir");
    let store = two_app_store(&tmp);

    let output = run(
        &store,
        &["--output", "json", "auth", "default", "nope", "alice"],
    );

    assert_ne!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
    assert_eq!(
        defaults(&store, "first"),
        ("first".to_string(), "bob".to_string())
    );
}
