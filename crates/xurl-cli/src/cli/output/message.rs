//! The text `xr` prints for a library error.
//!
//! The library's Display strings are bare fragments; the prefixes and the
//! recovery wording here are part of `xr`'s output contract and stay
//! byte-identical across releases.

use crate::cli::hints::NextStep;
use xdk::api::auth_matrix::WireScheme;
use xdk::error::{
    AUTH_URL_UNUSABLE, AuthMismatch, Error, MismatchShape, NextAction, STORE_DIR_UNCREATABLE,
    TOKEN_URL_UNUSABLE,
};

/// The environment variable that sets the URL an [`Error::InvalidUrl`] message
/// names, for the two URLs a sign-in reads from it.
fn url_variable(message: &str) -> Option<&'static str> {
    [
        (TOKEN_URL_UNUSABLE, "TOKEN_URL"),
        (AUTH_URL_UNUSABLE, "AUTH_URL"),
    ]
    .into_iter()
    .find_map(|(prefix, variable)| message.starts_with(prefix).then_some(variable))
}

/// The message `xr` prints for `error`, in text mode and in the envelope.
pub(crate) fn render(error: &Error) -> String {
    match error {
        Error::Http { message, .. } => format!("HTTP Error: {message}"),
        Error::Io { message, .. } if message.starts_with(STORE_DIR_UNCREATABLE) => format!(
            "IO Error: {message}. Change the permissions of the directory that holds it, or set XURL_TOKEN_STORE to a writable path"
        ),
        Error::Io { message, .. } => format!("IO Error: {message}"),
        Error::InvalidMethod(method) => format!("Invalid Method: Invalid HTTP method: {method}"),
        Error::Validation { rule, value } => {
            super::validation::sentence(rule, value.as_deref()).unwrap_or_else(|| error.to_string())
        }
        Error::TokenEndpointForbidden { body } if is_empty(body) => {
            "the token endpoint answered 403 with an empty body".to_string()
        }
        Error::TokenEndpointUnexpected { status, body } if is_empty(body) => {
            format!("the token endpoint answered {status} with an empty body")
        }
        Error::Api { .. }
        | Error::TokenEndpointForbidden { .. }
        | Error::TokenEndpointUnexpected { .. }
        | Error::ProcessingTimeout { .. }
        | Error::AddressInUse { .. }
        | Error::MediaProcessingFailed { .. } => error.to_string(),
        Error::AccountNotFound { account } => {
            format!("X found no account for the handle @{account}.")
        }
        Error::InvalidUrl { message, .. } => match url_variable(message) {
            Some(variable) => format!("Invalid URL: {message}; {variable} sets it"),
            None => format!("Invalid URL: {message}"),
        },
        Error::InvalidPathParam { name, value } => {
            format!(
                "Invalid path parameter {name:?}: value {value:?} contains a reserved character"
            )
        }
        Error::Internal(msg) => format!("Internal error: {msg}"),
        Error::Json { message, .. } => format!("JSON Error: {message}"),
        Error::Auth { message, .. } => format!("Auth Error: {message}"),
        Error::TokenStore { message, .. } => format!("Token Store Error: {message}"),
        Error::AppNotFound { .. } => format!("Token Store Error: {error}"),
        Error::AuthMethodMismatch(mismatch) => auth_method_mismatch_message(mismatch, None),
        // `Error` is `#[non_exhaustive]`; a variant this build has not seen
        // prints its own fragment rather than failing to compile downstream.
        _ => error.to_string(),
    }
}

/// Whether an answer's body says nothing: no bytes, or the `{}` an empty body
/// is carried as.
fn is_empty(body: &str) -> bool {
    matches!(body.trim(), "" | "{}")
}

/// The message `xr` prints for `error` when `step` rides with it. An
/// `AuthMethodMismatch` that names a command to add credentials names the
/// step's own, so the message and the step cannot send the caller to
/// different commands.
pub(crate) fn render_with_step(error: &Error, step: &NextStep) -> String {
    match error {
        Error::AuthMethodMismatch(mismatch) => auth_method_mismatch_message(mismatch, Some(step)),
        _ => render(error),
    }
}

