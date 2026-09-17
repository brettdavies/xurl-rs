//! Auth-scheme selection for a request, and the mismatch envelope that
//! follows an empty intersection between the credentials at hand and the
//! endpoint's accepted schemes.

use tracing::instrument::WithSubscriber;

use crate::api::auth_matrix::WireScheme;
use crate::error::{Error, Result};

use super::render_template_template;
use super::source::CredentialSource;
use super::{Client, RequestOptions, RequestTarget};

/// What scheme selection decided for one request: a header computed from
/// the credentials at hand, or an OAuth2 token that may still need a refresh.
enum Selection {
    Header(String),
    OAuth2 { username: String },
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
            Selection::Header(header) => Ok(header),
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
        let auth_type = &options.auth_type;
        let method_raw = options.method.to_uppercase();
        let method = if method_raw.is_empty() {
            "GET"
        } else {
            method_raw.as_str()
        };

        // One matrix lookup for the entire decision: the explicit-auth
        // validation below and the auto-detect intersection further down
        // both consume it.
        let endpoint_schemes = match &options.target {
            RequestTarget::Template { path, .. } => {
                crate::api::auth_matrix::supported_auth(method, path).map(|s| (path.clone(), s))
            }
            RequestTarget::RawUrl(_) => None,
        };

        if !auth_type.is_empty() {
            // Validate the explicit-auth request against the matrix entry
            // when present. Matrix-miss is permissive per the unknown-
            // endpoint rule. Empty wire list (currently unreachable) would
            // collapse to permissive too — the matrix only emits entries
            // for endpoints that declare a `security:` list.
            if let Some((path, schemes)) = &endpoint_schemes {
                let supported_static = crate::api::auth_matrix::schemes_to_wire_list(schemes);
                let requested_norm = auth_type.to_ascii_lowercase();
                if !supported_static.contains(&requested_norm.as_str()) {
                    let supported: Vec<String> =
                        supported_static.iter().map(|s| (*s).to_string()).collect();
                    let rendered_url = render_template_template(&options.target).ok();
                    return Err(Error::AuthMethodMismatch {
                        endpoint: path.clone(),
                        rendered_url,
                        method: method.to_string(),
                        requested: Some(requested_norm),
                        supported,
                        available_in_app: None,
                        app: credentials.active_app(),
                        other_apps_with_creds: None,
                    });
                }
            }
            let url = self.build_url(&options.target)?;
            return match auth_type.to_lowercase().as_str() {
                "oauth1" => credentials
                    .oauth1_header(method, &url)
                    .map(Selection::Header),
                "oauth2" => Ok(Selection::OAuth2 {
                    username: options.username.clone(),
                }),
                "app" => credentials.bearer_header().map(Selection::Header),
                _ => Err(Error::auth(format!("invalid auth type: {auth_type}"))),
            };
        }

        // Auto-detect: walk the preference order and take the first scheme
        // the credentials can serve, optionally intersected with the
        // endpoint's accepted set.
        let facts = credentials.scheme_facts();
        let endpoint_supported_static: Option<Vec<&'static str>> = endpoint_schemes
            .as_ref()
            .map(|(_, schemes)| crate::api::auth_matrix::schemes_to_wire_list(schemes));
        let selected_scheme = WireScheme::ALL_BY_PREFERENCE.into_iter().find(|m| {
            let in_hand = facts.available.contains(m);
            let in_endpoint = endpoint_supported_static
                .as_ref()
                .is_none_or(|sup| sup.contains(&m.as_wire()));
            in_hand && in_endpoint
        });

        let Some(selected_scheme) = selected_scheme else {
            // Empty intersection (or nothing at hand at all). The matrix-hit
            // branches construct the typed envelope; the matrix-miss branch
            // falls back to the generic auth error because no endpoint
            // context is in scope.
            if let Some((path, _)) = &endpoint_schemes {
                let rendered_url = render_template_template(&options.target).ok();
                let endpoint_supported = endpoint_supported_static
                    .as_ref()
                    .map(|sup| sup.iter().map(|s| (*s).to_string()).collect::<Vec<_>>())
                    .unwrap_or_default();
                let available_in_app: Vec<String> = facts
                    .available
                    .iter()
                    .map(|s| s.as_wire().to_string())
                    .collect();
                if facts.stored.is_empty() {
                    // The active app stores nothing. When other apps in the
                    // store hold credentials, surface a wrong-app envelope
                    // (exit 2) instead of generic auth-required (exit 77):
                    // the user signed in, just not against the app they
                    // invoked. An env bearer is still reported in
                    // `available_in_app` so the envelope stays truthful, but
                    // it never hides the wrong-app hint.
                    let other_apps =
                        credentials.other_apps_with_creds(facts.app.as_deref().unwrap_or_default());
                    if !other_apps.is_empty() {
                        return Err(Error::AuthMethodMismatch {
                            endpoint: path.clone(),
                            rendered_url,
                            method: method.to_string(),
                            requested: None,
                            supported: endpoint_supported,
                            available_in_app: Some(available_in_app),
                            app: facts.app,
                            other_apps_with_creds: Some(other_apps),
                        });
                    }
                    if facts.available.is_empty() {
                        return Err(Error::auth(crate::error::NO_AUTH_METHOD));
                    }
                }
                return Err(Error::AuthMethodMismatch {
                    endpoint: path.clone(),
                    rendered_url,
                    method: method.to_string(),
                    requested: None,
                    supported: endpoint_supported,
                    available_in_app: Some(available_in_app),
                    app: facts.app,
                    other_apps_with_creds: None,
                });
            }
            return Err(Error::auth(crate::error::NO_AUTH_METHOD));
        };

        // Dispatching on the typed [`WireScheme`] makes adding a new variant
        // a compile error.
        match selected_scheme {
            WireScheme::OAuth2 => Ok(Selection::OAuth2 {
                username: options.username.clone(),
            }),
            WireScheme::OAuth1 => {
                let url = self.build_url(&options.target)?;
                credentials
                    .oauth1_header(method, &url)
                    .map(Selection::Header)
            }
            WireScheme::App => credentials.bearer_header().map(Selection::Header),
        }
    }
}
