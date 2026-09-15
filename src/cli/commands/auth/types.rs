//! The bearer-source enum and the two redirect-URI response shapes. The
//! schema generator is their second consumer, so their field sets are a wire
//! contract. `AppStatusEntry`, which `auth status` and `apps list` answer
//! with, stays in the parent beside the builder that fills it, because its
//! fields are private and both the builder and the text renderers read them.
//! No credential or token value belongs in a shape declared here: these
//! render to stdout under `--output json`.

use serde::Serialize;

use crate::config::ResolveSource;

/// Origin of the bearer token an `AppStatusEntry` reports as present.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum BearerSource {
    /// Supplied by `XURL_BEARER_TOKEN`, which wins over any stored bearer.
    Env,
    /// Stored on the app in the token store.
    Store,
}

impl BearerSource {
    /// Suffix rendered after the bearer mark in `auth status` text mode.
    pub(super) fn as_text_label(self) -> &'static str {
        match self {
            Self::Env => " [XURL_BEARER_TOKEN environment variable]",
            Self::Store => "",
        }
    }
}

/// Response shape for `xr auth apps redirect-uri get` under `--output json`.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub(crate) struct RedirectUriGetResponse {
    /// App name the lookup targeted.
    pub app: String,
    /// Effective redirect URI after applying the precedence chain.
    pub effective_redirect_uri: String,
    /// Precedence layer that produced [`Self::effective_redirect_uri`].
    pub effective_source: ResolveSource,
    /// Stored per-app redirect URI; `None` when no value is stored.
    pub stored_redirect_uri: Option<String>,
}

/// Response shape for `xr auth apps redirect-uri set` under `--output json`.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub(crate) struct RedirectUriSetResponse {
    /// Always `"ok"` on success.
    pub status: &'static str,
    /// App name the write targeted.
    pub app: String,
    /// Redirect URI that was persisted.
    pub redirect_uri: String,
}
