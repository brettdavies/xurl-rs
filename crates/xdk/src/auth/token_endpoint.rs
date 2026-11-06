//! The answer the `OAuth2` token endpoint gives a grant, read the same way for
//! the code a sign-in exchanges and for the refresh token a refresh spends.
//!
//! A refusal is an answer RFC 6749 section 5.2 shapes: a 400 or a 401, or a JSON
//! body with a string `error` on any other status but a 5xx or a 429. It is
//! [`Error::Auth`], of the client or of the grant, since the credential in hand
//! is what was refused.
//!
//! Any other failure is one the endpoint did not process, and it is the one
//! any request to X reports: [`Error::Http`] with no answer, and [`Error::Api`]
//! for a failure status, such as a 5xx, a 429, a 404, or a 408. A 403 is
//! [`Error::TokenEndpointForbidden`], since the token URL or a proxy in front
//! of it is what to look at. Both grants report every such answer as an
//! `xdk::auth` warning that names the token endpoint and the status, since the
//! error alone says neither. A token URL no request can go to is
//! [`Error::InvalidUrl`].

use serde_json::Value;

use crate::api::api_error;
use crate::error::{
    Error, REFRESH_CLIENT_REFUSED, REFRESH_NO_TOKEN, REFRESH_TOKEN_REFUSED, Result,
    TOKEN_EXCHANGE_CLIENT_REFUSED, TOKEN_EXCHANGE_CODE_REFUSED, TOKEN_EXCHANGE_NO_TOKEN,
    TOKEN_URL_UNUSABLE,
};

/// The grant a request to the token endpoint carries, which names its
/// failures.
#[derive(Clone, Copy)]
pub(crate) enum Grant {
    /// The authorization code a sign-in exchanges.
    Code,
    /// The refresh token a refresh spends.
    Refresh,
}

impl Grant {
    fn request(self) -> &'static str {
        match self {
            Self::Code => "the sign-in's code exchange",
            Self::Refresh => "the OAuth2 login's refresh",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Code => "TokenExchangeError",
            Self::Refresh => "RefreshTokenError",
        }
    }

    fn client_refused(self) -> &'static str {
        match self {
            Self::Code => TOKEN_EXCHANGE_CLIENT_REFUSED,
            Self::Refresh => REFRESH_CLIENT_REFUSED,
        }
    }

    fn grant_refused(self) -> &'static str {
        match self {
            Self::Code => TOKEN_EXCHANGE_CODE_REFUSED,
            Self::Refresh => REFRESH_TOKEN_REFUSED,
        }
    }

    fn no_token(self) -> &'static str {
        match self {
            Self::Code => TOKEN_EXCHANGE_NO_TOKEN,
            Self::Refresh => REFRESH_NO_TOKEN,
        }
    }
}

/// A successful answer: the access token, and the body it came in.
pub(crate) struct Answer {
    pub(crate) access_token: String,
    pub(crate) body: Value,
}

/// Sends `request` to the token endpoint at `token_url` and reads its answer.
///
/// A 400 or a 401 is a refusal by its status alone. Any other failure status
/// is one only when its body is JSON with a string `error`, so a proxy's error
/// page reads as the status it came with.
///
/// # Errors
///
/// Returns the failure the module documents.
pub(crate) async fn answer(
    request: reqwest::RequestBuilder,
    token_url: &str,
    grant: Grant,
) -> Result<Answer> {
    let response = request
        .send()
        .await
        .map_err(|e| unsent(e, token_url, grant))?;

    let status = response.status();
    if matches!(
        status,
        reqwest::StatusCode::BAD_REQUEST | reqwest::StatusCode::UNAUTHORIZED
    ) {
        let body = response.json::<Value>().await.unwrap_or_default();
        return Err(refused(status, &body, grant));
    }
    if !status.is_success() {
        let error = match api_error(response).await {
            Ok(error) => error,
            Err(read) => {
                warn_of_answer(grant, status);
                return Err(broke_off(read, grant));
            }
        };
        if let Some(object) = error_object(status, &error) {
            return Err(refused(status, &object, grant));
        }
        warn_of_answer(grant, status);
        return Err(match error {
            Error::Api {
                status: 403, body, ..
            } => Error::TokenEndpointForbidden { body },
            other => other,
        });
    }

    // A success whose body breaks off is no answer, which the same request can
    // get past; one that reads whole and is not JSON holds no token.
    let bytes = response
        .bytes()
        .await
        .map_err(|e| broke_off(Error::from(e), grant))?;
    let body: Value = serde_json::from_slice(&bytes)
        .map_err(|e| Error::auth_with_cause(grant.no_token(), &e).with_source(e))?;
    let access_token = body["access_token"]
        .as_str()
        .ok_or_else(|| Error::auth(grant.no_token()))?
        .to_string();
    Ok(Answer { access_token, body })
}

