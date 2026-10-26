//! An account's names, carried to every credential the store holds for it.
//!
//! A credential's `user_id` is the account's for good, and each credential
//! learned its own from an answer given under it, so every credential that
//! carries an id belongs to that account. The username and display name can
//! change, so the newest answer for an account names it in every app, while
//! the id itself is never copied from one credential to another.

use super::TokenStore;

/// Whether a name the answer carried differs from the stored one. A name
/// the answer left out leaves the stored one as it is.
fn differs(stored: &Option<String>, answered: Option<&str>) -> bool {
    answered.is_some_and(|answered| stored.as_deref() != Some(answered))
}

impl TokenStore {
    /// Writes the username and display name an answer gave for the account
    /// `user_id` onto every `OAuth2` login and `OAuth1` access pair that
    /// carries that id, in every app, filling names a credential lacks and
    /// replacing stale ones. Returns whether any changed.
    pub(super) fn share_account_names(
        &mut self,
        user_id: &str,
        account_username: Option<&str>,
        display_name: Option<&str>,
    ) -> bool {
        let mut changed = false;
        let mut adopt =
            |id: &Option<String>, username: &mut Option<String>, name: &mut Option<String>| {
                if id.as_deref() != Some(user_id) {
                    return;
                }
                for (stored, answered) in [(username, account_username), (name, display_name)] {
                    if differs(stored, answered) {
                        *stored = answered.map(str::to_string);
                        changed = true;
                    }
                }
            };
        for app in self.apps.values_mut() {
            let logins = app
                .oauth2_tokens
                .values_mut()
                .chain(app.unnamed_oauth2_token.as_mut())
                .filter_map(|token| token.oauth2.as_mut());
            for token in logins {
                adopt(
                    &token.user_id,
                    &mut token.account_username,
                    &mut token.display_name,
                );
            }
            if let Some(pair) = app.oauth1_token.as_mut().and_then(|t| t.oauth1.as_mut()) {
                adopt(
                    &pair.user_id,
                    &mut pair.account_username,
                    &mut pair.display_name,
                );
            }
        }
        changed
    }

    /// Whether [`Self::share_account_names`] would change anything, read
    /// without taking the lock, so an answer the store already reflects
    /// writes nothing.
    pub(super) fn has_account_names_to_share(
        &self,
        user_id: &str,
        account_username: Option<&str>,
        display_name: Option<&str>,
    ) -> bool {
        self.apps.values().any(|app| {
            let logins = app
                .oauth2_tokens
                .values()
                .chain(app.unnamed_oauth2_token.as_ref())
                .filter_map(|token| token.oauth2.as_ref())
                .map(|t| (&t.user_id, &t.account_username, &t.display_name));
            let pair = app
                .oauth1_token
                .as_ref()
                .and_then(|token| token.oauth1.as_ref())
                .map(|t| (&t.user_id, &t.account_username, &t.display_name));
            logins.chain(pair).any(|(id, username, name)| {
                id.as_deref() == Some(user_id)
                    && (differs(username, account_username) || differs(name, display_name))
            })
        })
    }
}
