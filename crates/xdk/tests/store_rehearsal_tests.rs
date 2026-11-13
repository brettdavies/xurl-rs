//! `TokenStore::update_at` runs its write on a blocking-pool thread, which
//! carries no rehearsal mark, so it refuses ahead of that hop.

use tempfile::TempDir;
use xdk::Error;
use xdk::store::TokenStore;

#[test]
fn an_update_by_path_during_a_rehearsal_writes_nothing() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("auth.yml");
    let mut store = TokenStore::new_with_path(&path.to_string_lossy());
    store.add_app("work", "id", "secret").expect("add_app");
    let before = std::fs::read(&path).expect("the store file");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("runtime");

    let outcome = store.rehearse(|_| {
        runtime.block_on(TokenStore::update_at(path.clone(), |store| {
            store.add_app("extra", "id", "secret")
        }))
    });

    assert!(
        matches!(&outcome, Err(Error::Internal(message)) if message.contains("rehearsal")),
        "{outcome:?}"
    );
    assert_eq!(std::fs::read(&path).expect("the store file"), before);
}
