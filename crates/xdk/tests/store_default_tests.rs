//! `TokenStore::set_default_app_and_user`: the default app and its default
//! user change together or not at all.

use tempfile::TempDir;
use xdk::store::TokenStore;

/// A store with two apps. `first` is the default and holds users `bob` (its
/// default) and `alice`; `second` holds `alice`.
fn two_app_store(tmp: &TempDir) -> TokenStore {
    let path = tmp.path().join(".xurl");
    let mut store = TokenStore::new_with_path(path.to_str().expect("utf-8 path"));
    for (app, users) in [("first", &["bob", "alice"][..]), ("second", &["alice"][..])] {
        store
            .add_app(app, "CLIENT-ID-VALUE", "SECRET-VALUE")
            .expect("add_app");
        for user in users {
            store
                .save_oauth2_token_for_app(app, user, "user-at", "user-rt", 9_999_999_999)
                .expect("save_oauth2");
        }
    }
    store.set_default_app("first").expect("set_default_app");
    store
        .set_default_user("first", "bob")
        .expect("set_default_user");
    let _ = store.remove_app("default");
    store
}

/// What a fresh load of the file reads: the default app, and the default
/// user of `app`.
fn on_disk(tmp: &TempDir, app: &str) -> (String, String) {
    let path = tmp.path().join(".xurl");
    let store = TokenStore::new_with_path(path.to_str().expect("utf-8 path"));
    (
        store.get_default_app().to_string(),
        store.get_default_user(app).to_string(),
    )
}

#[test]
fn the_app_and_its_user_are_saved_together() {
    let tmp = TempDir::new().expect("tempdir");
    let mut store = two_app_store(&tmp);

    store
        .set_default_app_and_user("second", "alice")
        .expect("both names are registered");

    assert_eq!(store.get_default_app(), "second");
    assert_eq!(store.get_default_user("second"), "alice");
    assert_eq!(
        on_disk(&tmp, "second"),
        ("second".to_string(), "alice".to_string())
    );
}

/// A user the app does not hold is refused before the default app moves.
#[test]
fn an_unknown_user_leaves_the_default_app_in_place() {
    let tmp = TempDir::new().expect("tempdir");
    let mut store = two_app_store(&tmp);

    let err = store
        .set_default_app_and_user("second", "nobody")
        .expect_err("second holds no user named nobody");

    assert!(err.to_string().contains("nobody"), "{err}");
    assert_eq!(
        on_disk(&tmp, "first"),
        ("first".to_string(), "bob".to_string())
    );
}

/// An unregistered app is refused by name. `alice` is a user of the default
/// app, and is not made its default in the missing app's place.
#[test]
fn an_unknown_app_leaves_every_default_user_in_place() {
    let tmp = TempDir::new().expect("tempdir");
    let mut store = two_app_store(&tmp);

    let err = store
        .set_default_app_and_user("nope", "alice")
        .expect_err("no app named nope");

    assert!(err.to_string().contains("nope"), "{err}");
    assert_eq!(
        on_disk(&tmp, "first"),
        ("first".to_string(), "bob".to_string())
    );
}