/// Builds the message `xr` prints for `AuthMethodMismatch`.
///
/// Three shapes per the variant's docstring; each ends with an actionable
/// recovery instruction. Prefers `rendered_url` over `endpoint` for
/// user-facing strings so `{id}` placeholders don't leak into messages.
fn auth_method_mismatch_message(mismatch: &AuthMismatch, step: Option<&NextStep>) -> String {
    let display_path = mismatch
        .rendered_url
        .as_deref()
        .unwrap_or(&mismatch.endpoint);
    let method = &mismatch.method;
    let supported = &mismatch.supported;
    let app_name = mismatch.app.as_deref().unwrap_or("the active app");
    let list = |items: &[String]| {
        if items.is_empty() {
            "none".to_string()
        } else {
            items.join(", ")
        }
    };

    match mismatch.shape() {
        MismatchShape::Explicit { requested } => {
            let pretty_req = pretty_scheme(requested);
            let alt = supported
                .iter()
                .map(|s| format!("--auth {s}"))
                .collect::<Vec<_>>()
                .join(" or ");
            if alt.is_empty() {
                format!("{pretty_req} auth is not accepted at {method} {display_path}.")
            } else {
                format!("{pretty_req} auth is not accepted at {method} {display_path}. Use {alt}.")
            }
        }
        MismatchShape::WrongApp { others } => {
            let alts = others.join(", ");
            let accepts = list(supported);
            format!(
                "App '{app_name}' has no stored credentials, but other apps do ({alts}). Endpoint {method} {display_path} accepts: {accepts}. Try --app NAME with one of the apps above."
            )
        }
        MismatchShape::EmptyIntersection { available } => {
            let has = list(available);
            let accepts = list(supported);
            let suggest = step.map(credential_tail).unwrap_or_default();
            format!(
                "No stored auth method on app '{app_name}' is accepted at {method} {display_path}. App has: {has}. Endpoint accepts: {accepts}.{suggest}"
            )
        }
        MismatchShape::Unknown => {
            format!("Auth method is not accepted at {method} {display_path}.")
        }
    }
}

/// ` Add credentials with: <command>.`, naming the command or template of a
/// step that gets the caller a credential: a sign-in, an app to register, or
/// a template that stores one. A step that sends the request again under
/// another app or scheme adds no credential, so the message names none.
fn credential_tail(step: &NextStep) -> String {
    let runnable = match step.action {
        NextAction::SignIn | NextAction::RegisterApp => {
            step.command.as_deref().or(step.template.as_deref())
        }
        NextAction::FixInput if step.command.is_none() => step.template.as_deref(),
        _ => None,
    };
    runnable
        .map(|runnable| format!(" Add credentials with: {runnable}."))
        .unwrap_or_default()
}

