//! Cross-process exclusion for token-store writes.
//!
//! The lock lives on a sidecar file beside the store (`<store>.lock`), not
//! on the store itself: the store is replaced by rename on every save, so a
//! lock on its inode would stop protecting anything after the first write,
//! and the file may not exist yet at all. `std::fs::File::lock` is an OS
//! lock (`flock` on Unix, `LockFileEx` on Windows), which is what serializes
//! two processes; an in-process mutex would not.
//!
//! Within one thread the lock is reentrant: a mutator called from inside a
//! locked update finds the sidecar already held and proceeds without a
//! second OS lock, which would otherwise block against the first handle
//! forever.

use std::cell::RefCell;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

thread_local! {
    /// Sidecars this thread holds, by canonical path.
    static HELD: RefCell<Vec<PathBuf>> = const { RefCell::new(Vec::new()) };
}

/// The lock path beside `store_path`: the full file name plus `.lock`.
pub(crate) fn lock_path_for(store_path: &Path) -> PathBuf {
    let mut os = store_path.as_os_str().to_os_string();
    os.push(".lock");
    PathBuf::from(os)
}

/// An exclusive lock on a store's sidecar, released on drop.
#[derive(Debug)]
pub(crate) enum StoreLock {
    /// This guard took the OS lock and releases it.
    Owner { file: File, key: PathBuf },
    /// An outer guard on this thread already holds the OS lock.
    Reentrant,
}

impl StoreLock {
    /// Blocks until the sidecar lock for `store_path` is held by this thread.
    ///
    /// The sidecar sits beside the file the store path resolves to, so a
    /// store reached through a symlink shares its lock with the store
    /// reached directly. It is created on first use with mode `0600`; its
    /// parent directory is never created, so a store path whose directory
    /// does not exist fails here exactly as its save would.
    ///
    /// # Errors
    ///
    /// Returns an error naming the sidecar when it cannot be opened or locked.
    pub(crate) fn acquire(store_path: &Path) -> Result<Self> {
        let path = lock_path_for(&super::atomic::resolve(store_path));
        let key = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        if HELD.with(|held| held.borrow().contains(&key)) {
            return Ok(Self::Reentrant);
        }

        let mut opts = OpenOptions::new();
        opts.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let file = opts.open(&path).map_err(|e| {
            Error::Io(format!(
                "cannot open the store lock {}: {e}",
                path.display()
            ))
        })?;
        file.lock()
            .map_err(|e| Error::Io(format!("cannot lock {}: {e}", path.display())))?;
        let key = fs::canonicalize(&path).unwrap_or(key);
        HELD.with(|held| held.borrow_mut().push(key.clone()));
        Ok(Self::Owner { file, key })
    }

    /// Whether an outer guard on this thread already held the lock.
    pub(crate) fn is_reentrant(&self) -> bool {
        matches!(self, Self::Reentrant)
    }
}

impl Drop for StoreLock {
    fn drop(&mut self) {
        if let Self::Owner { file, key } = self {
            let _ = file.unlock();
            HELD.with(|held| held.borrow_mut().retain(|held_key| held_key != key));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_path_sits_beside_the_store() {
        assert_eq!(
            lock_path_for(Path::new("/h/.xurl")),
            PathBuf::from("/h/.xurl.lock")
        );
    }

    #[test]
    fn a_missing_parent_directory_is_an_error() {
        let dir = tempfile::TempDir::new().unwrap();
        let store = dir.path().join("absent").join(".xurl");
        assert!(StoreLock::acquire(&store).is_err());
    }

    #[test]
    fn a_second_acquire_on_the_same_thread_is_reentrant_until_the_owner_drops() {
        let dir = tempfile::TempDir::new().unwrap();
        let store = dir.path().join(".xurl");
        let owner = StoreLock::acquire(&store).unwrap();
        assert!(!owner.is_reentrant());
        assert!(StoreLock::acquire(&store).unwrap().is_reentrant());
        drop(owner);
        assert!(!StoreLock::acquire(&store).unwrap().is_reentrant());
    }
}
