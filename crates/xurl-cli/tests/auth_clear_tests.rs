//! `xr auth clear`: it acts on the app `--app` selects, and on the default
//! app when none is named.

mod common;

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use xdk::store::TokenStore;

/// What an app holds: whether it has `OAuth1` tokens, its `OAuth2` logins, and
/// whether it has a bearer token.
type Held = (bool, Vec<String>, bool);

/// A store with two apps, `first` (the default) and `second`, each holding
/// `OAuth1` tokens, an `OAuth2` login `alice`, and a bearer token.
fn two_app_store(tmp: &TempDir) -> PathBuf {
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    for app in ["first", "second"] {
        ts.add_app(app, "CLIENT-ID-VALUE", "SECRET-VALUE")
            .expect("add_app");
        ts.save_oauth2_token_for_app(app, "alice", "user-at", "user-rt", 9_999_999_999)
            .expect("save_oauth2");
        ts.save_oauth1_tokens_for_app(app, "oa1-at", "oa1-secret", "oa1-key", "oa1-consumer")
            .expect("save_oauth1");
        ts.save_bearer_token_for_app(app, "bearer-value")
            .expect("save_bearer");
    }
    ts.set_default_app("first").expect("set_default_app");
    let _ = ts.remove_app("default");
    store
}

fn held(store: &Path, app: &str) -> Held {
    let ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    let app = ts.get_app(app).expect("the app survives a clear");
    let mut logins: Vec<String> = app.oauth2_tokens.keys().cloned().collect();
    logins.sort();
    (
        app.oauth1_token.is_some(),
        logins,
        app.bearer_token.is_some(),
    )
}

fn run(store: &Path, args: &[&str]) -> std::process::Output {
    common::xr_with_store(store)
        .args(args)
        .output()
        .expect("xr runs")
}

fn everything() -> Held {
    (true, vec!["alice".to_string()], true)
}

/// A selector with `--app` clears its own credential in that app, and the
/// default app keeps everything.
#[rstest::rstest]
#[case::all(&["--all"], (false, vec![], false))]
#[case::oauth1(&["--oauth1"], (false, vec!["alice".to_string()], true))]
#[case::oauth2(&["--oauth2-username", "alice"], (true, vec![], true))]
#[case::bearer(&["--bearer"], (true, vec!["alice".to_string()], false))]
fn a_clear_acts_on_the_app_app_selects(#[case] selector: &[&str], #[case] left: Held) {
    let tmp = TempDir::new().expect("tempdir");
    let store = two_app_store(&tmp);
    let mut args = vec!["--app", "second", "auth", "clear", "--force"];
    args.extend_from_slice(selector);

    let output = run(&store, &args);

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(held(&store, "second"), left, "{selector:?}: the named app");
    assert_eq!(
        held(&store, "first"),
        everything(),
        "{selector:?}: the default app lost a credential"
    );
}

