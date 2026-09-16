//! Auth-scheme selection for a request, and the mismatch envelope that
//! follows an empty intersection between the active app's stored credentials
//! and the endpoint's accepted schemes.

use crate::error::{Error, Result};

use super::render_template_template;
use super::{ApiClient, RequestOptions, RequestTarget};

impl ApiClient {
    /// Gets the authorization header for a request (public accessor for command layer).
    ///
    /// # Errors
    ///
    /// Returns an error if no valid auth method is found, or — when the
    /// auto-detect path resolves an empty intersection between stored
    /// credentials and the endpoint's accepted schemes — an
    /// [`Error::AuthMethodMismatch`] in the empty-intersection shape.
    pub fn get_auth_header_public(&mut self, options: &RequestOptions) -> Result<String> {
        self.get_auth_header(options)
    }

    /// Gets the authorization header for a request.
    ///
    /// When `options.auth_type` is non-empty, dispatches directly to that
    /// scheme. When empty, runs the endpoint-aware auto-detect: intersects
    /// the active app's stored credentials with the endpoint's accepted
    /// auth schemes per the matrix, picks the first match in OAuth2 →
    /// OAuth1 → Bearer preference order, and falls back to that fixed
    /// order for [`RequestTarget::RawUrl`] and matrix-miss
    /// [`RequestTarget::Template`] targets (the permissive surface for
    /// unknown endpoints).
    pub(super) fn get_auth_header(&mut self, options: &RequestOptions) -> Result<String> {
        let auth_type = &options.auth_type;
        let method_raw = options.method.to_uppercase();
        let method = if method_raw.is_empty() {
            "GET"
        } else {
            method_raw.as_str()
        };

        // One matrix lookup for the entire decision. Both the explicit-auth
        // validation below and the auto-detect intersection further down
        // consume this, eliminating the prior duplicate `supported_auth`
        // call that fired from `auth_matrix::validate` plus a second pass
        // here.
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
                    let raw_app = self.auth.app_name();
                    let app_name = if raw_app.is_empty() {
                        self.auth.token_store.default_app.clone()
                    } else {
                        raw_app.to_string()
                    };
                    return Err(Error::AuthMethodMismatch {
                        endpoint: path.clone(),
                        rendered_url,
                        method: method.to_string(),
                        requested: Some(requested_norm),
                        supported,
                        available_in_app: None,
                        app: Some(app_name),
                        other_apps_with_creds: None,
                    });
                }
            }
            let url = self.build_url(&options.target)?;
            return match auth_type.to_lowercase().as_str() {
                "oauth1" => self.auth.get_oauth1_header(method, &url, None),
                "oauth2" => self.auth.get_oauth2_header(&options.username),
                "app" => self.auth.get_bearer_token_header(),
                _ => Err(Error::auth(format!("invalid auth type: {auth_type}"))),
            };
        }

        // Auto-detect: scope every check to the active app (set by
        // `--app NAME` via `Auth::with_app_name`) so a `--app NAME`
        // invocation without `--auth` picks NAME's tokens, not the
        // default app's. Per-app probes route through `_for_app(app_name)`
        // accessors on the token store so the active-app contract holds
        // even when the active app differs from the default.
        let raw_app = self.auth.app_name();
        // Empty active app name is the "use default app" convention. Resolve
        // it to the store's actual default_app name so the envelope's `app`
        // field carries something the user can act on (e.g. "default" rather
        // than "").
        let app_name = if raw_app.is_empty() {
            self.auth.token_store.default_app.clone()
        } else {
            raw_app.to_string()
        };
        let stored_in_app = self.stored_auth_in_app(&app_name);
        let available_in_app = self.available_auth_in_app(&stored_in_app);

        // Auto-detect filter: walk the preference order and keep every
        // scheme the active app has, optionally intersected with the
        // endpoint's accepted set. The closure expression collapses the
        // two earlier branches that duplicated the availability filter.
        let endpoint_supported_static: Option<Vec<&'static str>> = endpoint_schemes
            .as_ref()
            .map(|(_, schemes)| crate::api::auth_matrix::schemes_to_wire_list(schemes));
        let candidate_order: Vec<crate::api::auth_matrix::WireScheme> =
            crate::api::auth_matrix::WireScheme::ALL_BY_PREFERENCE
                .into_iter()
                .filter(|m| {
                    let wire = m.as_wire();
                    let in_app = available_in_app.contains(&wire);
                    let in_endpoint = endpoint_supported_static
                        .as_ref()
                        .is_none_or(|sup| sup.contains(&wire));
                    in_app && in_endpoint
                })
                .collect();

        if candidate_order.is_empty() {
            // Empty intersection (or empty active app entirely). The
            // matrix-hit branches construct the typed envelope; the
            // matrix-miss branch falls back to the generic auth error
            // because no endpoint context is in scope.
            if let Some((path, _)) = &endpoint_schemes {
                let rendered_url = render_template_template(&options.target).ok();
                let endpoint_supported = endpoint_supported_static
                    .as_ref()
                    .map(|sup| sup.iter().map(|s| (*s).to_string()).collect::<Vec<_>>())
                    .unwrap_or_default();
                if stored_in_app.is_empty() {
                    // Active app stores nothing. Check whether OTHER apps
                    // in the store hold credentials. If so, surface a
                    // wrong-app envelope (exit 2) instead of generic
                    // auth-required (exit 77) — the user logged in, just
                    // not against the app they invoked. The env bearer is
                    // still reported in `available_in_app` so the envelope
                    // stays truthful, but it never hides the wrong-app hint.
                    let other_apps = self.other_apps_with_credentials(&app_name);
                    if !other_apps.is_empty() {
                        return Err(Error::AuthMethodMismatch {
                            endpoint: path.clone(),
                            rendered_url,
                            method: method.to_string(),
                            requested: None,
                            supported: endpoint_supported,
                            available_in_app: Some(
                                available_in_app.iter().map(|s| (*s).to_string()).collect(),
                            ),
                            app: Some(app_name.clone()),
                            other_apps_with_creds: Some(other_apps),
                        });
                    }
                    if available_in_app.is_empty() {
                        return Err(Error::auth(crate::error::NO_AUTH_METHOD));
                    }
                }
                return Err(Error::AuthMethodMismatch {
                    endpoint: path.clone(),
                    rendered_url,
                    method: method.to_string(),
                    requested: None,
                    supported: endpoint_supported,
                    available_in_app: Some(
                        available_in_app.iter().map(|s| (*s).to_string()).collect(),
                    ),
                    app: Some(app_name.clone()),
                    other_apps_with_creds: None,
                });
            }
            return Err(Error::auth(crate::error::NO_AUTH_METHOD));
        }

        // Pick the first candidate in OAuth2 → OAuth1 → Bearer preference
        // order. Dispatching on the typed [`WireScheme`] makes adding a
        // new variant a compile error — the previous `&str`-keyed match
        // could panic at runtime if the candidate list ever grew without
        // a matching arm.
        use crate::api::auth_matrix::WireScheme;
        match candidate_order[0] {
            WireScheme::OAuth2 => self.auth.get_oauth2_header(&options.username),
            WireScheme::OAuth1 => {
                let url = self.build_url(&options.target)?;
                self.auth.get_oauth1_header(method, &url, None)
            }
            WireScheme::App => self.auth.get_bearer_token_header(),
        }
    }

    /// Probes the active app for which auth schemes have stored credentials.
    ///
    /// Returned vector lists the wire strings (`"oauth2"`, `"oauth1"`,
    /// `"app"`) in OAuth2 → OAuth1 → Bearer order, reading the token store
    /// only. Used by the wrong-app decision in [`Self::get_auth_header`],
    /// which must not be swayed by an env-supplied bearer.
    fn stored_auth_in_app(&self, app_name: &str) -> Vec<&'static str> {
        let mut out: Vec<&'static str> = Vec::with_capacity(3);
        if self
            .auth
            .token_store
            .get_first_oauth2_token_for_app(app_name)
            .is_some()
        {
            out.push("oauth2");
        }
        if self
            .auth
            .token_store
            .get_oauth1_tokens_for_app(app_name)
            .is_some()
        {
            out.push("oauth1");
        }
        if self
            .auth
            .token_store
            .get_bearer_token_for_app(app_name)
            .is_some()
        {
            out.push("app");
        }
        out
    }

    /// Extends [`Self::stored_auth_in_app`] with the `"app"` scheme when
    /// `XURL_BEARER_TOKEN` supplies a bearer, since the env value wins the
    /// bearer precedence for every app. Used by the auto-detect intersection
    /// in [`Self::get_auth_header`] and by the error envelopes to populate
    /// `available_in_app`.
    fn available_auth_in_app(&self, stored: &[&'static str]) -> Vec<&'static str> {
        let mut out = stored.to_vec();
        if self.auth.env_bearer_token_present() && !out.contains(&"app") {
            out.push("app");
        }
        out
    }

    /// Names of apps OTHER than `active` that hold at least one stored
    /// credential.
    ///
    /// Used by `get_auth_header` to surface a "wrong-app" envelope when the
    /// active app is empty but the user has credentials elsewhere. Returns
    /// the apps in stable BTreeMap iteration order so the resulting message
    /// is deterministic.
    fn other_apps_with_credentials(&self, active: &str) -> Vec<String> {
        self.auth
            .token_store
            .apps_with_credentials()
            .into_iter()
            .filter(|name| name != active)
            .collect()
    }
}
