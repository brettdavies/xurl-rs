//! Where the token store lives on disk.
//!
//! The store is `auth.yml` inside a directory, `~/.xurl/` by default, which
//! is where Go `xurl` 1.3.0 and later keep theirs. `xdk-rs` 0.2 and earlier,
//! and Go `xurl` before 1.3.0, kept the same YAML in a single file at the
//! directory's own path.
//!
//! Two functions read that history. [`locate_store`] finds the store and
//! moves nothing, which is what every constructor that resolves the default
//! path uses: opening a store never rearranges the caller's home directory.
//! [`adopt_directory_layout`] makes the move, and is the caller's deliberate
//! act; `xr` calls it on the first command that opens the store.
//!
//! ```text
//!                                    locate_store     adopt_directory_layout
//! <dir> missing or a directory     -> <dir>/auth.yml   <dir>/auth.yml
//! <dir> a regular file             -> <dir>            moved to <dir>/auth.yml
//! <dir> a symbolic link to a file  -> <dir>            <dir>
//! the move fails                   ->                  put back; <dir>
//! ```

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::lock::StoreLock;

/// The `tracing` target of every event the store emits.
pub const STORE_TARGET: &str = "xdk::store";

/// The store's file name inside its directory.
pub const AUTH_FILE_NAME: &str = "auth.yml";

/// The store's directory name under the home directory.
const STORE_DIR_NAME: &str = ".xurl";

/// The default token-store file: `~/.xurl/auth.yml`.
///
/// Falls back to `./.xurl/auth.yml` when the home directory cannot be
/// resolved. This names the path and touches nothing.
#[must_use]
pub fn default_store_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(STORE_DIR_NAME)
        .join(AUTH_FILE_NAME)
}

/// The path to open for `store_path`, moving nothing.
///
/// `store_path` itself, unless it is in the directory layout and a file still
/// sits where its directory belongs: that file is the same store in the
/// single-file layout, and its path is returned.
#[must_use]
pub fn locate_store(store_path: &Path) -> PathBuf {
    match directory_of(store_path) {
        Some(dir) => path_to_open(dir, store_path),
        None => store_path.to_path_buf(),
    }
}

/// Moves a single-file store into the directory layout and returns the path
/// to open.
///
/// `store_path` is in the directory layout when its file name is `auth.yml`.
/// When a regular file sits where its directory belongs, that file is the
/// same store in the single-file layout: it becomes `store_path`, its pending
/// sign-in state moves with it, and a `tracing` event on [`STORE_TARGET`]
/// (`kind = "layout-moved"`, with `from` and `to`) reports the move. A path
/// with any other file name is returned as given.
///
/// The move is two renames under the single-file store's own lock, so two
/// processes cannot interleave it, and one interrupted between the renames is
/// finished by the next call. When it cannot be made, the file is put back
/// and its path is returned, so the caller keeps working against it.
#[must_use]
pub fn adopt_directory_layout(store_path: &Path) -> PathBuf {
    let Some(dir) = directory_of(store_path) else {
        return store_path.to_path_buf();
    };
    let aside = with_suffix(dir, ".migrating");
    if !is_regular_file(dir) && !aside.exists() {
        return path_to_open(dir, store_path);
    }

    // The lock is the single-file store's own, so a release that still reads
    // that layout waits here too.
    let Ok(_lock) = StoreLock::acquire(dir) else {
        return path_to_open(dir, store_path);
    };
    finish_interrupted_move(dir, &aside, store_path);
    if !is_regular_file(dir) {
        return path_to_open(dir, store_path);
    }
    match move_into_directory(dir, &aside, store_path) {
        Ok(()) => {
            move_sidecars(dir, store_path);
            tracing::info!(
                target: STORE_TARGET,
                kind = "layout-moved",
                from = %dir.display(),
                to = %store_path.display(),
                "token store moved into its directory",
            );
            store_path.to_path_buf()
        }
        Err(e) => {
            tracing::warn!(
                target: STORE_TARGET,
                "could not move the token store {} to {}: {e}; using it where it is",
                dir.display(),
                store_path.display(),
            );
            dir.to_path_buf()
        }
    }
}

/// The directory of a path in the directory layout; `None` for any other
/// path.
fn directory_of(store_path: &Path) -> Option<&Path> {
    if store_path.file_name() != Some(OsStr::new(AUTH_FILE_NAME)) {
        return None;
    }
    store_path
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
}

/// `store_path`, unless a file, or a symbolic link to one, occupies its
/// directory's path. Once [`adopt_directory_layout`] has moved a regular
/// file, only a link is left to find there: a link cannot be turned into a
/// directory without breaking whatever placed it, so the store stays where
/// it points.
fn path_to_open(dir: &Path, store_path: &Path) -> PathBuf {
    if fs::metadata(dir).is_ok_and(|meta| meta.is_file()) {
        dir.to_path_buf()
    } else {
        store_path.to_path_buf()
    }
}

