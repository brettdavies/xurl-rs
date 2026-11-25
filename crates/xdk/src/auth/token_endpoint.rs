//! The answer the `OAuth2` token endpoint gives a grant, read the same way for
//! the code a sign-in exchanges and for the refresh token a refresh spends.
//!
//! A refusal is a failure status other than a 5xx or a 429 whose JSON body
//! holds one of the error codes of RFC 6749 section 5.2, the one list in
//! [`REFUSAL_CODES`]. It is [`Error::Auth`], of the client or of the grant,
//! since the credential in hand is what was refused. A status alone is not a
//! refusal: X's handler names its error, so a bare 400 or 401, or a JSON body
//! with any other `error`, came from something in front of it.
//!
//! Any other failure is one the endpoint did not process. [`Error::Http`] is
//! no answer. A 404, a 408, a 429, and a 5xx are the [`Error::Api`] any
//! request to X reports. A 403 is [`Error::TokenEndpointForbidden`], and every
//! other status, a redirect included since no request to the token endpoint
//! follows one, is [`Error::TokenEndpointUnexpected`], since the token URL or
//! a proxy in front of it is what to look at. Both grants report every such
//! answer as an `xdk::auth` warning that names the token endpoint and the
//! status, since the error alone does not name the status. A token URL no
//! request can go to is [`Error::InvalidUrl`].

use serde_json::Value;

use super::answer_text::Sent;
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

/// Who an RFC 6749 section 5.2 error code says was refused.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Refused {
    /// The client's own credentials, which another grant would not mend.
    Client,
    /// The authorization code or the refresh token.
    Grant,
}

/// An error code that makes an answer a refusal, and who it refuses.
struct Code {
    name: &'static str,
    refused: Refused,
}

/// The error codes that make an answer a refusal. RFC 6749 section 5.2 lists
/// these six, and X's documentation of the token endpoint's errors
/// (docs.x.com, `fundamentals/authentication/oauth-2-0/oauth-1-0a-token-exchange`)
/// names five of them on a 400: `invalid_request`, `invalid_grant`,
/// `invalid_scope`, `unauthorized_client`, and `invalid_client`. It names two
/// more, `rate_limited` and `temporarily_unavailable`, only on a 429 and a
/// 503, which are never refusals.
const REFUSAL_CODES: [Code; 6] = [
    Code {
        name: "invalid_request",
        refused: Refused::Grant,
    },
    Code {
        name: "invalid_client",
        refused: Refused::Client,
    },
    Code {
        name: "invalid_grant",
        refused: Refused::Grant,
    },
    Code {
        name: "unauthorized_client",
        refused: Refused::Client,
    },
    Code {
        name: "unsupported_grant_type",
        refused: Refused::Grant,
    },
    Code {
        name: "invalid_scope",
        refused: Refused::Grant,
    },
];

/// A successful answer: the access token, and the body it came in.
pub(crate) struct Answer {
    pub(crate) access_token: String,
    pub(crate) body: Value,
}

/// A `POST` to the token endpoint at `token_url`, on a client that follows no
/// redirect. The grant's form holds the code and its verifier, or the refresh
/// token, and a 307 or a 308 would send it again to the host the `Location`
/// names. A redirect is an answer the endpoint did not process, read as its
/// status.
///
/// # Errors
///
/// Returns [`Error::Http`] when the HTTP client cannot be built.
pub(crate) fn post(token_url: &str) -> Result<reqwest::RequestBuilder> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| Error::http(format!("cannot build the HTTP client: {e}")).with_source(e))?;
    Ok(client.post(token_url))
}

/// Sends `request`, which [`post`] built, to the token endpoint at `token_url`
/// and reads its answer.
///
/// A failure status is a refusal only when its body is JSON whose `error` is
/// one of [`REFUSAL_CODES`] and it is no 5xx or 429, so a proxy's error page,
/// or a bare 400 or 401, reads as the status it came with. The text of a
/// failure answer is masked of what `sent` names and cut short, since a page
/// can repeat the request.
///
/// # Errors
///
/// Returns the failure the module documents.
pub(crate) async fn answer(
    request: reqwest::RequestBuilder,
    token_url: &str,
    grant: Grant,
    sent: &Sent,
) -> Result<Answer> {
    let response = request
        .send()
        .await
        .map_err(|e| unsent(e, token_url, grant))?;

    let status = response.status();
    if !status.is_success() {
        let error = match api_error(response).await {
            Ok(error) => error,
            Err(read) => {
                warn_of_answer(grant, status);
                return Err(broke_off(read, grant));
            }
        };
        if let Some((code, body)) = refusal_body(status, &error) {
            return Err(refused(status, code, &body, grant, sent));
        }
        warn_of_answer(grant, status);
        return Err(unprocessed(error, sent));
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
/// `status`, which the error that follows does not carry.
fn warn_of_answer(grant: Grant, status: reqwest::StatusCode) {
    tracing::warn!(
        target: "xdk::auth",
        "the token endpoint answered {} with HTTP {status}",
        grant.request()
    );
}

/// What the answer's body names, when it refuses: a failure status other than
/// a 5xx or a 429 whose JSON body holds a string `error` that is one of
/// [`REFUSAL_CODES`].
fn refusal_body(status: reqwest::StatusCode, error: &Error) -> Option<(&'static Code, Value)> {
    if status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return None;
    }
    let Error::Api { body, .. } = error else {
        return None;
    };
    let object: Value = serde_json::from_str(body).ok()?;
    let named = object["error"].as_str()?;
    let code = REFUSAL_CODES.iter().find(|code| code.name == named)?;
    Some((code, object))
}

/// The failure an answer that is no refusal is. A 404, a 408, a 429, and a 5xx
/// read as they do on any request to X, a 403 is the token endpoint's own, and
/// every other status is one the module reports with its status.
fn unprocessed(error: Error, sent: &Sent) -> Error {
    match error {
        Error::Api {
            status: 403, body, ..
        } => Error::TokenEndpointForbidden {
            body: sent.fit_to_print(&body),
        },
        Error::Api {
            status,
            body,
            reset_at,
        } if matches!(status, 404 | 408 | 429 | 500..=599) => Error::Api {
            status,
            body: sent.fit_to_print(&body),
            reset_at,
        },
        Error::Api { status, body, .. } => Error::TokenEndpointUnexpected {
            status,
            body: sent.fit_to_print(&body),
        },
        other => other,
    }
}

/// An answer whose body broke off before it ended: no answer to read, so the
/// same request can go again.
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
/// `error` and `error_description`.
fn refused(
    status: reqwest::StatusCode,
    code: &Code,
    body: &Value,
    grant: Grant,
    sent: &Sent,
) -> Error {
    let mut message = format!(
        "{}: HTTP {status} — {}",
        refusal(status, code, grant),
        code.name
    );
    if let Some(description) = body["error_description"].as_str() {
        message.push_str(&format!(": {description}"));
    }
    Error::auth(sent.fit_to_print(&message))
}

/// Which refusal an error answer is: of the client (a 401, or a code RFC 6749
/// section 5.2 lets arrive as a 400), or of the grant.
fn refusal(status: reqwest::StatusCode, code: &Code, grant: Grant) -> &'static str {
    if status == reqwest::StatusCode::UNAUTHORIZED || code.refused == Refused::Client {
        grant.client_refused()
    } else {
        grant.grant_refused()
    }
}
