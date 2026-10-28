//! The text answer to a dry run.
//!
//! Structured output carries the dry-run envelope, which an agent branches
//! on. A person reading text output needs the facts its keys hold: that
//! nothing was sent, whether the real run would succeed, and why not. The
//! summary is built from the context the command already gives the
//! envelope, so a new command's dry run needs no text of its own.

use serde_json::Value;

use super::validation;

/// The context keys the summary words itself; every other key is an input.
const WORDED: [&str; 4] = ["command", "reason", "auth", "confirmation_required"];

/// A credential refusal: the message the real run's error prints, and the
/// advice its text error prints after it (none under `--quiet`).
pub(super) struct CredentialRefusal<'a> {
    pub(super) message: &'a str,
    pub(super) advice: &'a [String],
}

/// The summary's lines, in the order they print.
///
/// An input refusal carries its validator code in the context's `reason`;
/// a credential refusal arrives as `refusal`, apart from the context, so
/// the error's own keys are never read as inputs.
pub(super) fn summary(
    would_succeed: bool,
    exit_code: i32,
    ctx: &Value,
    refusal: Option<&CredentialRefusal>,
) -> Vec<String> {
    let mut lines = vec![match ctx.get("command").and_then(Value::as_str) {
        Some(command) => format!("Dry run of `{command}`: nothing was sent."),
        None => "Dry run: nothing was sent.".to_string(),
    }];
    if would_succeed {
        lines.push("The real run would succeed, as far as an offline check can tell.".to_string());
    } else {
        let why = match refusal {
            Some(refusal) => Some(refusal.message.to_string()),
            None => ctx
                .get("reason")
                .and_then(Value::as_str)
                .map(|code| validation::sentence(code).unwrap_or_else(|| code.to_string())),
        };
        lines.push(match why {
            Some(why) => format!("The real run would fail with exit code {exit_code}: {why}"),
            None => format!("The real run would fail with exit code {exit_code}."),
        });
        if let Some(refusal) = refusal {
            lines.extend(refusal.advice.iter().cloned());
        }
    }
    if let Some(map) = ctx.as_object() {
        for (key, value) in map {
            if WORDED.contains(&key.as_str()) {
                continue;
            }
            if let Some(text) = input_text(value) {
                lines.push(format!("  {key}: {text}"));
            }
        }
    }
    if let Some(auth) = ctx.get("auth") {
        lines.push(format!("  credential: {}", credential_text(auth)));
    }
    if ctx.get("confirmation_required").and_then(Value::as_bool) == Some(true) {
        lines.push(
            "  the real run would stop to ask for confirmation; --force skips it".to_string(),
        );
    }
    lines
}

/// One input's value as a line shows it: a scalar as it is, an empty string
/// quoted so it shows, a list joined with commas, and any other object as
/// compact JSON. An empty list or a null has no line.
fn input_text(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(text) if text.is_empty() => Some("\"\"".to_string()),
        Value::String(text) => Some(text.clone()),
        Value::Array(items) if items.is_empty() => None,
        Value::Array(items) => Some(
            items
                .iter()
                .map(|item| match item {
                    Value::String(text) => text.clone(),
                    other => other.to_string(),
                })
                .collect::<Vec<_>>()
                .join(", "),
        ),
        other => Some(other.to_string()),
    }
}

/// The credential the request would go out under: its scheme, the app it is
/// stored under, the `OAuth2` login, and whether that login's token has
/// expired, each when the envelope's `auth` names it.
fn credential_text(auth: &Value) -> String {
    let field = |key: &str| auth.get(key).and_then(Value::as_str);
    let mut parts = vec![field("scheme").unwrap_or("unknown scheme").to_string()];
    if let Some(app) = field("app") {
        parts.push(format!("app {app}"));
    }
    if let Some(login) = field("username") {
        parts.push(format!("login {login}"));
    }
    match auth.get("token_expired").and_then(Value::as_bool) {
        Some(true) => parts.push("token expired".to_string()),
        Some(false) => parts.push("token current".to_string()),
        None => {}
    }
    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{CredentialRefusal, summary};

    #[test]
    fn a_dry_run_that_would_succeed_names_its_inputs_and_credential() {
        let ctx = json!({
            "command": "post",
            "body": "Hello",
            "media_ids": [],
            "auth": {"scheme": "oauth2", "app": "myapp", "username": "alice", "token_expired": false},
        });
        assert_eq!(
            summary(true, 0, &ctx, None),
            [
                "Dry run of `post`: nothing was sent.",
                "The real run would succeed, as far as an offline check can tell.",
                "  body: Hello",
                "  credential: oauth2, app myapp, login alice, token current",
            ]
        );
    }

    #[test]
    fn an_input_refusal_states_the_rule_and_the_exit_code() {
        let ctx =
            json!({"command": "bookmark-folders-add", "name": "", "reason": "empty-folder-name"});
        assert_eq!(
            summary(false, 1, &ctx, None),
            [
                "Dry run of `bookmark-folders-add`: nothing was sent.",
                "The real run would fail with exit code 1: The folder name is empty.",
                "  name: \"\"",
            ]
        );
    }

    #[test]
    fn a_credential_refusal_carries_the_real_runs_message_and_advice() {
        let ctx = json!({"command": "whoami"});
        let advice = ["Register an app first. Run: <template>".to_string()];
        let refusal = CredentialRefusal {
            message: "Auth Error: NoAuthMethod: no authentication method available",
            advice: &advice,
        };
        assert_eq!(
            summary(false, 77, &ctx, Some(&refusal)),
            [
                "Dry run of `whoami`: nothing was sent.",
                "The real run would fail with exit code 77: Auth Error: NoAuthMethod: no authentication method available",
                "Register an app first. Run: <template>",
            ]
        );
    }

    #[test]
    fn lists_join_and_a_raw_request_has_no_command() {
        let ctx =
            json!({"method": "POST", "url": "/2/tweets", "invitees": ["783214", "2244994945"]});
        assert_eq!(
            summary(true, 0, &ctx, None),
            [
                "Dry run: nothing was sent.",
                "The real run would succeed, as far as an offline check can tell.",
                "  invitees: 783214, 2244994945",
                "  method: POST",
                "  url: /2/tweets",
            ]
        );
    }

    #[test]
    fn a_destructive_command_says_it_would_ask_first() {
        let ctx = json!({"command": "delete", "post_id": "1", "confirmation_required": true});
        let lines = summary(true, 0, &ctx, None);
        assert_eq!(
            lines.last().map(String::as_str),
            Some("  the real run would stop to ask for confirmation; --force skips it")
        );
    }
}