/// Whether `path` is a regular file itself, not a link to one.
fn is_regular_file(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|meta| meta.is_file())
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(suffix);
    PathBuf::from(os)
}

/// Completes a move that stopped between its two renames: the store is
/// stranded at `aside` and its directory is missing or empty.
///
/// A stranded file beside a store that is already in its directory is not
/// that store's content any more. It is kept under a `.bak` name, so it stops
/// sending every later call down the locked path.
fn finish_interrupted_move(dir: &Path, aside: &Path, store_path: &Path) {
    if !aside.exists() || is_regular_file(dir) {
        return;
    }
    if store_path.exists() {
        let _ = fs::rename(aside, with_suffix(aside, ".bak"));
        return;
    }
    if create_store_dir(dir).is_ok() && fs::rename(aside, store_path).is_ok() {
        tracing::info!(
            target: STORE_TARGET,
            kind = "layout-moved",
            from = %dir.display(),
            to = %store_path.display(),
            "interrupted token store move finished",
        );
    }
}

/// Renames the file at `dir` aside, creates the directory, and renames the
/// file into it. Each failure undoes the steps before it.
fn move_into_directory(dir: &Path, aside: &Path, store_path: &Path) -> io::Result<()> {
    if aside.exists() {
        // A stranded file from an attempt that could not be finished is kept,
        // not overwritten.
        let _ = fs::rename(aside, with_suffix(aside, ".bak"));
    }
    fs::rename(dir, aside)?;
    if let Err(e) = create_store_dir(dir) {
        let _ = fs::rename(aside, dir);
        return Err(e);
    }
    if let Err(e) = fs::rename(aside, store_path) {
        let _ = fs::remove_dir(dir);
        let _ = fs::rename(aside, dir);
        return Err(e);
    }
    Ok(())
}

/// Carries the pending sign-in state across and drops the two lock files the
/// single-file layout kept beside the store. Each is recreated on demand at
/// its new path, so a failure here costs nothing.
fn move_sidecars(dir: &Path, store_path: &Path) {
    let pending = with_suffix(dir, ".pending");
    if pending.exists() {
        let _ = fs::rename(pending, with_suffix(store_path, ".pending"));
    }
    for lock in [".lock", ".refresh.lock"] {
        let _ = fs::remove_file(with_suffix(dir, lock));
    }
}

/// Creates the directory a store file or one of its sidecars lives in, when
/// `path` is in the directory layout and the directory is missing. The store
/// owns that directory; a path in any other shape names a directory the
/// caller chose, and a missing one stays an error.
pub(crate) fn ensure_store_dir(path: &Path) -> io::Result<()> {
    let in_layout = path
        .file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| {
            name.strip_prefix(AUTH_FILE_NAME)
                .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
        });
    match path.parent() {
        Some(dir) if in_layout && !dir.as_os_str().is_empty() && !dir.exists() => {
            create_store_dir(dir)
        }
        _ => Ok(()),
    }
}