/// Without `--app` the default app is the one cleared.
#[test]
fn a_clear_without_app_acts_on_the_default_app() {
    let tmp = TempDir::new().expect("tempdir");
    let store = two_app_store(&tmp);

    let output = run(&store, &["auth", "clear", "--force", "--all"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(held(&store, "first"), (false, vec![], false));
    assert_eq!(held(&store, "second"), everything());
}

/// A store that holds nothing has nothing to clear when no app is named: the
/// run succeeds and does not create the store.
#[rstest::rstest]
#[case::all("--all")]
#[case::oauth1("--oauth1")]
#[case::bearer("--bearer")]
fn a_clear_of_an_empty_store_without_app_changes_nothing(#[case] selector: &str) {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");

    let output = run(&store, &["auth", "clear", "--force", selector]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!store.exists(), "the clear created {}", store.display());
}

/// The dry run and the `confirmation-required` refusal of a clear name the
/// app they act on: the one `--app` selects, else the default app.
#[rstest::rstest]
#[case::named(&["--app", "second"], "second")]
#[case::default(&[], "first")]
fn a_clear_names_the_app_it_acts_on(#[case] app: &[&str], #[case] acting: &str) {
    let tmp = TempDir::new().expect("tempdir");
    let store = two_app_store(&tmp);
    let json = |output: &std::process::Output, stream: &[u8]| -> serde_json::Value {
        assert!(
            serde_json::from_slice::<serde_json::Value>(stream).is_ok(),
            "no document: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(stream).expect("a document")
    };

    let mut args = vec!["--output", "json"];
    args.extend_from_slice(app);
    args.extend(["auth", "clear", "--all"]);

    let mut dry = args.clone();
    dry.insert(0, "--dry-run");
    let planned = run(&store, &dry);
    let planned = json(&planned, &planned.stdout);
    assert_eq!(planned["app"], acting, "dry run: {planned}");

    let mut asking = args.clone();
    asking.insert(0, "--no-interactive");
    let refused = run(&store, &asking);
    let refused = json(&refused, &refused.stderr);
    assert_eq!(refused["reason"], "confirmation-required", "{refused}");
    assert_eq!(refused["app"], acting, "refusal: {refused}");
}

/// The `confirm` template a refusal hands out clears the app the refusal
/// names, whether the app came from `--app`, from `XURL_APP`, or from the
/// store's default.
#[rstest::rstest]
#[case::named(&["--app", "second"], None, "second", "first")]
#[case::environment(&[], Some("second"), "second", "first")]
#[case::default(&[], None, "first", "second")]
fn the_confirm_template_clears_the_app_the_refusal_names(
    #[case] app: &[&str],
    #[case] environment: Option<&str>,
    #[case] cleared: &str,
    #[case] untouched: &str,
) {
    let tmp = TempDir::new().expect("tempdir");
    let store = two_app_store(&tmp);
    let mut refusing = common::xr_with_store(&store);
    if let Some(app) = environment {
        refusing.env("XURL_APP", app);
    }
    let output = refusing
        .args(["--output", "json", "--no-interactive"])
        .args(app)
        .args(["auth", "clear", "--all"])
        .output()
        .expect("xr runs");
    let refused: serde_json::Value = serde_json::from_slice(&output.stderr).expect("a document");
    let template = refused["next_step"]["template"]
        .as_str()
        .unwrap_or_else(|| panic!("no template: {refused}"));
    let words: Vec<&str> = template.split_whitespace().collect();
    assert_eq!(
        words.iter().filter(|word| **word == "--app").count(),
        1,
        "{template}"
    );
    assert_eq!(held(&store, cleared), everything(), "refusing changed it");

    let confirmed = run(&store, &words[1..]);

    assert_eq!(
        confirmed.status.code(),
        Some(0),
        "{template}: {}",
        String::from_utf8_lossy(&confirmed.stderr)
    );
    assert_eq!(held(&store, cleared), (false, vec![], false), "{template}");
    assert_eq!(held(&store, untouched), everything(), "{template}");
}

/// A login is looked up in the app `--app` selects. `second` holds `bob` and
/// the default app does not, so the same selector clears it in one and is
/// refused in the other.
#[test]
fn a_login_is_looked_up_in_the_app_app_selects() {
    let tmp = TempDir::new().expect("tempdir");
    let store = two_app_store(&tmp);
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.save_oauth2_token_for_app("second", "bob", "user-at", "user-rt", 9_999_999_999)
        .expect("save_oauth2");

    let refused = run(
        &store,
        &[
            "--app",
            "first",
            "auth",
            "clear",
            "--force",
            "--oauth2-username",
            "bob",
        ],
    );

    assert_ne!(
        refused.status.code(),
        Some(0),
        "the default app holds no bob"
    );
    assert_eq!(
        held(&store, "second").1,
        ["alice", "bob"],
        "refusal changed it"
    );

    let cleared = run(
        &store,
        &[
            "--app",
            "second",
            "auth",
            "clear",
            "--force",
            "--oauth2-username",
            "bob",
        ],
    );

    assert_eq!(
        cleared.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&cleared.stderr)
    );
    assert_eq!(held(&store, "second").1, ["alice"]);
    assert_eq!(held(&store, "first"), everything());
}

/// A store that holds no app has none for a clear to name: the dry run says
/// so with `null`, not with an empty name.
#[test]
fn a_clear_of_an_empty_store_names_no_app() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");

    let output = run(
        &store,
        &["--output", "json", "--dry-run", "auth", "clear", "--bearer"],
    );

    let planned: serde_json::Value = serde_json::from_slice(&output.stdout).expect("a document");
    assert_eq!(planned["app"], serde_json::Value::Null, "{planned}");
}
