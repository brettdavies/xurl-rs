//! Atomic replacement of a file's contents.
//!
//! The bytes land in a sibling temp file that is created with mode `0600`
//! at open time, flushed and synced, then renamed over the destination. A
//! reader of the destination path therefore sees either the previous
//! contents or the new ones, never a truncated file, and the tokens are
//! never world-readable for any interval.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// The temp path beside `path`: the full file name plus `.tmp`, so a
/// dotfile such as `.xurl.pending` becomes `.xurl.pending.tmp` rather than
/// having its "extension" replaced.
fn temp_path_for(path: &Path) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(".tmp");
    PathBuf::from(os)
}

/// Replaces the contents of `path` with `bytes` atomically.
///
/// A temp file left behind by an interrupted earlier write is removed
/// first, so `create_new` can succeed.
///
/// # Errors
///
/// Returns the first filesystem error: creating, writing, syncing, or
/// renaming the temp file.
pub(crate) fn write_atomically(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = temp_path_for(path);
    match fs::remove_file(&tmp) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }

    let mut opts = OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }

    {
        let mut file = opts.open(&tmp)?;
        file.write_all(bytes)?;
        file.flush()?;
        file.sync_all()?;
    }

    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_path_appends_to_the_full_name() {
        assert_eq!(
            temp_path_for(Path::new("/h/.xurl.pending")),
            PathBuf::from("/h/.xurl.pending.tmp")
        );
    }

    #[test]
    fn stale_temp_file_is_replaced() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("f");
        let tmp = temp_path_for(&path);
        fs::write(&tmp, b"stale").unwrap();

        write_atomically(&path, b"fresh").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"fresh");
        assert!(!tmp.exists());
    }
}
