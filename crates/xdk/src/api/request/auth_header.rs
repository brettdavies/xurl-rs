//! Auth-scheme selection for a request, and the mismatch envelope that
//! follows an empty intersection between the credentials at hand and the
//! endpoint's accepted schemes.

use tracing::instrument::WithSubscriber;

use crate::api::auth_matrix::WireScheme;
use crate::error::{AuthMismatch, Error, Result};

use super::render_template_template;
use super::source::{CredentialSource, SchemeFacts};
use super::transport::attaches_credential;
use super::{Client, RequestOptions, RequestTarget};

/// What scheme selection decided for one request: a header computed from
/// the credentials at hand, or an OAuth2 token that may still need a refresh.
enum Selection {
    Header { scheme: WireScheme, header: String },
    OAuth2 { username: String },
}

/// The credential a request would be sent with.
///
/// [`Client::auth_preflight`] and [`Call::auth_preflight`] read it from the
/// credentials at hand. Neither sends anything.
///
/// [`Call::auth_preflight`]: super::Call::auth_preflight
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AuthPreflight {
    /// The scheme the request would go out under.
    pub scheme: WireScheme,
    /// The app whose stored credential would be sent. `None` for a bearer
    /// token the environment supplied, which belongs to no stored app, and
    /// for a client built from credentials.
    pub app: Option<String>,
    /// The name the `OAuth2` token is stored under. `None` under another
    /// scheme, for a token stored without a name, and for a client built
    /// from credentials.
    pub username: Option<String>,
    /// Whether the `OAuth2` access token has passed its expiry, so that
    /// sending would refresh it first. `false` under another scheme.
    pub token_expired: bool,
    /// The id of the account the user credential belongs to, when the
    /// store holds it beside that credential: `/2/users/me` answered under
    /// it at sign-in, at a refresh, or since. `None` for an app-only bearer,
    /// which is no user's.
    pub user_id: Option<String>,
}

impl Client {
    /// Gets the authorization header for a request (public accessor for command layer).
    ///
    /// # Errors
    ///
    /// Returns an error if no valid auth method is found, or — when the
    /// auto-detect path resolves an empty intersection between the
    /// credentials at hand and the endpoint's accepted schemes — an
    /// [`Error::AuthMethodMismatch`] in the empty-intersection shape.
    pub async fn get_auth_header_public(&self, options: &RequestOptions) -> Result<String> {
        self.get_auth_header(options).await
    }

    /// Reports the credential `options` would be sent with, or `None` for a
    /// request the client attaches none to: one marked `no_auth`, or one
    /// carrying an `Authorization` header of the caller's own.
    ///
    /// Scheme selection runs exactly as it does for a send, and stops
    /// there: nothing is sent, and an expired `OAuth2` token is reported in
    /// [`AuthPreflight::token_expired`] instead of being refreshed. Whether X
    /// still honors the credential is a question only a request answers.
    ///
    /// # Errors
    ///
    /// The error a send would stop on before its first request:
    /// [`Error::AuthMethodMismatch`] when the endpoint accepts no scheme the
    /// credentials can serve, and [`Error::Auth`] when no credential serves
    /// the scheme selected.
    pub async fn auth_preflight(&self, options: &RequestOptions) -> Result<Option<AuthPreflight>> {
        if !attaches_credential(options) {
            return Ok(None);
        }
        let credentials = self.credentials().await;
        let preflight = match self.select_scheme(&credentials, options)? {
            Selection::Header { scheme, .. } => AuthPreflight {
                scheme,
                app: credentials.credential_app(scheme),
                username: None,
                token_expired: false,
                user_id: match scheme {
                    WireScheme::OAuth1 => credentials.oauth1_user_id(),
                    WireScheme::OAuth2 | WireScheme::App => None,
                },
            },
            Selection::OAuth2 { username } => {
                let state = credentials
                    .oauth2_state(&username)
                    .ok_or_else(|| Error::auth(crate::error::NO_OAUTH2_TOKEN))?;
                AuthPreflight {
                    scheme: WireScheme::OAuth2,
                    app: credentials.credential_app(WireScheme::OAuth2),
                    username: state.username,
                    token_expired: state.expired,
                    user_id: state.user_id,
                }
            }
        };
        Ok(Some(preflight))
    }

