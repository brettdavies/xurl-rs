//! Atomic replacement of a file's contents.
//!
//! The bytes land in a uniquely named sibling temp file that is created with
//! mode `0600` at open time, flushed and synced, then renamed over the
//! destination, and the directory is synced so the rename itself survives a
//! crash. A reader of the destination path therefore sees either the
//! previous contents or the new ones, never a truncated file, and the tokens
//! are never world-readable for any interval. A symlinked destination is
//! written through to its target, as a plain write would be, and a temp file
//! that never reached the rename is removed on the way out.
//!
//! The rename is what makes one replacement atomic; the sidecar lock in
//! [`super::lock`] is what serializes two writers. That lock is advisory
//! (`flock`), so a filesystem that ignores it, such as an NFS or SMB mount
//! without lock support, gives no exclusion, and deleting the sidecar while
//! a holder has it open lets the next process lock a fresh inode. Both guard
//! against xurl's own concurrent processes, not a hostile neighbour.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// How old a sibling temp file must be before a writer treats it as the
/// leftover of a crashed process and removes it; a live writer finishes in
/// milliseconds.
const STALE_TEMP_AGE: Duration = Duration::from_secs(60);

/// The path a write to `path` lands on: the symlink target when `path` is a
/// link, `path` itself otherwise.
///
/// A plain `fs::write` follows a symlink; a rename would replace the link
/// with a regular file and leave the real store stale, so the writer and
/// the lock both resolve the link first. A path that does not exist yet
/// resolves to itself.
pub(crate) fn resolve(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// The temp path beside `target`: the full file name plus `.tmp.<pid>.<nanos>`.
///
/// Unique per writer, so two processes writing one store never share a temp
/// file, and appended to the whole name so a dotfile such as `.xurl.pending`
/// keeps its name rather than having its "extension" replaced.
fn temp_path_for(target: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut os = target.as_os_str().to_os_string();
    os.push(format!(".tmp.{}.{nanos}", std::process::id()));
    PathBuf::from(os)
}

/// Removes temp files a crashed writer left beside `target`, so a copy of
/// the credentials never outlives the process that wrote it by more than
/// the stale age. A recent temp file may belong to a live writer and stays.
fn sweep_stale_temps(target: &Path) {
    let (Some(parent), Some(name)) = (target.parent(), target.file_name()) else {
        return;
    };
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    let mut prefix = name.to_os_string();
    prefix.push(".tmp.");
    let prefix = prefix.to_string_lossy().into_owned();
    let Ok(entries) = fs::read_dir(parent) else {
        return;
    };
    let now = SystemTime::now();
    for entry in entries.flatten() {
        if !entry.file_name().to_string_lossy().starts_with(&prefix) {
            continue;
        }
        let stale = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|modified| now.duration_since(modified).ok())
            .is_some_and(|age| age >= STALE_TEMP_AGE);
        if stale {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// A temp file that unlinks itself unless the rename published it, so no
/// error path leaves a copy of the credentials beside the store.
struct TempFile {
    path: PathBuf,
    published: bool,
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if !self.published {
            let _ = fs::remove_file(&self.path);
        }
    }
}

/// Makes a completed rename in `target`'s directory durable.
///
/// Filesystems that cannot sync a directory handle report it as
/// unsupported or invalid; the data itself was already synced, so that is
/// not a failure.
#[cfg(unix)]
fn sync_parent(target: &Path) -> io::Result<()> {
    let parent = match target.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    match File::open(parent)?.sync_all() {
        Err(e)
            if matches!(
                e.kind(),
                io::ErrorKind::Unsupported | io::ErrorKind::InvalidInput
            ) =>
        {
            Ok(())
        }
        other => other,
    }
}

#[cfg(not(unix))]
fn sync_parent(_target: &Path) -> io::Result<()> {
    Ok(())
}

/// Replaces the contents of `path` with `bytes` atomically.
///
/// # Errors
///
/// Returns the first filesystem error: creating, writing, syncing, or
/// renaming the temp file, or syncing the directory afterwards.
pub(crate) fn write_atomically(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let target = resolve(path);
    sweep_stale_temps(&target);
    let mut tmp = TempFile {
        path: temp_path_for(&target),
        published: false,
    };

    let mut opts = OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }

    {
        let mut file = opts.open(&tmp.path)?;
        file.write_all(bytes)?;
        file.flush()?;
        file.sync_all()?;
    }

    fs::rename(&tmp.path, &target)?;
    tmp.published = true;
    sync_parent(&target)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_files_in(dir: &Path) -> Vec<PathBuf> {
        fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.to_string_lossy().contains(".tmp."))
            .collect()
    }

    #[test]
    fn temp_path_appends_to_the_full_name() {
        let tmp = temp_path_for(Path::new("/h/.xurl.pending"));
        assert!(
            tmp.to_string_lossy().starts_with("/h/.xurl.pending.tmp."),
            "{}",
            tmp.display()
        );
    }

    #[test]
    fn two_temp_paths_for_one_target_differ() {
        let a = temp_path_for(Path::new("/h/.xurl"));
        let b = temp_path_for(Path::new("/h/.xurl"));
        assert_ne!(a, b);
    }

    #[test]
    fn a_stale_temp_file_is_swept_and_a_recent_one_is_left_alone() {
        let dir = tempfile::TempDir::new().unwrap();
        let target = dir.path().join("store");
        let stale = dir.path().join("store.tmp.1.1");
        let recent = dir.path().join("store.tmp.2.2");
        fs::write(&stale, b"old secret").unwrap();
        fs::write(&recent, b"live writer").unwrap();
        File::open(&stale)
            .unwrap()
            .set_modified(SystemTime::now() - Duration::from_secs(3600))
            .unwrap();

        write_atomically(&target, b"fresh").unwrap();

        assert!(!stale.exists(), "a crashed writer's temp file is removed");
        assert!(recent.exists(), "a temp file a live writer may own stays");
        assert_eq!(fs::read(&target).unwrap(), b"fresh");
    }

    #[test]
    fn a_failed_rename_leaves_no_temp_file_behind() {
        let dir = tempfile::TempDir::new().unwrap();
        let target = dir.path().join("occupied");
        fs::create_dir(&target).unwrap();

        let err = write_atomically(&target, b"secret").unwrap_err();
        assert!(!err.to_string().is_empty());
        assert!(
            temp_files_in(dir.path()).is_empty(),
            "the temp file must not survive a failed rename"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_symlinked_target_is_written_through() {
        let dir = tempfile::TempDir::new().unwrap();
        let real = dir.path().join("real-store");
        fs::write(&real, b"old").unwrap();
        let link = dir.path().join("link-store");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        write_atomically(&link, b"new").unwrap();

        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink(),
            "the link survives the write"
        );
        assert_eq!(fs::read(&real).unwrap(), b"new");
        assert!(temp_files_in(dir.path()).is_empty());
    }
}
