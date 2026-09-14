//! Recovery hints: the `next_step` object and the text lines beside it.
//!
//! Text output is for humans and JSON for agents, so one built value feeds
//! both renderings: the text lines derive from the same `NextStep` the
//! envelope carries, which keeps a hint and its machine-readable form from
//! drifting apart.

use serde::{Deserialize, Serialize};

use crate::store::snapshot::StoreSnapshot;

/// What the caller should do next. Closed set; agents branch on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum NextAction {
    /// No app carries client credentials; register one.
    RegisterApp,
    /// The target app has credentials and no token; sign in.
    SignIn,
    /// Another app is the one to use; rerun naming it.
    SelectApp,
    /// The store could not be read or parsed; look at the file.
    InspectStore,
    /// X refused the app; enroll it in the developer portal.
    EnrollApp,
}

/// The `next_step` object carried by an error envelope.
///
/// Exactly one of [`Self::command`] and [`Self::template`] is present,
/// except for [`NextAction::EnrollApp`], which carries only `docs`:
/// a command is a verbatim invocation safe for a non-TTY caller, while a
/// template carries angle-bracket placeholders the caller must fill in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NextStep {
    /// The action to take.
    pub action: NextAction,
    /// A runnable invocation, when one exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// A placeholder invocation, when the caller must supply values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// Documentation URL, when a recipe exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
}

impl NextStep {
    /// Registration, which needs values only the caller has.
    #[must_use]
    pub fn register_app() -> Self {
        Self {
            action: NextAction::RegisterApp,
            command: None,
            template: Some(
                "xr auth apps add <name> --client-id <client-id> --client-secret <client-secret>"
                    .to_string(),
            ),
            docs: None,
        }
    }

    /// Sign-in for `app`.
    ///
    /// `headless` selects the two-step form an agent can run without a
    /// browser; callers pass the structured-output flag, since a structured
    /// caller is by definition not watching a browser window.
    #[must_use]
    pub fn sign_in(app: Option<&str>, headless: bool) -> Self {
        let mut command = if headless {
            "xr auth oauth2 --no-browser --step 1".to_string()
        } else {
            "xr auth oauth2".to_string()
        };
        if let Some(name) = app {
            command.push_str(" --app ");
            command.push_str(&quote_app_name(name));
        }
        Self {
            action: NextAction::SignIn,
            command: Some(command),
            template: None,
            docs: None,
        }
    }

    /// Rerun `command` against a different app.
    #[must_use]
    pub fn select_app(command: String) -> Self {
        Self {
            action: NextAction::SelectApp,
            command: Some(command),
            template: None,
            docs: None,
        }
    }

    /// Look at a store file that could not be loaded.
    #[must_use]
    pub fn inspect_store() -> Self {
        Self {
            action: NextAction::InspectStore,
            command: Some("xr auth status".to_string()),
            template: None,
            docs: None,
        }
    }

    /// Enroll the app; the recipe is documentation, not a command.
    #[must_use]
    pub fn enroll_app(docs: String) -> Self {
        Self {
            action: NextAction::EnrollApp,
            command: None,
            template: None,
            docs: Some(docs),
        }
    }

    /// The invocation to show a human: the command, else the template.
    #[must_use]
    pub fn display_invocation(&self) -> Option<&str> {
        self.command.as_deref().or(self.template.as_deref())
    }
}

/// A recovery hint: prose for a human, a typed step for an agent.
///
/// Both renderings derive from the same built value, so the lines a person
/// reads and the object an agent runs cannot name different commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hint {
    /// Lines printed after the error line in text mode.
    pub text_lines: Vec<String>,
    /// The step folded into the envelope in structured modes.
    pub next_step: NextStep,
}

/// Chooses the recovery hint for a no-credentials failure.
///
/// ```text
/// load failed               -> inspect-store, naming the path
/// another app holds tokens  -> select-app, rerunning this invocation
/// target can sign in        -> sign-in
/// another app has a client  -> select-app, signing in against it
/// nothing anywhere          -> register-app, with a template
/// ```
///
/// The snapshot is taken before dispatch, so the choice sees the store the
/// run actually loaded, including an app named by `--app` and a client id
/// supplied by the environment.
#[must_use]
pub fn choose_hint(snapshot: &StoreSnapshot, invocation: &[String], headless: bool) -> Hint {
    let target = snapshot.active_app.as_str();
    let target_facts = snapshot.apps.get(target);

    if snapshot.load_failed() {
        let next_step = NextStep::inspect_store();
        return Hint {
            text_lines: vec![
                format!("The token store could not be read: {}", snapshot.store_path),
                "Inspect or move that file, then retry.".to_string(),
            ],
            next_step,
        };
    }

    let tokens_elsewhere: Vec<String> = snapshot
        .apps_with_tokens()
        .into_iter()
        .filter(|name| name != target)
        .collect();
    if let Some(app) = tokens_elsewhere.first() {
        let command = rerun_with_app(invocation, app);
        return Hint {
            text_lines: vec![format!("App {app:?} is already signed in. Run: {command}")],
            next_step: NextStep::select_app(command),
        };
    }

    let target_has_client_id =
        snapshot.env_client_id_present || target_facts.is_some_and(|f| f.has_client_id);
    if target_has_client_id {
        let next_step = NextStep::sign_in(None, headless);
        let command = next_step.command.clone().unwrap_or_default();
        return Hint {
            text_lines: vec![format!("Sign in first. Run: {command}")],
            next_step,
        };
    }

    let ready: Vec<String> = snapshot
        .apps_ready_to_sign_in()
        .into_iter()
        .filter(|name| name != target)
        .collect();
    if let Some(app) = ready.first() {
        let next_step = NextStep::sign_in(Some(app), headless);
        let command = next_step.command.clone().unwrap_or_default();
        return Hint {
            text_lines: vec![format!(
                "App {app:?} has client credentials. Run: {command}"
            )],
            next_step: NextStep::select_app(command),
        };
    }

    let next_step = NextStep::register_app();
    let template = next_step.template.clone().unwrap_or_default();
    Hint {
        text_lines: vec![format!("Register an app first. Run: {template}")],
        next_step,
    }
}

