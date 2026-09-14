//! Recovery hints: the `next_step` object and the text lines beside it.
//!
//! Text output is for humans and JSON for agents, so one built value feeds
//! both renderings: the text lines derive from the same `NextStep` the
//! envelope carries, which keeps a hint and its machine-readable form from
//! drifting apart.

use serde::{Deserialize, Serialize};

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