/// Maps a wire-format auth string to its pretty-printed scheme name.
///
/// Delegates to [`WireScheme::pretty`] so the display vocabulary lives in
/// one place. Unknown strings fall back to the input verbatim so a future
/// scheme added to the matrix without an updated pretty mapping still
/// surfaces something readable.
fn pretty_scheme(name: &str) -> String {
    WireScheme::from_wire(name)
        .map(|ws| ws.pretty().to_string())
        .unwrap_or_else(|| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mismatch(
        requested: Option<&str>,
        available_in_app: Option<&[&str]>,
        app: Option<&str>,
        other_apps_with_creds: Option<&[&str]>,
    ) -> Error {
        let strings = |items: &[&str]| items.iter().map(ToString::to_string).collect::<Vec<_>>();
        Error::from(AuthMismatch {
            endpoint: "/2/users/me".into(),
            rendered_url: Some("/2/users/me".into()),
            method: "GET".into(),
            requested: requested.map(str::to_string),
            supported: vec!["oauth2".into(), "oauth1".into()],
            available_in_app: available_in_app.map(strings),
            app: app.map(str::to_string),
            other_apps_with_creds: other_apps_with_creds.map(strings),
        })
    }

    /// The prefixes are the ones the golden fixtures pin; a library Display
    /// change must never reach `xr`'s output.
    #[test]
    fn every_variant_renders_with_the_prefix_xr_has_always_printed() {
        let cases = [
            (
                Error::auth("NoAuthMethod: no authentication method available"),
                "Auth Error: NoAuthMethod: no authentication method available",
            ),
            (
                Error::AppNotFound {
                    name: "nosuchapp".into(),
                    held: Vec::new(),
                },
                "Token Store Error: the store holds no app named \"nosuchapp\"",
            ),
            (
                Error::json("failed to parse media ID from init response"),
                "JSON Error: failed to parse media ID from init response",
            ),
            (
                Error::InvalidMethod("BAD METHOD".into()),
                "Invalid Method: Invalid HTTP method: BAD METHOD",
            ),
            (
                Error::InvalidPathParam {
                    name: "id".into(),
                    value: "1/2".into(),
                },
                "Invalid path parameter \"id\": value \"1/2\" contains a reserved character",
            ),
            (
                Error::http("connection refused"),
                "HTTP Error: connection refused",
            ),
            (Error::io("file not found"), "IO Error: file not found"),
            (
                Error::invalid_url("ftp://example"),
                "Invalid URL: ftp://example",
            ),
            (
                Error::Internal("missing {id}".into()),
                "Internal error: missing {id}",
            ),
            (Error::api(400, "bad request"), "bad request"),
            (Error::validation("bad input"), "bad input"),
            (
                Error::AccountNotFound {
                    account: "ghost".into(),
                },
                "X found no account for the handle @ghost.",
            ),
        ];
        for (error, expected) in cases {
            assert_eq!(render(&error), expected, "{error:?}");
        }
    }

    #[test]
    fn auth_method_mismatch_keeps_its_three_recovery_messages() {
        assert_eq!(
            render(&mismatch(Some("oauth1"), None, Some("default"), None)),
            "OAuth 1.0a auth is not accepted at GET /2/users/me. Use --auth oauth2 or --auth oauth1."
        );
        assert_eq!(
            render(&mismatch(None, Some(&["app"]), Some(""), None)),
            "No stored auth method on app '' is accepted at GET /2/users/me. App has: app. Endpoint accepts: oauth2, oauth1."
        );
        assert_eq!(
            render(&mismatch(
                None,
                Some(&[]),
                Some("default"),
                Some(&["prod", "staging"])
            )),
            "App 'default' has no stored credentials, but other apps do (prod, staging). Endpoint GET /2/users/me accepts: oauth2, oauth1. Try --app NAME with one of the apps above."
        );
        assert_eq!(
            render(&mismatch(None, None, None, None)),
            "Auth method is not accepted at GET /2/users/me."
        );
    }

    /// The command a mismatch names to add credentials is the step's own,
    /// and a step that adds none leaves the message naming none.
    #[test]
    fn a_mismatch_names_the_command_of_the_step_that_adds_a_credential() {
        let empty = mismatch(None, Some(&["app"]), Some("default"), None);
        let base = "No stored auth method on app 'default' is accepted at GET /2/users/me. App has: app. Endpoint accepts: oauth2, oauth1.";
        let register = NextStep::register_app();
        let template = register.template.clone().unwrap_or_default();
        let cases = [
            (
                NextStep::sign_in_as(None, None, true),
                format!("{base} Add credentials with: xr auth oauth2 --no-browser --step 1."),
            ),
            (
                register,
                format!("{base} Add credentials with: {template}."),
            ),
            (
                NextStep::fix_input_template("xr auth oauth1 --app default <keys>".into()),
                format!("{base} Add credentials with: xr auth oauth1 --app default <keys>."),
            ),
            (
                NextStep::select_app("xr --app other whoami".into()),
                base.to_string(),
            ),
        ];
        for (step, expected) in cases {
            assert_eq!(render_with_step(&empty, &step), expected, "{step:?}");
        }
    }
}
