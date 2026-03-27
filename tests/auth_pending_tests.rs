//! Tests for the pending OAuth2 state persistence module.
//!
//! Validates save/load round-trip, TTL expiry, file permissions,
//! and the delete/exists helpers.

use std::time::{SystemTime, UNIX_EPOCH};

use tempfile::TempDir;

use xurl::auth::pending::{self, PendingOAuth2State};

// ── Helpers ──────────────────────────────────────────────────────────────

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn sample_state() -> PendingOAuth2State {
    PendingOAuth2State {
        code_verifier: "test_verifier_abc123".into(),
        state: "random_state_xyz".into(),
        client_id: "client_id_001".into(),
        redirect_uri: "http://localhost:8739/callback".into(),
        app_name: "myapp".into(),
        created_at: now_secs(),
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[test]
fn save_and_load_round_trip() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".xurl.pending");

    let original = sample_state();
    pending::save(&original, &path).unwrap();

    let loaded = pending::load(&path).unwrap();
    assert_eq!(original, loaded);
}

#[test]
fn load_nonexistent_returns_error() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("does_not_exist");

    let err = pending::load(&path).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("PendingStateNotFound"),
        "expected PendingStateNotFound, got: {msg}"
    );
}

#[test]
fn delete_removes_file() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".xurl.pending");

    let state = sample_state();
    pending::save(&state, &path).unwrap();
    assert!(path.exists());

    pending::delete(&path).unwrap();
    assert!(!path.exists());
}

#[test]
fn delete_nonexistent_is_ok() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("nope");
    // Should not error.
    pending::delete(&path).unwrap();
}

#[cfg(unix)]
#[test]
fn file_permissions_are_0600() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".xurl.pending");

    let state = sample_state();
    pending::save(&state, &path).unwrap();

    let meta = std::fs::metadata(&path).unwrap();
    let mode = meta.permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "expected 0600, got {mode:04o}");
}

#[test]
fn load_rejects_expired_state() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".xurl.pending");

    let mut state = sample_state();
    // Set created_at to 20 minutes ago — well past the 15-minute TTL.
    state.created_at = now_secs().saturating_sub(1200);

    pending::save(&state, &path).unwrap();
    assert!(path.exists());

    let err = pending::load(&path).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("PendingStateExpired"),
        "expected PendingStateExpired, got: {msg}"
    );

    // The expired file should have been deleted.
    assert!(!path.exists());
}

#[test]
fn exists_returns_false_when_no_file() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".xurl.pending");
    assert!(!pending::exists(&path));
}

#[test]
fn exists_returns_true_after_save() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".xurl.pending");

    let state = sample_state();
    pending::save(&state, &path).unwrap();
    assert!(pending::exists(&path));
}