/// Creates the store's directory, readable by its owner alone.
fn create_store_dir(dir: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const STORE: &[u8] =
        b"apps:\n  work:\n    client_id: abc\n    client_secret: shh\ndefault_app: work\n";

    /// `<tmp>/.xurl` and the directory-layout path inside it.
    fn paths(tmp: &TempDir) -> (PathBuf, PathBuf) {
        let dir = tmp.path().join(".xurl");
        let store = dir.join(AUTH_FILE_NAME);
        (dir, store)
    }

    #[test]
    fn a_single_file_store_moves_into_its_directory() {
        let tmp = TempDir::new().unwrap();
        let (dir, store) = paths(&tmp);
        fs::write(&dir, STORE).unwrap();

        assert_eq!(adopt_directory_layout(&store), store);
        assert!(dir.is_dir());
        assert_eq!(fs::read(&store).unwrap(), STORE);
        assert!(!with_suffix(&dir, ".migrating").exists());
    }

    #[cfg(unix)]
    #[test]
    fn the_moved_store_keeps_its_mode_in_a_directory_only_its_owner_reads() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = TempDir::new().unwrap();
        let (dir, store) = paths(&tmp);
        fs::write(&dir, STORE).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o600)).unwrap();

        let _ = adopt_directory_layout(&store);
        let mode = |path: &Path| fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&dir), 0o700);
        assert_eq!(mode(&store), 0o600);
    }

    #[test]
    fn locating_a_single_file_store_returns_it_and_moves_nothing() {
        let tmp = TempDir::new().unwrap();
        let (dir, store) = paths(&tmp);
        fs::write(&dir, STORE).unwrap();
        fs::write(with_suffix(&dir, ".pending"), b"pending").unwrap();

        assert_eq!(locate_store(&store), dir);
        assert!(dir.is_file());
        assert_eq!(fs::read(&dir).unwrap(), STORE);
        assert!(with_suffix(&dir, ".pending").is_file());
    }

    #[test]
    fn locating_a_store_in_its_directory_or_not_yet_written_returns_its_path() {
        let tmp = TempDir::new().unwrap();
        let (dir, store) = paths(&tmp);
        assert_eq!(locate_store(&store), store);
        assert!(!dir.exists());

        fs::create_dir(&dir).unwrap();
        fs::write(&store, STORE).unwrap();
        assert_eq!(locate_store(&store), store);

        let explicit = tmp.path().join("my-store.yaml");
        assert_eq!(locate_store(&explicit), explicit);
    }

    #[test]
    fn a_path_with_nothing_there_is_returned_and_nothing_is_created() {
        let tmp = TempDir::new().unwrap();
        let (dir, store) = paths(&tmp);

        assert_eq!(adopt_directory_layout(&store), store);
        assert!(!dir.exists());
    }

    #[test]
    fn a_store_already_in_its_directory_is_left_alone() {
        let tmp = TempDir::new().unwrap();
        let (dir, store) = paths(&tmp);
        fs::create_dir(&dir).unwrap();
        fs::write(&store, STORE).unwrap();

        assert_eq!(adopt_directory_layout(&store), store);
        assert_eq!(fs::read(&store).unwrap(), STORE);
    }

    #[test]
    fn a_path_outside_the_directory_layout_is_returned_as_given() {
        let tmp = TempDir::new().unwrap();
        let explicit = tmp.path().join("my-store.yaml");
        fs::write(&explicit, STORE).unwrap();

        assert_eq!(adopt_directory_layout(&explicit), explicit);
        assert!(explicit.is_file());
    }

    #[test]
    fn a_move_interrupted_between_its_renames_is_finished() {
        let tmp = TempDir::new().unwrap();
        let (dir, store) = paths(&tmp);
        fs::write(with_suffix(&dir, ".migrating"), STORE).unwrap();

        assert_eq!(adopt_directory_layout(&store), store);
        assert_eq!(fs::read(&store).unwrap(), STORE);
        assert!(!with_suffix(&dir, ".migrating").exists());
    }

    #[test]
    fn a_stranded_file_beside_a_store_in_its_directory_is_kept_aside() {
        let tmp = TempDir::new().unwrap();
        let (dir, store) = paths(&tmp);
        fs::create_dir(&dir).unwrap();
        fs::write(&store, STORE).unwrap();
        let stranded = with_suffix(&dir, ".migrating");
        fs::write(&stranded, b"older").unwrap();

        assert_eq!(adopt_directory_layout(&store), store);
        assert_eq!(fs::read(&store).unwrap(), STORE);
        assert!(!stranded.exists());
        assert_eq!(fs::read(with_suffix(&stranded, ".bak")).unwrap(), b"older");
    }

    #[test]
    fn pending_sign_in_state_moves_and_the_old_lock_files_go() {
        let tmp = TempDir::new().unwrap();
        let (dir, store) = paths(&tmp);
        fs::write(&dir, STORE).unwrap();
        fs::write(with_suffix(&dir, ".pending"), b"pending").unwrap();
        fs::write(with_suffix(&dir, ".refresh.lock"), b"").unwrap();

        let _ = adopt_directory_layout(&store);
        assert_eq!(
            fs::read(with_suffix(&store, ".pending")).unwrap(),
            b"pending"
        );
        for gone in [".pending", ".lock", ".refresh.lock"] {
            assert!(!with_suffix(&dir, gone).exists(), "{gone} is left behind");
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_symbolic_link_to_a_store_is_used_where_it_points() {
        let tmp = TempDir::new().unwrap();
        let (dir, store) = paths(&tmp);
        let target = tmp.path().join("elsewhere.yaml");
        fs::write(&target, STORE).unwrap();
        std::os::unix::fs::symlink(&target, &dir).unwrap();

        assert_eq!(adopt_directory_layout(&store), dir);
        assert!(fs::symlink_metadata(&dir).unwrap().file_type().is_symlink());
        assert_eq!(fs::read(&target).unwrap(), STORE);
    }

    #[cfg(unix)]
    #[test]
    fn a_move_that_cannot_be_made_leaves_the_file_and_returns_its_path() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        fs::create_dir(&home).unwrap();
        let dir = home.join(".xurl");
        let store = dir.join(AUTH_FILE_NAME);
        fs::write(&dir, STORE).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o500)).unwrap();
        // A process that may write a read-only directory (root) cannot make
        // the move fail this way.
        let writable = fs::write(home.join("probe"), b"").is_ok();

        let opened = adopt_directory_layout(&store);
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
        if writable {
            return;
        }
        assert_eq!(opened, dir);
        assert_eq!(fs::read(&dir).unwrap(), STORE);
    }
}