    /// Gets the authorization header for a request.
    ///
    /// Scheme selection runs under the credential lock and never awaits; an
    /// OAuth2 token that needs refreshing is refreshed afterwards through
    /// [`Self::oauth2_bearer`], which takes the lock again for the rotation.
    pub(super) async fn get_auth_header(&self, options: &RequestOptions) -> Result<String> {
        let selected = {
            let credentials = self.credentials().await;
            self.select_scheme(&credentials, options)?
        };
        match selected {
            Selection::Header { header, .. } => Ok(header),
            Selection::OAuth2 { username } => self.oauth2_bearer(&username).await,
        }
    }

    /// The OAuth2 `Bearer` header for `username`, refreshing the token when
    /// it has expired.
    ///
    /// The refresh runs in its own task and is awaited through its handle,
    /// so a caller dropping the request cannot drop the refresh between X
    /// rotating the refresh token and the new pair being installed and
    /// persisted. A second request that finds the lock held waits, then
    /// finds the fresh token and returns without refreshing again.
    async fn oauth2_bearer(&self, username: &str) -> Result<String> {
        {
            let credentials = self.credentials().await;
            if !credentials.has_oauth2_token(username) {
                return Err(Error::auth(crate::error::NO_OAUTH2_TOKEN));
            }
            if let Some(access_token) = credentials.unexpired_oauth2_access_token(username) {
                return Ok(format!("Bearer {access_token}"));
            }
        }
        let client = self.clone();
        let username = username.to_string();
        // The task is polled outside the request future, so without the
        // caller's dispatcher it would report under the global no-op one and
        // the binary would never render the refresh warnings.
        let access_token = tokio::spawn(
            async move {
                let mut credentials = client.credentials().await;
                credentials
                    .refresh_oauth2_token(client.http(), &username)
                    .await
            }
            .with_current_subscriber(),
        )
        .await
        .map_err(|e| Error::Internal(format!("token refresh task failed: {e}")))??;
        Ok(format!("Bearer {access_token}"))
    }

    /// Picks the scheme for a request.
    ///
    /// When `options.auth_type` is non-empty, dispatches directly to that
    /// scheme. When empty, runs the endpoint-aware auto-detect: intersects
    /// the schemes the credentials can serve with the endpoint's accepted
    /// auth schemes per the matrix, picks the first match in OAuth2 →
    /// OAuth1 → Bearer preference order, and falls back to that fixed
    /// order for [`RequestTarget::RawUrl`] and matrix-miss
    /// [`RequestTarget::Template`] targets (the permissive surface for
    /// unknown endpoints).
    fn select_scheme(
        &self,
        credentials: &CredentialSource,
        options: &RequestOptions,
    ) -> Result<Selection> {
        let method_raw = options.method.to_uppercase();
        let method = if method_raw.is_empty() {
            "GET"
        } else {
            method_raw.as_str()
        };
        // One matrix lookup for the entire decision: the explicit-auth
        // validation and the auto-detect intersection both consume it.
        let endpoint = Endpoint::of(&options.target, method);
        if options.auth_type.is_empty() {
            self.detect_scheme(credentials, options, method, endpoint.as_ref())
        } else {
            self.explicit_scheme(credentials, options, method, endpoint.as_ref())
        }
    }

    /// The scheme the caller named, refused when the matrix knows the
    /// endpoint and the endpoint does not accept it. An endpoint the matrix
    /// does not know is permissive.
    fn explicit_scheme(
        &self,
        credentials: &CredentialSource,
        options: &RequestOptions,
        method: &str,
        endpoint: Option<&Endpoint<'_>>,
    ) -> Result<Selection> {
        let auth_type = &options.auth_type;
        if let Some(endpoint) = endpoint {
            let requested = auth_type.to_ascii_lowercase();
            if !endpoint.accepts(&requested) {
                return Err(Error::from(AuthMismatch {
                    requested: Some(requested),
                    ..endpoint.mismatch(credentials.active_app())
                }));
            }
        }
        let url = self.build_url(&options.target)?;
        match WireScheme::from_wire(auth_type) {
            Some(scheme) => self.selection(credentials, options, method, scheme, || Ok(url)),
            None => Err(Error::auth(format!("invalid auth type: {auth_type}"))),
        }
    }

