//! Persistence layer for in-flight OAuth2 PKCE state.
//!
//! During the remote OAuth2 flow the authorization URL is opened on one device
//! while the callback is received on another. [`PendingOAuth2State`] captures
//! the PKCE code verifier, state nonce, and associated metadata so the callback
//! handler can resume the exchange even if the originating process has exited.
//!
//! The pending file lives at `~/.xurl/auth.yml.pending` by default and is created with
//! `0o600` permissions on Unix. A 15-minute TTL guards against stale state.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{
    Error, PENDING_EXPIRED, PENDING_INVALID, PENDING_NOT_FOUND, PENDING_PERMISSIONS, Result,
};

/// Maximum age of a pending state file before it is considered expired.
const PENDING_TTL_SECS: u64 = 900; // 15 minutes

/// Serialisable snapshot of an in-flight OAuth2 PKCE authorization.
///
/// Persisted to disk between `xr auth oauth2 --step 1` (which writes it)
/// and `xr auth oauth2 --step 2` (which reads + deletes it on success).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingOAuth2State {
    /// PKCE code verifier; presented to the token endpoint in step 2.
    pub code_verifier: String,
    /// CSRF state nonce; the step-2 redirect URL must echo this value.
    pub state: String,
    /// `OAuth2` client ID active when step 1 ran. Step 2 rejects the
    /// exchange if the runtime client ID has changed.
    pub client_id: String,
    /// Active app name when step 1 ran; surfaced in error messages when
    /// step 2 mismatches the runtime context.
    pub app_name: String,
    /// Unix epoch seconds when the authorization was initiated. Used by
    /// [`load`] to enforce the 15-minute freshness window.
    pub created_at: u64,
    /// The scopes the authorize URL requested. Empty in a file written
    /// before the field existed, which requested every scope.
    #[serde(default)]
    pub scopes: Vec<String>,
}

impl PendingOAuth2State {
    /// The scopes step 1 requested, reading an absent list as every scope.
    #[must_use]
    pub fn requested_scopes(&self) -> Vec<&str> {
        if self.scopes.is_empty() {
            super::oauth2::get_oauth2_scopes()
        } else {
            self.scopes.iter().map(String::as_str).collect()
        }
    }
}

/// Returns the pending-state path that sits beside a token store: `<store>.pending`.
///
/// Deriving it from the store path keeps every file the auth flow writes under
/// the one location the caller chose, so an injected store path isolates the
/// pending state too.
#[must_use]
pub fn pending_path_for_store(store_path: &Path) -> PathBuf {
    let mut os = store_path.as_os_str().to_os_string();
    os.push(".pending");
    PathBuf::from(os)
}

/// Returns the default path for the pending-state file
/// (`~/.xurl/auth.yml.pending`).
///
/// # Errors
///
/// Returns an error if the home directory cannot be determined (e.g.,
/// containers or CI environments without `HOME` set).
pub fn default_pending_path() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|_| {
            pending_path_for_store(&crate::store::locate_store(
                &crate::store::default_store_path(),
            ))
        })
        .ok_or_else(|| {
            Error::auth(
                "could not determine home directory for pending state file. \
                 Set the HOME environment variable",
            )
        })
}

/// Persists `state` to `path` atomically with restricted permissions.
///
/// # Errors
///
/// Returns an error if serialisation or filesystem operations fail.
pub fn save(state: &PendingOAuth2State, path: &Path) -> Result<()> {
    // The loader rejects a symlinked pending file; the writer must not follow
    // one either, or a planted link turns the save into a write of the PKCE
    // verifier to a path the user did not choose.
    if fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink()) {
        return Err(Error::auth(format!(
            "{PENDING_PERMISSIONS}: pending state path {} is a symlink (refusing to write through it)",
            path.display()
        )));
    }
    let data =
        crate::yaml::to_string(state).map_err(|e| Error::auth(e.to_string()).with_source(e))?;
    crate::store::ensure_store_dir(path)?;
    crate::store::write_atomically(path, data.as_bytes())?;
    Ok(())
}

/// Loads and validates a `PendingOAuth2State` from `path`.
///
/// # Validation
///
/// 1. The file must exist and not be a symlink.
/// 2. On Unix the file must be owned by the current user with mode `0o600`.
/// 3. The `created_at` timestamp must be within 15 minutes of now.
///
/// If the file is expired it is deleted before the error is returned.
///
/// # Errors
///
/// Returns an error if the file is missing, is a symlink, has incorrect
/// permissions/owner, is expired, or cannot be deserialised.
pub fn load(path: &Path) -> Result<PendingOAuth2State> {
    let state = read(path)?;
    ensure_fresh(&state).inspect_err(|_| {
        let _ = fs::remove_file(path);
    })?;
    Ok(state)
}

/// [`load`] that leaves an expired file where it is.
///
/// # Errors
///
/// As [`load`].
// `xr --dry-run auth oauth2` and the recovery hint it shares with the real
// run read the flow through this, since a dry run must leave an expired flow
// on disk. It is not embedder API.
#[doc(hidden)]
pub fn peek(path: &Path) -> Result<PendingOAuth2State> {
    let state = read(path)?;
    ensure_fresh(&state)?;
    Ok(state)
}

/// Reads the file at `path` and parses it, whatever its age.
fn read(path: &Path) -> Result<PendingOAuth2State> {
    // Reject symlinks to prevent an attacker from pointing to a crafted file.
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        use std::os::unix::fs::PermissionsExt;

        let meta = match fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(Error::auth(PENDING_NOT_FOUND));
            }
            Err(e) => return Err(e.into()),
        };

        if meta.file_type().is_symlink() {
            return Err(Error::auth(format!(
                "{PENDING_PERMISSIONS}: pending state file {} is a symlink (rejected for security)",
                path.display()
            )));
        }

        let mode = meta.permissions().mode() & 0o777;
        if mode != 0o600 {
            return Err(Error::auth(format!(
                "{PENDING_PERMISSIONS}: {}: expected mode 0600, got {mode:04o}",
                path.display()
            )));
        }

        let file_uid = meta.uid();
        let current_uid = rustix::process::getuid().as_raw();
        if file_uid != current_uid {
            return Err(Error::auth(format!(
                "{PENDING_PERMISSIONS}: {}: file owned by uid {file_uid}, expected {current_uid}",
                path.display()
            )));
        }
    }

    // On non-Unix, do a simple existence check.
    #[cfg(not(unix))]
    if !path.exists() {
        return Err(Error::auth(PENDING_NOT_FOUND));
    }

    let data = match fs::read_to_string(path) {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(Error::auth(
                "PendingStateNotFound: no pending OAuth2 state file found",
            ));
        }
        Err(e) => return Err(e.into()),
    };
    serde_saphyr::from_str(&data).map_err(|e| {
        Error::auth(format!(
            "{PENDING_INVALID}: {}",
            crate::yaml::failure_site(path, &e)
        ))
        .with_source(e)
    })
}

/// Refuses a flow older than the 15-minute window.
fn ensure_fresh(state: &PendingOAuth2State) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if now.saturating_sub(state.created_at) > PENDING_TTL_SECS {
        return Err(Error::auth(PENDING_EXPIRED));
    }
    Ok(())
}

/// Deletes the pending-state file at `path`.
///
/// Silently succeeds if the file does not exist.
///
/// # Errors
///
/// Returns an error for filesystem failures other than `NotFound`.
pub fn delete(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}
