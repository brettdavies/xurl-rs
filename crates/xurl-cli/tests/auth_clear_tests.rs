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

/// Without `--app` the default app is the one cleared, as it always was.
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
