/// Persistence layer for in-flight OAuth2 PKCE state.
///
/// During the remote OAuth2 flow the authorization URL is opened on one device
/// while the callback is received on another.  `PendingOAuth2State` captures
/// the PKCE code verifier, state nonce, and associated metadata so the callback
/// handler can resume the exchange even if the originating process has exited.
///
/// The pending file lives at `~/.xurl.pending` by default and is created with
/// `0o600` permissions on Unix.  A 15-minute TTL guards against stale state.
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{Result, XurlError};

/// Maximum age of a pending state file before it is considered expired.
const PENDING_TTL_SECS: u64 = 900; // 15 minutes

/// Serialisable snapshot of an in-flight OAuth2 PKCE authorization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingOAuth2State {
    pub code_verifier: String,
    pub state: String,
    pub client_id: String,
    pub app_name: String,
    /// Unix epoch seconds when the authorization was initiated.
    pub created_at: u64,
}

/// Returns the default path for the pending-state file (`~/.xurl.pending`).
///
/// # Errors
///
/// Returns an error if the home directory cannot be determined (e.g.,
/// containers or CI environments without `HOME` set).
pub fn default_pending_path() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|h| h.join(".xurl.pending"))
        .ok_or_else(|| {
            XurlError::auth(
                "could not determine home directory for pending state file. \
                 Set the HOME environment variable",
            )
        })
}

/// Persists `state` to `path` atomically with restricted permissions.
///
/// Writes to a temporary file (`{path}.tmp`) first, then renames it into
/// place so that readers never see a partially-written file.
///
/// # Errors
///
/// Returns an error if serialisation or filesystem operations fail.
pub fn save(state: &PendingOAuth2State, path: &Path) -> Result<()> {
    let data = serde_yaml::to_string(state).map_err(|e| XurlError::Auth(e.to_string()))?;

    // Append ".tmp" rather than replacing the extension — with_extension("tmp")
    // would turn ".xurl.pending" into ".xurl.tmp" instead of ".xurl.pending.tmp".
    let mut tmp_os = path.as_os_str().to_os_string();
    tmp_os.push(".tmp");
    let tmp_path = std::path::PathBuf::from(tmp_os);

    // If a stale temp file exists from a previous interrupted save, remove it
    // so `create_new(true)` can succeed.
    if tmp_path.exists() {
        let _ = fs::remove_file(&tmp_path);
    }

    {
        let mut opts = OpenOptions::new();
        opts.write(true).create_new(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }

        let mut file = opts.open(&tmp_path)?;
        file.write_all(data.as_bytes())?;
        file.flush()?;
        file.sync_all()?;
    }

    fs::rename(&tmp_path, path)?;
    Ok(())
}

/// Loads and validates a `PendingOAuth2State` from `path`.
///
/// # Validation
///
/// 1. The file must exist and not be a symlink.
/// 2. On Unix the file must be owned by the current user with mode `0o600`.
/// 3. The `created_at` timestamp must be within [`PENDING_TTL_SECS`] of now.
///
/// If the file is expired it is deleted before the error is returned.
///
/// # Errors
///
/// Returns an error if the file is missing, is a symlink, has incorrect
/// permissions/owner, is expired, or cannot be deserialised.
pub fn load(path: &Path) -> Result<PendingOAuth2State> {
    // Reject symlinks to prevent an attacker from pointing to a crafted file.
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        use std::os::unix::fs::PermissionsExt;

        let meta = match fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(XurlError::auth(
                    "PendingStateNotFound: no pending OAuth2 state file found",
                ));
            }
            Err(e) => return Err(e.into()),
        };

        if meta.file_type().is_symlink() {
            return Err(XurlError::auth(
                "PendingStatePermissions: pending state file is a symlink (rejected for security)",
            ));
        }

        let mode = meta.permissions().mode() & 0o777;
        if mode != 0o600 {
            return Err(XurlError::auth(format!(
                "PendingStatePermissions: expected mode 0600, got {mode:04o}"
            )));
        }

        let file_uid = meta.uid();
        let current_uid = unsafe { libc::getuid() };
        if file_uid != current_uid {
            return Err(XurlError::auth(format!(
                "PendingStatePermissions: file owned by uid {file_uid}, expected {current_uid}"
            )));
        }
    }

    // On non-Unix, do a simple existence check.
    #[cfg(not(unix))]
    if !path.exists() {
        return Err(XurlError::auth(
            "PendingStateNotFound: no pending OAuth2 state file found",
        ));
    }

    let data = match fs::read_to_string(path) {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(XurlError::auth(
                "PendingStateNotFound: no pending OAuth2 state file found",
            ));
        }
        Err(e) => return Err(e.into()),
    };
    let state: PendingOAuth2State =
        serde_yaml::from_str(&data).map_err(|e| XurlError::Auth(e.to_string()))?;

    // TTL check.
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if now.saturating_sub(state.created_at) > PENDING_TTL_SECS {
        let _ = fs::remove_file(path);
        return Err(XurlError::auth(
            "PendingStateExpired: pending OAuth2 state is older than 15 minutes",
        ));
    }

    Ok(state)
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