/// The enrollment recipe an `enroll-app` step points at.
const ENROLLMENT_DOCS: &str = "https://github.com/brettdavies/xurl-rs#x-platform-enrollment";

/// Builds the enrollment hint when a 403 body says X refused the app.
///
/// The two markers both appear in observed refusals, and the body's own
/// `detail` line is quoted when present so the reader can judge whether the
/// match is right rather than trusting the label.
#[must_use]
pub fn enrollment_hint(status: u16, body: &str) -> Option<Hint> {
    if status != 403 {
        return None;
    }
    let haystack = body.to_ascii_lowercase();
    if !haystack.contains("client-not-enrolled") && !haystack.contains("client-forbidden") {
        return None;
    }

    let mut text_lines = Vec::with_capacity(2);
    if let Some(detail) = detail_line(body) {
        text_lines.push(format!("X refused the app: {detail}"));
    } else {
        text_lines.push("X refused the app.".to_string());
    }
    text_lines.push(format!(
        "Move it to the Pay-per-use package and the Production environment. See Troubleshooting: {ENROLLMENT_DOCS}"
    ));

    Some(Hint {
        text_lines,
        next_step: NextStep::enroll_app(ENROLLMENT_DOCS.to_string()),
    })
}

/// Pulls the `detail` string out of a JSON error body, when there is one.
fn detail_line(body: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()?
        .get("detail")?
        .as_str()
        .map(str::to_string)
}

/// Rebuilds this invocation with `--app NAME` inserted after the program.
///
/// The rerun has to be runnable as printed, so the app name is quoted by the
/// same rule registration enforces.
fn rerun_with_app(invocation: &[String], app: &str) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(invocation.len() + 2);
    let mut rest = invocation.iter();
    let program = rest.next().map_or("xr", String::as_str);
    parts.push(program.to_string());
    parts.push("--app".to_string());
    parts.push(quote_app_name(app));
    for arg in rest {
        parts.push(shell_word(arg));
    }
    parts.join(" ")
}

/// Quotes one argument of a rebuilt invocation when a shell would split it.
fn shell_word(arg: &str) -> String {
    if !arg.is_empty()
        && arg.chars().all(|c| {
            c.is_ascii_alphanumeric()
                || matches!(c, '_' | '.' | '-' | '/' | ':' | '=' | '@' | '+' | '~')
        })
    {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', r"'\''"))
    }
}

/// Single-quotes an app name carrying anything registration would reject.
///
/// Registration rejects such names, but a store written by an older version
/// can still hold one, and these strings are printed as runnable commands.
/// The character set comes from [`crate::store::is_app_name_char`], so the
/// rule that rejects a name and the rule that quotes it cannot drift.
#[must_use]
pub fn quote_app_name(name: &str) -> String {
    if !name.is_empty() && name.chars().all(crate::store::is_app_name_char) {
        name.to_string()
    } else {
        format!("'{}'", name.replace('\'', r"'\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_app_offers_a_template_and_no_command() {
        let step = NextStep::register_app();
        assert_eq!(step.action, NextAction::RegisterApp);
        assert!(step.command.is_none(), "never both");
        assert!(step.template.as_deref().unwrap().contains("<client-id>"));
    }

    #[test]
    fn sign_in_picks_the_form_the_caller_can_run() {
        assert_eq!(
            NextStep::sign_in(None, false).command.as_deref(),
            Some("xr auth oauth2")
        );
        assert_eq!(
            NextStep::sign_in(Some("work"), true).command.as_deref(),
            Some("xr auth oauth2 --no-browser --step 1 --app work")
        );
    }

    #[test]
    fn enroll_app_carries_only_docs() {
        let step = NextStep::enroll_app("https://example.test/#enrollment".to_string());
        assert_eq!(step.action, NextAction::EnrollApp);
        assert!(step.command.is_none() && step.template.is_none());
        assert!(step.docs.is_some());
    }

    #[test]
    fn inspect_store_and_select_app_carry_a_runnable_command() {
        assert_eq!(
            NextStep::inspect_store().command.as_deref(),
            Some("xr auth status")
        );
        let step = NextStep::select_app("xr auth oauth2 --app work".to_string());
        assert_eq!(step.action, NextAction::SelectApp);
        assert_eq!(step.command.as_deref(), Some("xr auth oauth2 --app work"));
    }

    #[test]
    fn a_name_the_shell_would_split_is_quoted() {
        assert_eq!(quote_app_name("work"), "work");
        assert_eq!(quote_app_name("my.app-2_x"), "my.app-2_x");
        assert_eq!(quote_app_name("my app"), "'my app'");
        assert_eq!(quote_app_name(""), "''");
        assert_eq!(quote_app_name("it's"), r"'it'\''s'");
    }

    #[test]
    fn display_invocation_prefers_the_runnable_form() {
        assert_eq!(
            NextStep::sign_in(None, false).display_invocation(),
            Some("xr auth oauth2")
        );
        assert!(
            NextStep::register_app()
                .display_invocation()
                .unwrap()
                .contains("<name>")
        );
        assert!(
            NextStep::enroll_app("https://example.test".to_string())
                .display_invocation()
                .is_none()
        );
    }
}