/// A request that never got an answer. A token URL no request can go to is
/// the caller's to fix, so a second try would fail the same way; any other
/// failure is the transport's, and the cause chain says which one, such as a
/// refused connection or a timeout.
fn unsent(error: reqwest::Error, token_url: &str, grant: Grant) -> Error {
    if error.is_builder() {
        return Error::invalid_url(format!(
            "{TOKEN_URL_UNUSABLE} '{token_url}' is not one a request can go to"
        ))
        .with_source(error);
    }
    let cause = cause_chain(&error);
    Error::http(format!(
        "{}: the token endpoint did not answer (cause: {cause})",
        grant.label()
    ))
    .with_source(error)
}

/// Says that the token endpoint answered the request `grant` names with
/// `status`: the error that follows prints the endpoint's body alone, which
/// names neither the request nor the status.
fn warn_of_answer(grant: Grant, status: reqwest::StatusCode) {
    tracing::warn!(
        target: "xdk::auth",
        "the token endpoint answered {} with HTTP {status}",
        grant.request()
    );
}

/// The JSON body of a failure status other than a 5xx or a 429 when it holds
/// a string `error`, which makes the answer a refusal whatever its status.
fn error_object(status: reqwest::StatusCode, error: &Error) -> Option<Value> {
    if status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return None;
    }
    let Error::Api { body, .. } = error else {
        return None;
    };
    let object: Value = serde_json::from_str(body).ok()?;
    object["error"].is_string().then_some(object)
}

/// A failure status whose body broke off before it ended: no answer to read,
/// so the same request can go again.
fn broke_off(error: Error, grant: Grant) -> Error {
    let cause = std::error::Error::source(&error).map_or_else(|| error.to_string(), cause_chain);
    Error::http(format!(
        "{}: the token endpoint's answer broke off (cause: {cause})",
        grant.label()
    ))
    .with_source(error)
}

/// `error` and every error under it, joined, so a message keeps the cause a
/// top-level transport error leaves out.
fn cause_chain(error: &dyn std::error::Error) -> String {
    let mut chain = error.to_string();
    let mut next = error.source();
    while let Some(inner) = next {
        chain.push_str(": ");
        chain.push_str(&inner.to_string());
        next = inner.source();
    }
    chain
}

/// The refusal an error answer states: the status, and the endpoint's own
/// `error` and `error_description` when it sent them.
fn refused(status: reqwest::StatusCode, body: &Value, grant: Grant) -> Error {
    let error_code = body["error"].as_str();
    let description = body["error_description"].as_str();
    let mut message = format!(
        "{}: HTTP {status}",
        refusal(status, error_code.unwrap_or_default(), grant)
    );
    match (error_code, description) {
        (Some(code), Some(description)) => {
            message.push_str(&format!(" — {code}: {description}"));
        }
        (Some(said), None) | (None, Some(said)) => message.push_str(&format!(" — {said}")),
        (None, None) => {}
    }
    Error::auth(message)
}

/// Which refusal an error answer is: of the client (RFC 6749 section 5.2 lets
/// that arrive as a 400), or of the grant.
fn refusal(status: reqwest::StatusCode, error_code: &str, grant: Grant) -> &'static str {
    if status == reqwest::StatusCode::UNAUTHORIZED
        || matches!(error_code, "invalid_client" | "unauthorized_client")
    {
        grant.client_refused()
    } else {
        grant.grant_refused()
    }
}
