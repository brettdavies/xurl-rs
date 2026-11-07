//! Tests for the pending OAuth2 state persistence module.
//!
//! Validates save/load round-trip, TTL expiry, file permissions,
//! and the delete helper.

use std::time::{SystemTime, UNIX_EPOCH};

use tempfile::TempDir;

use xdk::auth::Auth;
use xdk::auth::pending::{self, PendingOAuth2State};
use xdk::config::{Config, EnvOverrides};

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
        app_name: "myapp".into(),
        created_at: now_secs(),
        scopes: vec!["tweet.read".into(), "offline.access".into()],
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

/// A file with no `scopes` key loads, and reads as the full set every
/// sign-in requested when that file was written.
#[test]
fn a_state_without_scopes_loads_as_every_scope() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".xurl.pending");
    let mut written = sample_state();
    written.scopes.clear();
    pending::save(&written, &path).unwrap();
    let yaml = std::fs::read_to_string(&path).unwrap();
    let without_key: String = yaml
        .lines()
        .filter(|line| !line.starts_with("scopes"))
        .map(|line| format!("{line}\n"))
        .collect();
    assert_ne!(yaml, without_key, "the key was there to remove");
    std::fs::write(&path, without_key).unwrap();

    let loaded = pending::load(&path).unwrap();

    assert!(loaded.scopes.is_empty());
    assert_eq!(
        loaded.requested_scopes(),
        xdk::auth::oauth2::get_oauth2_scopes()
    );
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
fn peek_refuses_an_expired_state_and_leaves_the_file_where_it_is() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".xurl.pending");
    let mut state = sample_state();
    state.created_at = now_secs().saturating_sub(1200);
    pending::save(&state, &path).unwrap();
    let kept = std::fs::read(&path).unwrap();

    let msg = pending::peek(&path).unwrap_err().to_string();

    assert!(msg.contains("PendingStateExpired"), "got: {msg}");
    assert_eq!(std::fs::read(&path).unwrap(), kept);
}

#[test]
fn peek_reads_a_fresh_state_as_load_does() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".xurl.pending");
    let state = sample_state();
    pending::save(&state, &path).unwrap();

    assert_eq!(pending::peek(&path).unwrap(), state);
    assert_eq!(pending::load(&path).unwrap(), state);
    assert!(path.exists());
}

fn auth_for_the_sample_client(tmp: &TempDir) -> Auth {
    let mut cfg = Config::from_overrides(&EnvOverrides::default());
    cfg.client_id = "client_id_001".into();
    cfg.client_secret = "client_secret_001".into();
    Auth::new_with_store_path_and_overrides(
        &cfg,
        &tmp.path().join(".xurl"),
        &EnvOverrides::default(),
    )
}

#[test]
fn a_check_with_no_redirect_url_passes_over_a_flow_that_fits() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".xurl.pending");
    let mut state = sample_state();
    state.scopes.clear();
    pending::save(&state, &path).unwrap();

    let auth = auth_for_the_sample_client(&tmp);

    auth.check_remote_oauth2_step2(None, &path).unwrap();
    assert!(path.exists());
}

#[test]
fn a_check_with_no_redirect_url_still_wants_a_saved_flow() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".xurl.pending");
    let auth = auth_for_the_sample_client(&tmp);

    let msg = auth
        .check_remote_oauth2_step2(None, &path)
        .unwrap_err()
        .to_string();

    assert!(msg.contains("PendingStateNotFound"), "got: {msg}");
}

// ── Adversarial / Red Team Tests ────────────────────────────────────────

#[cfg(unix)]
#[test]
fn load_rejects_wrong_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".xurl.pending");

    let state = sample_state();
    pending::save(&state, &path).unwrap();

    // Widen permissions to 0644 — load() should reject this.
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    let err = pending::load(&path).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("PendingStatePermissions"),
        "expected PendingStatePermissions, got: {msg}"
    );
}

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
    // Should be a deserialization error, not a panic
    assert_eq!(err.kind(), "auth-required", "got: {err:?}");
    assert!(
        std::error::Error::source(&err)
            .and_then(|source| source.downcast_ref::<serde_saphyr::Error>())
            .is_some(),
        "the YAML error is the auth error's source: {err:?}"
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
    assert!(matches!(err, xdk::Error::Auth { .. }), "got: {err:?}");
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

#[cfg(unix)]
#[test]
fn save_refuses_a_symlinked_pending_path_and_leaves_the_target_untouched() {
    let tmp = TempDir::new().unwrap();
    let victim = tmp.path().join("victim.txt");
    std::fs::write(&victim, "precious").unwrap();
    let path = tmp.path().join(".xurl.pending");
    std::os::unix::fs::symlink(&victim, &path).unwrap();

    let err =
        pending::save(&sample_state(), &path).expect_err("a symlinked pending path is refused");
    assert!(err.to_string().contains("symlink"), "{err}");
    assert_eq!(std::fs::read_to_string(&victim).unwrap(), "precious");
    assert!(
        std::fs::symlink_metadata(&path)
            .unwrap()
            .file_type()
            .is_symlink()
    );
}
