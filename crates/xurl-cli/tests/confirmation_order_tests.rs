//! A destructive command refuses an input or a credential the real run would
//! refuse before it asks the person to confirm, so the `--force` invocation it
//! hands out is one that then runs. A dry run of a refused invocation says it
//! would refuse, and does not say it would ask.

mod common;

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use wiremock::MockServer;

use common::{oauth1_store, run_in_process as run};

const WEBHOOK_ID: &str = "1146654567674912769";

/// A store whose default app holds client credentials and no token.
fn no_credential_store(dir: &Path) -> PathBuf {
    let store = dir.join(".xurl");
    let mut ts = xdk::store::TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.add_app("myapp", "CLIENT-ID-VALUE", "SECRET-VALUE")
        .expect("add_app");
    ts.set_default_app("myapp").expect("set_default_app");
    let _ = ts.remove_app("default");
    store
}

/// A store whose default app holds a bearer token alone.
fn bearer_only_store(dir: &Path) -> PathBuf {
    let store = no_credential_store(dir);
    let mut ts = xdk::store::TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.save_bearer_token_for_app("myapp", "BEARER-TOKEN")
        .expect("save bearer");
    store
}

#[derive(Clone, Copy)]
enum Held {
    OAuth1,
    BearerOnly,
    NoCredential,
}

fn seed(held: Held, dir: &Path) -> PathBuf {
    match held {
        Held::OAuth1 => oauth1_store(dir),
        Held::BearerOnly => bearer_only_store(dir),
        Held::NoCredential => no_credential_store(dir),
    }
}

fn json(text: &str) -> serde_json::Value {
    serde_json::from_str(text.trim()).unwrap_or_else(|e| panic!("not JSON ({e}): {text}"))
}

/// `xr --no-interactive --output json [--dry-run] <args>` against a mock
/// server that must receive nothing: the exit code and the document the run
/// printed, which is on stdout for a dry run and on stderr for a real one.
async fn invoke(held: Held, args: &[&str], dry_run: bool) -> (i32, serde_json::Value) {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = seed(held, tmp.path());
    let mut argv = vec!["--no-interactive", "--output", "json"];
    if dry_run {
        argv.push("--dry-run");
    }
    argv.extend_from_slice(args);
    let (code, stdout, stderr) = run(&store, &server.uri(), &argv).await;
    assert_eq!(
        server.received_requests().await.expect("recorded").len(),
        0,
        "{args:?}: a request went out"
    );
    (code, json(if dry_run { &stdout } else { &stderr }))
}

/// An invocation the real run refuses is refused before it asks: with the
/// reason its input or credential earns, and not `confirmation-required`.
/// Its dry run says the same and does not say it would ask.
#[rstest::rstest]
#[case::empty_post_id(Held::OAuth1, &["delete", ""], "validation", 1)]
#[case::no_credential(Held::NoCredential, &["delete", "12345"], "auth-required", 77)]
#[case::scheme_the_endpoint_refuses(Held::BearerOnly, &["delete", "12345", "--auth", "app"], "auth-method-mismatch", 2)]
#[case::invalid_webhook_id(Held::OAuth1, &["webhooks", "remove", "not-an-id"], "validation", 1)]
#[case::invalid_webhook_id_for_a_subscription(Held::BearerOnly, &["webhooks", "subscriptions", "remove", "not-an-id", "@jack"], "validation", 1)]
#[case::invalid_account_for_a_subscription(Held::BearerOnly, &["webhooks", "subscriptions", "remove", WEBHOOK_ID, "not a handle"], "validation", 1)]
#[case::no_credential_for_a_subscription(Held::NoCredential, &["webhooks", "subscriptions", "remove", WEBHOOK_ID, "@jack"], "auth-required", 77)]
#[tokio::test]
async fn a_refused_invocation_is_refused_before_it_asks(
    #[case] held: Held,
    #[case] args: &[&str],
    #[case] reason: &str,
    #[case] exit_code: i32,
) {
    let (code, real) = invoke(held, args, false).await;
    assert_eq!(
        (code, real["reason"].as_str()),
        (exit_code, Some(reason)),
        "{args:?}: {real}"
    );

    let (dry_code, dry) = invoke(held, args, true).await;
    assert_eq!(dry_code, 0, "{args:?}: {dry}");
    assert_eq!(dry["would_succeed"], false, "{args:?}: {dry}");
    assert_eq!(dry["reason"], real["reason"], "{args:?}: {dry}");
    assert!(
        dry.get("confirmation_required").is_none(),
        "{args:?}: the dry run says it would ask where the real run refuses: {dry}"
    );
}

/// An invocation that would run, absent the confirmation, still asks: the
/// real run stops with `confirmation-required`, and its dry run says so.
#[rstest::rstest]
#[case::post(Held::OAuth1, &["delete", "12345"])]
#[case::webhook(Held::OAuth1, &["webhooks", "remove", WEBHOOK_ID])]
#[case::subscription(Held::BearerOnly, &["webhooks", "subscriptions", "remove", WEBHOOK_ID, "@jack"])]
#[tokio::test]
async fn a_runnable_invocation_still_asks(#[case] held: Held, #[case] args: &[&str]) {
    let (code, real) = invoke(held, args, false).await;
    assert_eq!(
        (code, real["reason"].as_str()),
        (1, Some("confirmation-required")),
        "{args:?}: {real}"
    );

    let (dry_code, dry) = invoke(held, args, true).await;
    assert_eq!(dry_code, 0, "{args:?}: {dry}");
    assert_eq!(dry["would_succeed"], true, "{args:?}: {dry}");
    assert_eq!(dry["confirmation_required"], true, "{args:?}: {dry}");
}
