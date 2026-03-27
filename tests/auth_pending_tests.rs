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

// ── Adversarial / Red Team Tests ────────────────────────────────────────

#[test]
fn load_corrupt_yaml_returns_error() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".xurl.pending");

    // Write garbage that isn't valid YAML
    std::fs::write(&path, "{{{{ not valid yaml !@#$").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }

    let err = pending::load(&path).unwrap_err();
    let msg = err.to_string();
    // Should be a deserialization error, not a panic
    assert!(
        msg.contains("Auth Error"),
        "Expected auth/parse error, got: {msg}"
    );
}

#[test]
fn load_valid_yaml_missing_fields_returns_error() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".xurl.pending");

    // Valid YAML but missing required fields
    std::fs::write(&path, "code_verifier: abc\nstate: xyz\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }

    let err = pending::load(&path).unwrap_err();
    // Should fail deserialization, not panic
    assert!(err.to_string().contains("Auth Error"));
}

#[test]
fn save_creates_correct_temp_file_name() {
    // Regression test: with_extension("tmp") was turning .xurl.pending into .xurl.tmp
    // instead of .xurl.pending.tmp
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".xurl.pending");

    let state = sample_state();
    pending::save(&state, &path).unwrap();

    // The temp file should have been cleaned up (renamed to final)
    let wrong_tmp = tmp.path().join(".xurl.tmp");
    let correct_tmp = tmp.path().join(".xurl.pending.tmp");
    assert!(
        !wrong_tmp.exists(),
        "Wrong temp path .xurl.tmp should not exist"
    );
    assert!(
        !correct_tmp.exists(),
        "Temp file should be renamed away after save"
    );
    assert!(path.exists(), "Final pending file should exist");
}

#[test]
fn save_twice_overwrites_atomically() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".xurl.pending");

    let state1 = sample_state();
    pending::save(&state1, &path).unwrap();

    let mut state2 = sample_state();
    state2.code_verifier = "different_verifier".into();
    state2.state = "different_state".into();
    pending::save(&state2, &path).unwrap();

    let loaded = pending::load(&path).unwrap();
    assert_eq!(loaded.code_verifier, "different_verifier");
    assert_eq!(loaded.state, "different_state");
}
