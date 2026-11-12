//! Every path that writes the store refuses during a rehearsal and leaves the
//! disk as it found it.

use super::*;
use tempfile::TempDir;

fn seeded(path: &std::path::Path) -> TokenStore {
    let mut store = TokenStore::new_with_path(&path.to_string_lossy());
    store.add_app("work", "id", "secret").unwrap();
    store
}

fn refused_as_a_defect<T: std::fmt::Debug>(outcome: &Result<T>) -> bool {
    matches!(outcome, Err(Error::Internal(message)) if message.contains("rehearsal"))
}

#[test]
fn a_save_during_a_rehearsal_writes_nothing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("auth.yml");
    let store = seeded(&path);
    let before = std::fs::read(&path).unwrap();

    let outcome = store.rehearse(|copy| {
        copy.apps.clear();
        copy.save_to_file()
    });

    assert!(refused_as_a_defect(&outcome), "{outcome:?}");
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn a_locked_save_during_a_rehearsal_creates_no_directory_and_no_lock() {
    let dir = TempDir::new().unwrap();
    let home = dir.path().join("fresh");
    let path = home.join("auth.yml");
    let store = TokenStore::new_with_path(&path.to_string_lossy());

    let outcome = store.rehearse(|copy| copy.save_locked());

    assert!(refused_as_a_defect(&outcome), "{outcome:?}");
    assert!(!home.exists(), "the rehearsal created {}", home.display());
}

#[test]
fn a_write_after_a_rehearsal_is_not_refused() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("auth.yml");
    let mut store = seeded(&path);

    store
        .rehearse(|copy| copy.add_app("extra", "id", "secret"))
        .unwrap();
    store.add_app("extra", "id", "secret").unwrap();

    assert!(
        TokenStore::new_with_path(&path.to_string_lossy())
            .get_app("extra")
            .is_some()
    );
}
