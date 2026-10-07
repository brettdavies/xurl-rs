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
//!
//! A second sidecar, `<store>.refresh.lock`, serializes `OAuth2` refreshes:
//! see [`RefreshLock`].

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
            Error::io(format!(
                "cannot open the store lock {}: {e}",
                path.display()
            ))
            .with_source(e)
        })?;
        file.lock().map_err(|e| {
            Error::io(format!("cannot lock {}: {e}", path.display())).with_source(e)
        })?;
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

/// The refresh lock path beside `store_path`: the full file name plus
/// `.refresh.lock`.
pub(crate) fn refresh_lock_path_for(store_path: &Path) -> PathBuf {
    let mut os = store_path.as_os_str().to_os_string();
    os.push(".refresh.lock");
    PathBuf::from(os)
}

/// An exclusive lock that serializes the `OAuth2` refreshes of one store
/// across processes, released on drop.
///
/// A refresh token is single-use, so two processes holding the same expired
/// login must not both spend it: the second waits here, then reads the pair
/// the first saved. The lock is a second sidecar rather than [`StoreLock`]
/// because it is held across the token request. `StoreLock` records what each
/// thread holds, and an `.await` can resume on another thread, which would
/// leave that record naming the wrong one. This lock keeps no record; the save
/// a refresh ends with takes the store lock on its own.
///
/// Like every lock here it is as strong as the filesystem's: on a network
/// mount that ignores `flock`, two hosts can still refresh at once.
#[derive(Debug)]
pub(crate) struct RefreshLock {
    file: File,
}

impl RefreshLock {
    /// Blocks until the refresh lock for `store_path` is held.
    ///
    /// The sidecar is created on first use with mode `0600`, beside the file
    /// the store path resolves to; its parent directory is never created.
    ///
    /// # Errors
    ///
    /// Returns an error naming the sidecar when it cannot be opened or locked.
    pub(crate) fn acquire(store_path: &Path) -> Result<Self> {
        let path = refresh_lock_path_for(&super::atomic::resolve(store_path));
        let mut opts = OpenOptions::new();
        opts.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let file = opts.open(&path).map_err(|e| {
            Error::io(format!(
                "cannot open the refresh lock {}: {e}",
                path.display()
            ))
            .with_source(e)
        })?;
        file.lock().map_err(|e| {
            Error::io(format!("cannot lock {}: {e}", path.display())).with_source(e)
        })?;
        Ok(Self { file })
    }

    /// [`Self::acquire`] on tokio's blocking pool, so a lock another process
    /// holds for the length of its token request never parks a runtime
    /// thread.
    ///
    /// # Errors
    ///
    /// Everything [`Self::acquire`] returns, plus an internal error when the
    /// blocking task cannot be joined.
    pub(crate) async fn acquire_off_runtime(store_path: PathBuf) -> Result<Self> {
        tokio::task::spawn_blocking(move || Self::acquire(&store_path))
            .await
            .map_err(|e| Error::Internal(format!("refresh lock task failed: {e}")))?
    }
}

impl Drop for RefreshLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_lock_path_sits_beside_the_store() {
        assert_eq!(
            refresh_lock_path_for(Path::new("/h/.xurl")),
            PathBuf::from("/h/.xurl.refresh.lock")
        );
    }

    #[test]
    fn a_second_refresh_lock_waits_for_the_first_to_drop() {
        let dir = tempfile::TempDir::new().unwrap();
        let store = dir.path().join(".xurl");
        let first = RefreshLock::acquire(&store).unwrap();
        let (acquired, on_acquire) = std::sync::mpsc::channel();
        let waiter = std::thread::spawn({
            let store = store.clone();
            move || {
                let _second = RefreshLock::acquire(&store).unwrap();
                acquired.send(()).unwrap();
            }
        });

        assert!(
            on_acquire
                .recv_timeout(std::time::Duration::from_millis(200))
                .is_err(),
            "the second acquire completed while the first was held"
        );
        drop(first);
        on_acquire
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("the second acquire completes once the first drops");
        waiter.join().unwrap();
    }

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
