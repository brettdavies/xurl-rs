//! `TokenStore::resolve_app_mut` returns the app a name holds, the default app
//! for the empty name, and a refusal for any other name, with the store left as
//! it was.

use tempfile::TempDir;
use xdk::error::Error;
use xdk::store::TokenStore;

const APP: &str = "main-app";

fn store_holding_app(tmp: &TempDir) -> TokenStore {
    let path = tmp.path().join("auth.yml");
    let mut store = TokenStore::new_with_path(&path.to_string_lossy());
    store
        .add_app(APP, "CLIENT-ID", "CLIENT-SECRET")
        .expect("add app");
    store
}

fn empty_store(tmp: &TempDir) -> TokenStore {
    TokenStore::new_with_path(&tmp.path().join("auth.yml").to_string_lossy())
}

#[test]
fn a_name_the_store_does_not_hold_creates_no_app() {
    let tmp = TempDir::new().expect("tempdir");
    let mut store = empty_store(&tmp);

    let _ = store.resolve_app_mut("typo");

    assert!(
        store.list_apps().is_empty(),
        "the store gained {:?}",
        store.list_apps()
    );
}

#[test]
fn a_name_the_store_does_not_hold_is_refused_naming_the_apps_it_holds() {
    let tmp = TempDir::new().expect("tempdir");
    let mut store = store_holding_app(&tmp);

    for name in ["typo", "Main-App", "main-app ", "default"] {
        let Err(Error::AppNotFound {
            name: refused,
            held,
        }) = store.resolve_app_mut(name)
        else {
            panic!("{name:?} was not refused");
        };
        assert_eq!(refused, name);
        assert_eq!(held, [APP]);
    }
    assert_eq!(store.list_apps(), [APP]);
}

#[test]
fn the_empty_name_and_a_held_name_resolve_as_they_did() {
    let tmp = TempDir::new().expect("tempdir");
    let mut store = store_holding_app(&tmp);

    assert_eq!(
        store
            .resolve_app_mut("")
            .expect("the default app")
            .client_id,
        "CLIENT-ID"
    );
    assert_eq!(
        store.resolve_app_mut(APP).expect("a held app").client_id,
        "CLIENT-ID"
    );

    let other = TempDir::new().expect("tempdir");
    let mut empty = empty_store(&other);
    empty
        .resolve_app_mut("")
        .expect("the empty name is the default app, created when none exists");
    assert_eq!(empty.list_apps(), ["default"]);
}