    /// The header for `scheme`, or the `OAuth2` user whose token is read
    /// once the credential lock is released. `url` is built only for
    /// `OAuth1`, the one scheme that signs it.
    fn selection(
        &self,
        credentials: &CredentialSource,
        options: &RequestOptions,
        method: &str,
        scheme: WireScheme,
        url: impl FnOnce() -> Result<String>,
    ) -> Result<Selection> {
        let header = match scheme {
            WireScheme::OAuth2 => {
                return Ok(Selection::OAuth2 {
                    username: options.username.clone(),
                });
            }
            WireScheme::OAuth1 => credentials.oauth1_header(method, &url()?)?,
            WireScheme::App => credentials.bearer_header()?,
        };
        Ok(Selection::Header { scheme, header })
    }

    /// The first scheme in preference order the credentials can serve and
    /// the endpoint accepts. An endpoint the matrix does not know accepts
    /// every scheme.
    fn detect_scheme(
        &self,
        credentials: &CredentialSource,
        options: &RequestOptions,
        method: &str,
        endpoint: Option<&Endpoint<'_>>,
    ) -> Result<Selection> {
        let facts = credentials.scheme_facts();
        let selected = WireScheme::ALL_BY_PREFERENCE.into_iter().find(|scheme| {
            facts.available.contains(scheme)
                && endpoint.is_none_or(|endpoint| endpoint.accepts(scheme.as_wire()))
        });
        match selected {
            None => Err(no_scheme(credentials, facts, endpoint)),
            Some(scheme) => self.selection(credentials, options, method, scheme, || {
                self.build_url(&options.target)
            }),
        }
    }
}

/// A request's endpoint as the auth matrix knows it: its path template, its
/// method, and the schemes it accepts in their wire spelling.
struct Endpoint<'a> {
    path: &'a str,
    method: &'a str,
    target: &'a RequestTarget,
    schemes: Vec<&'static str>,
}

impl<'a> Endpoint<'a> {
    /// The matrix entry for `target`, or `None` for a raw URL and for a
    /// template the matrix does not list.
    fn of(target: &'a RequestTarget, method: &'a str) -> Option<Self> {
        let RequestTarget::Template { path, .. } = target else {
            return None;
        };
        let schemes = crate::api::auth_matrix::supported_auth(method, path)?;
        Some(Self {
            path,
            method,
            target,
            schemes: crate::api::auth_matrix::schemes_to_wire_list(schemes),
        })
    }

    fn accepts(&self, scheme: &str) -> bool {
        self.schemes.contains(&scheme)
    }

    /// The mismatch for this endpoint under `app`, with the fields every
    /// rejection shares. A rejection path sets what it alone knows:
    /// `requested`, `available_in_app`, or `other_apps_with_creds`.
    fn mismatch(&self, app: Option<String>) -> AuthMismatch {
        AuthMismatch {
            endpoint: self.path.to_string(),
            rendered_url: render_template_template(self.target).ok(),
            method: self.method.to_string(),
            requested: None,
            supported: self.schemes.iter().map(|s| (*s).to_string()).collect(),
            available_in_app: None,
            app,
            other_apps_with_creds: None,
        }
    }
}

/// The error for a request no credential at hand can serve.
///
/// An endpoint the matrix knows gets the typed mismatch; one it does not
/// know gets the generic auth error, because there is no endpoint to name.
fn no_scheme(
    credentials: &CredentialSource,
    facts: SchemeFacts,
    endpoint: Option<&Endpoint<'_>>,
) -> Error {
    let Some(endpoint) = endpoint else {
        return Error::auth(crate::error::NO_AUTH_METHOD);
    };
    let mut other_apps_with_creds = None;
    if facts.stored.is_empty() {
        // The active app stores nothing. When other apps in the store hold
        // credentials, surface a wrong-app envelope (exit 2) instead of
        // generic auth-required (exit 77): the user signed in, just not
        // against the app they invoked. An env bearer is still reported in
        // `available_in_app` so the envelope stays truthful, but it never
        // hides the wrong-app hint.
        let other_apps =
            credentials.other_apps_with_creds(facts.app.as_deref().unwrap_or_default());
        if !other_apps.is_empty() {
            other_apps_with_creds = Some(other_apps);
        } else if facts.available.is_empty() {
            return Error::auth(crate::error::NO_AUTH_METHOD);
        }
    }
    let available_in_app = facts
        .available
        .iter()
        .map(|scheme| scheme.as_wire().to_string())
        .collect();
    Error::from(AuthMismatch {
        available_in_app: Some(available_in_app),
        other_apps_with_creds,
        ..endpoint.mismatch(facts.app)
    })
}
