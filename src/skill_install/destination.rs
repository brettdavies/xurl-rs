//! `$HOME` expansion and destination inspection. Owns the pre-clone status
//! taxonomy and the path functions that resolve and vet an install directory.

use std::fs;
use std::path::{Path, PathBuf};

use super::InstallError;

/// Snapshot of what [`check_destination`] found at the resolved path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DestinationStatus {
    /// Nothing exists at the resolved path.
    Absent,
    /// An empty directory exists; safe to clone into.
    EmptyDir,
    /// A directory with at least one entry blocks the clone.
    NonEmptyDir,
    /// A regular file (or device / socket / fifo) blocks the clone.
    File,
}

impl DestinationStatus {
    /// Kebab-case identifier for the JSON envelope `destination_status` field.
    pub fn as_envelope_str(self) -> &'static str {
        match self {
            DestinationStatus::Absent => "absent",
            DestinationStatus::EmptyDir => "empty-dir",
            DestinationStatus::NonEmptyDir => "non-empty-dir",
            DestinationStatus::File => "file",
        }
    }
}

/// Expand a leading `~` or `~/` to `$HOME`. Pure passthrough on inputs that
/// do not start with `~`. `MissingHome` only fires when the input actually
/// begins with `~` and `$HOME` is unset or empty.
pub fn expand_tilde(template: &str) -> Result<PathBuf, InstallError> {
    let home = std::env::var("HOME").ok();
    expand_tilde_with(template, home.as_deref())
}

/// Pure-function core of [`expand_tilde`]. Tests pass `home` explicitly so
/// they never mutate the process environment (which would race with parallel
/// tests).
pub fn expand_tilde_with(template: &str, home: Option<&str>) -> Result<PathBuf, InstallError> {
    let needs_home = template == "~" || template.starts_with("~/");
    if !needs_home {
        return Ok(PathBuf::from(template));
    }
    let home = home
        .filter(|s| !s.is_empty())
        .ok_or(InstallError::MissingHome)?;
    if template == "~" {
        return Ok(PathBuf::from(home));
    }
    let rest = template
        .strip_prefix("~/")
        .expect("template starts with ~/ per the branch guard");
    let mut p = PathBuf::from(home);
    p.push(rest);
    Ok(p)
}

/// Canonicalizes the path so a symlinked skills directory resolves to its
/// real target before the check runs. Returns `Absent`/`EmptyDir` on success;
/// `DestIsFile` for a regular file, `DestNotEmpty` for a populated directory.
pub fn check_destination(path: &Path) -> Result<DestinationStatus, InstallError> {
    match path.try_exists() {
        Ok(false) => return Ok(DestinationStatus::Absent),
        Ok(true) => {}
        Err(_) => return Err(InstallError::DestNotEmpty),
    }

    let canonical = fs::canonicalize(path).map_err(|_| InstallError::DestNotEmpty)?;
    let metadata = fs::metadata(&canonical).map_err(|_| InstallError::DestNotEmpty)?;

    if metadata.is_file() {
        return Err(InstallError::DestIsFile);
    }

    if metadata.is_dir() {
        let mut entries = fs::read_dir(&canonical).map_err(|_| InstallError::DestNotEmpty)?;
        if entries.next().is_some() {
            return Err(InstallError::DestNotEmpty);
        }
        return Ok(DestinationStatus::EmptyDir);
    }

    // Block / char devices, sockets, fifos — treat as a file conflict.
    Err(InstallError::DestIsFile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_tilde_replaces_leading_tilde_slash_with_home() {
        let got = expand_tilde_with("~/.claude/skills/xurl-rs", Some("/home/test"))
            .expect("HOME present + ~/ prefix should expand cleanly");
        assert_eq!(got, PathBuf::from("/home/test/.claude/skills/xurl-rs"));
    }

    #[test]
    fn expand_tilde_missing_home_only_when_input_starts_with_tilde() {
        let err = expand_tilde_with("~/anything", None)
            .expect_err("HOME unset + tilde input should be MissingHome");
        assert_eq!(err, InstallError::MissingHome);

        let err_empty =
            expand_tilde_with("~", Some("")).expect_err("HOME empty string is treated as unset");
        assert_eq!(err_empty, InstallError::MissingHome);
    }

    #[test]
    fn expand_tilde_no_tilde_passthrough() {
        let got = expand_tilde_with("/abs/path", Some("/home/test"))
            .expect("non-tilde input never errors");
        assert_eq!(got, PathBuf::from("/abs/path"));

        let got_no_home =
            expand_tilde_with("/abs/path", None).expect("non-tilde input ignores HOME");
        assert_eq!(got_no_home, PathBuf::from("/abs/path"));
    }

    #[test]
    fn check_destination_absent_for_nonexistent_path() {
        let tmp = tempfile::tempdir().expect("tempdir creation");
        let target = tmp.path().join("does-not-exist");
        let status = check_destination(&target).expect("absent path should be Ok(Absent)");
        assert_eq!(status, DestinationStatus::Absent);
    }

    #[test]
    fn check_destination_empty_dir() {
        let tmp = tempfile::tempdir().expect("tempdir creation");
        let status = check_destination(tmp.path()).expect("empty tempdir should be Ok(EmptyDir)");
        assert_eq!(status, DestinationStatus::EmptyDir);
    }

    #[test]
    fn check_destination_non_empty_dir_errors() {
        let tmp = tempfile::tempdir().expect("tempdir creation");
        std::fs::write(tmp.path().join("placeholder"), b"x").expect("write placeholder");
        let err = check_destination(tmp.path()).expect_err("populated dir should be DestNotEmpty");
        assert_eq!(err, InstallError::DestNotEmpty);
    }

    #[test]
    fn check_destination_regular_file_errors() {
        let tmp = tempfile::tempdir().expect("tempdir creation");
        let target = tmp.path().join("a-file");
        std::fs::write(&target, b"contents").expect("write file");
        let err = check_destination(&target).expect_err("file should be DestIsFile");
        assert_eq!(err, InstallError::DestIsFile);
    }
}
