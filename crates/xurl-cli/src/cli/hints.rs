//! Recovery hints: the `next_step` object and the text lines beside it.
//!
//! Text output is for humans and JSON for agents, so one built value feeds
//! both renderings: the text lines derive from the same `NextStep` the
//! envelope carries, which keeps a hint and its machine-readable form from
//! drifting apart.

use serde::{Deserialize, Serialize};

use xdk::error::Error;
pub use xdk::error::NextAction;
use xdk::store::snapshot::StoreSnapshot;

use crate::cli::classify::ROOT_COMMAND;

/// The `next_step` object carried by an error envelope.
///
/// At most one of [`Self::command`] and [`Self::template`] is present: a
/// command is a verbatim invocation safe for a non-TTY caller, while a
/// template is one the caller finishes or decides on before running it. It
/// carries angle-bracket placeholders to fill in, or, under
/// [`NextAction::Confirm`], the invocation that destroys once it runs. A
/// step with neither names an action that is not an `xr` invocation, with
/// `docs` when a page covers it: [`NextAction::EnrollApp`],
/// [`NextAction::WaitAndRetry`], [`NextAction::Retry`],
/// [`NextAction::FixInput`], [`NextAction::ReportIssue`], and
/// [`NextAction::InspectStore`] when the command that failed is the one
/// that reports on the store.
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

/// The registration invocation, with a placeholder for each value only the
/// caller has. `<secret-command>` is whatever prints the client secret.
pub(crate) const REGISTER_APP_TEMPLATE: &str =
    "<secret-command> | xr auth apps add <name> --client-id <client-id> --client-secret-file -";

/// The README section on a token store that could not be loaded.
pub(crate) const STORE_DOCS: &str = "https://github.com/brettdavies/xurl-rs/blob/main/crates/xurl-cli/README.md#token-store-could-not-be-read";

impl NextStep {
    /// The help page that answers a mistyped command or a usage error, run
    /// as given.
    #[must_use]
    pub fn show_help(command: String) -> Self {
        Self {
            action: NextAction::ShowHelp,
            command: Some(command),
            template: None,
            docs: None,
        }
    }

    /// Registration, which needs values only the caller has. The secret is
    /// piped in, so filling the template puts no secret in argv.
    #[must_use]
    pub fn register_app() -> Self {
        Self {
            action: NextAction::RegisterApp,
            command: None,
            template: Some(REGISTER_APP_TEMPLATE.to_string()),
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

    /// Look at a store file that could not be loaded, starting from the
    /// command that reports on it and names its path.
    #[must_use]
    pub fn inspect_store() -> Self {
        Self {
            command: Some("xr auth status".to_string()),
            ..Self::inspect_store_page()
        }
    }

    /// [`Self::inspect_store`] for a failure that already names the path: no
    /// `xr` invocation says more, so the step is the page alone.
    #[must_use]
    pub fn inspect_store_page() -> Self {
        Self {
            action: NextAction::InspectStore,
            command: None,
            template: None,
            docs: Some(STORE_DOCS.to_string()),
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

    /// The wait on `media_id` again, for `secs` seconds.
    #[must_use]
    pub fn resume_wait(media_id: &str, secs: u64) -> Self {
        Self {
            action: NextAction::ResumeWait,
            command: Some(format!("xr media status {media_id} --wait={secs}")),
            template: None,
            docs: None,
        }
    }

    /// Send the same request again once the rate limit has reset. There is
    /// no command: the request is the caller's own, and the time to wait is
    /// in the envelope beside this step.
    #[must_use]
    pub fn wait_and_retry(docs: String) -> Self {
        Self {
            action: NextAction::WaitAndRetry,
            command: None,
            template: None,
            docs: Some(docs),
        }
    }

    /// Send the same request again: nothing about it was wrong. `docs` is
    /// the page that says what the answer meant, when X sent one.
    #[must_use]
    pub fn retry(docs: Option<String>) -> Self {
        Self {
            action: NextAction::Retry,
            command: None,
            template: None,
            docs,
        }
    }

    /// Change the input or the local state the command read; no `xr`
    /// command repairs it. `docs` is the page that says what was refused,
    /// when one exists.
    #[must_use]
    pub fn fix_input(docs: Option<String>) -> Self {
        Self {
            action: NextAction::FixInput,
            command: None,
            template: None,
            docs,
        }
    }

    /// Report a fault of `xr` or of the library at `docs`.
    #[must_use]
    pub fn report_issue(docs: String) -> Self {
        Self {
            action: NextAction::ReportIssue,
            command: None,
            template: None,
            docs: Some(docs),
        }
    }

    /// Run `invocation` again with `--force`. It is a template and not a
    /// command: running it destroys what the command names, so the caller
    /// decides.
    #[must_use]
    pub fn confirm(invocation: &[String]) -> Self {
        let mut words: Vec<String> = invocation.iter().map(|arg| shell_word(arg)).collect();
        if let Some(program) = words.first_mut() {
            *program = ROOT_COMMAND.to_string();
        }
        words.push("--force".to_string());
        Self {
            action: NextAction::Confirm,
            command: None,
            template: Some(words.join(" ")),
            docs: None,
        }
    }

    /// Run the one other command that repairs this.
    #[must_use]
    pub fn run_command(command: String) -> Self {
        Self {
            action: NextAction::RunCommand,
            command: Some(command),
            template: None,
            docs: None,
        }
    }

    /// Name the app to make the default; only the caller knows which.
    #[must_use]
    pub fn name_default_app() -> Self {
        Self {
            action: NextAction::SelectApp,
            command: None,
            template: Some("xr auth default <app>".to_string()),
            docs: None,
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
                format!("Inspect or move that file, then retry. See: {STORE_DOCS}"),
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

/// Builds the hint for a store command that failed on a file it could not
/// load.
///
/// The load state decides, not the error's wording: `token-store` is also the
/// reason for a name the store does not hold, where the file loaded and there
/// is nothing to inspect.
#[must_use]
pub fn unloadable_store_hint(snapshot: &StoreSnapshot, error: &Error) -> Option<Hint> {
    if !(matches!(error, Error::TokenStore { .. }) && snapshot.load_failed()) {
        return None;
    }
    Some(Hint {
        text_lines: vec![format!("See: {STORE_DOCS}")],
        next_step: NextStep::inspect_store_page(),
    })
}

/// Builds the enrollment hint when `error` is X refusing the app.
///
/// The body's own `detail` line is quoted when present so the reader can
/// judge whether the match is right rather than trusting the label.
#[must_use]
pub fn enrollment_hint(error: &Error) -> Option<Hint> {
    let (Error::Api { body, .. }, Some(NextAction::EnrollApp), Some(docs)) =
        (error, error.next_action(), error.docs_url())
    else {
        return None;
    };

    let mut text_lines = Vec::with_capacity(2);
    if let Some(detail) = detail_line(body) {
        text_lines.push(format!("X refused the app: {detail}"));
    } else {
        text_lines.push("X refused the app.".to_string());
    }
    text_lines.push(format!(
        "Move it to the Pay-per-use package and the Production environment. See Troubleshooting: {docs}"
    ));

    Some(Hint {
        text_lines,
        next_step: NextStep::enroll_app(docs.to_string()),
    })
}

/// Builds the resume hint when `error` is a processing wait that reached
/// its deadline.
///
/// The command waits twice as long as the wait that expired: a job that
/// outlasted one deadline is unlikely to finish inside the same one again.
#[must_use]
pub fn resume_wait_hint(error: &Error) -> Option<Hint> {
    let (Error::ProcessingTimeout { media_id, waited }, Some(NextAction::ResumeWait)) =
        (error, error.next_action())
    else {
        return None;
    };
    let next_step = NextStep::resume_wait(media_id, waited.as_secs().saturating_mul(2));
    let command = next_step.command.clone().unwrap_or_default();
    Some(Hint {
        text_lines: vec![format!(
            "The upload is intact and X keeps the media id for 24 hours. Resume the wait. Run: {command}"
        )],
        next_step,
    })
}

/// Builds the retry hint when `error` is a 429 that named its reset.
///
/// A 429 that named no reset gets no hint: there is no time to state.
#[must_use]
pub fn wait_and_retry_hint(error: &Error) -> Option<Hint> {
    let (
        Error::Api {
            reset_at: Some(reset_at),
            ..
        },
        Some(NextAction::WaitAndRetry),
        Some(docs),
    ) = (error, error.next_action(), error.docs_url())
    else {
        return None;
    };
    let (retry_after_secs, retry_at) = crate::cli::output::retry_times(*reset_at);
    Some(Hint {
        text_lines: vec![format!(
            "Rate limited. Retry in {retry_after_secs} seconds, at {retry_at}. See: {docs}"
        )],
        next_step: NextStep::wait_and_retry(docs.to_string()),
    })
}

/// Builds the hint for a scheme mismatch the store can answer.
///
/// ```text
/// the active app stores nothing, another does  -> select-app, rerunning this invocation
/// the endpoint takes OAuth2, the app has none  -> the step that gets a login: see choose_hint
/// the caller named the scheme, or neither fits -> none; the message lists what is accepted
/// ```
#[must_use]
pub fn mismatch_hint(
    snapshot: &StoreSnapshot,
    error: &Error,
    invocation: &[String],
    headless: bool,
) -> Option<Hint> {
    let Error::AuthMethodMismatch(mismatch) = error else {
        return None;
    };
    match mismatch.shape() {
        xdk::error::MismatchShape::WrongApp { others } => {
            let app = others.first()?;
            let command = rerun_with_app(invocation, app);
            Some(Hint {
                text_lines: vec![format!("App {app:?} is already signed in. Run: {command}")],
                next_step: NextStep::select_app(command),
            })
        }
        xdk::error::MismatchShape::EmptyIntersection { available }
            if mismatch.supported.iter().any(|scheme| scheme == "oauth2")
                && !available.iter().any(|scheme| scheme == "oauth2") =>
        {
            Some(choose_hint(snapshot, invocation, headless))
        }
        _ => None,
    }
}

/// Builds the hint for an argument the command refused: the help of the
/// command the invocation names.
#[must_use]
pub fn show_help_hint(help: &str) -> Hint {
    Hint {
        text_lines: vec![format!("Try '{help}'.")],
        next_step: NextStep::show_help(help.to_string()),
    }
}

/// Builds the hint for an error whose step the library names and no stored
/// credential decides.
///
/// A method, a URL, or a path value `xr` refused is an argument, so it
/// takes the help of the command it was given to, as `help` names it.
#[must_use]
pub fn general_hint(error: &Error, help: &str) -> Option<Hint> {
    if matches!(
        error,
        Error::InvalidMethod(_) | Error::InvalidUrl { .. } | Error::InvalidPathParam { .. }
    ) {
        return Some(show_help_hint(help));
    }
    let docs = error.docs_url().map(str::to_string);
    match error.next_action()? {
        NextAction::Retry => Some(Hint {
            text_lines: vec!["Nothing was wrong with the request. Send it again.".to_string()],
            next_step: NextStep::retry(docs),
        }),
        NextAction::FixInput => Some(Hint {
            text_lines: Vec::new(),
            next_step: NextStep::fix_input(docs),
        }),
        NextAction::ReportIssue => {
            let docs = docs?;
            Some(Hint {
                text_lines: vec![format!("This is a fault in xr. Report it: {docs}")],
                next_step: NextStep::report_issue(docs),
            })
        }
        _ => None,
    }
}

/// Pulls the `detail` string out of a JSON error body, when there is one.
fn detail_line(body: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()?
        .get("detail")?
        .as_str()
        .map(str::to_string)
}

/// Rebuilds this invocation to run against `app`: `--app NAME` after the
/// program, and without the `--app` the caller passed, which clap would
/// refuse as a second one.
///
/// The rerun has to be runnable as printed, so the program is the name every
/// hint gives the binary and the app name is quoted by the same rule
/// registration enforces. Nothing after `--` is a flag, so it is kept as it
/// was.
fn rerun_with_app(invocation: &[String], app: &str) -> String {
    let mut parts = vec![
        ROOT_COMMAND.to_string(),
        "--app".to_string(),
        quote_app_name(app),
    ];
    let mut rest = invocation.iter().skip(1);
    while let Some(arg) = rest.next() {
        if arg == "--" {
            parts.push(shell_word(arg));
            parts.extend(rest.by_ref().map(|arg| shell_word(arg)));
        } else if arg == "--app" {
            rest.next();
        } else if !arg.starts_with("--app=") {
            parts.push(shell_word(arg));
        }
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
/// The character set comes from [`xdk::store::is_app_name_char`], so the
/// rule that rejects a name and the rule that quotes it cannot drift.
#[must_use]
pub fn quote_app_name(name: &str) -> String {
    if !name.is_empty() && name.chars().all(xdk::store::is_app_name_char) {
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
        let template = step.template.as_deref().unwrap();
        assert!(template.contains("<client-id>"));
        assert!(
            template.ends_with("--client-secret-file -"),
            "the secret is piped, never an argument: {template}"
        );
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
        assert_eq!(NextStep::inspect_store().docs.as_deref(), Some(STORE_DOCS));
        let page = NextStep::inspect_store_page();
        assert_eq!(page.action, NextAction::InspectStore);
        assert!(page.command.is_none() && page.template.is_none());
        assert_eq!(page.docs.as_deref(), Some(STORE_DOCS));
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

    fn step(error: &Error) -> Option<NextStep> {
        general_hint(error, "xr post --help").map(|hint| hint.next_step)
    }

    #[test]
    fn a_failure_that_is_not_the_requests_fault_is_retried() {
        assert_eq!(
            step(&Error::http("connection refused")),
            Some(NextStep::retry(None))
        );
        let server = step(&Error::api(503, "unavailable")).expect("a step");
        assert_eq!(server.action, NextAction::Retry);
        assert!(server.command.is_none() && server.template.is_none());
        assert!(server.docs.is_some(), "X's page on response codes");
    }

    #[test]
    fn a_refused_input_is_fixed_and_carries_no_invocation() {
        let not_found = step(&Error::api(404, "no such post")).expect("a step");
        assert_eq!(not_found.action, NextAction::FixInput);
        assert!(not_found.command.is_none() && not_found.template.is_none());
        assert!(not_found.docs.is_some(), "X's page on response codes");
        assert_eq!(
            step(&Error::io("permission denied")),
            Some(NextStep::fix_input(None))
        );
        assert_eq!(
            step(&Error::validation("media processing failed")),
            Some(NextStep::fix_input(None))
        );
    }

    #[test]
    fn a_fault_of_xr_names_the_issue_tracker() {
        for error in [
            Error::Internal("missing {id}".into()),
            Error::json("expected value"),
            Error::api(418, "teapot"),
        ] {
            let step = step(&error).expect("a step");
            assert_eq!(step.action, NextAction::ReportIssue, "{error:?}");
            assert_eq!(
                step.docs.as_deref(),
                Some("https://github.com/brettdavies/xurl-rs/issues")
            );
        }
    }

    #[test]
    fn a_refused_method_url_or_path_value_takes_the_commands_help() {
        for error in [
            Error::InvalidMethod("FETCH".into()),
            Error::invalid_url("ftp://example"),
            Error::InvalidPathParam {
                name: "id".into(),
                value: "1/2".into(),
            },
        ] {
            assert_eq!(
                step(&error),
                Some(NextStep::show_help("xr post --help".to_string())),
                "{error:?}"
            );
        }
    }

    #[test]
    fn a_credential_failure_gets_no_general_step() {
        assert_eq!(step(&Error::auth("token expired")), None);
        assert_eq!(step(&Error::api(403, "plain forbidden")), None);
        assert_eq!(step(&Error::api(429, "slow down")), None);
    }

    /// The confirming invocation is a template, quoted so it runs as
    /// printed, and it names the binary however it was started.
    #[test]
    fn confirm_is_the_invocation_with_force_and_never_a_command() {
        let invocation: Vec<String> = ["/usr/local/bin/xr", "auth", "apps", "remove", "my app"]
            .iter()
            .map(|arg| (*arg).to_string())
            .collect();
        let step = NextStep::confirm(&invocation);
        assert_eq!(step.action, NextAction::Confirm);
        assert!(
            step.command.is_none(),
            "running it destroys; the caller decides"
        );
        assert_eq!(
            step.template.as_deref(),
            Some("xr auth apps remove 'my app' --force")
        );
    }

    fn mismatch(
        supported: &[&str],
        requested: Option<&str>,
        available: &[&str],
        others: Option<&[&str]>,
    ) -> Error {
        let strings = |items: &[&str]| items.iter().map(ToString::to_string).collect::<Vec<_>>();
        Error::from(xdk::error::AuthMismatch {
            endpoint: "/2/users/me".into(),
            rendered_url: None,
            method: "GET".into(),
            requested: requested.map(str::to_string),
            supported: strings(supported),
            available_in_app: Some(strings(available)),
            app: Some("work".into()),
            other_apps_with_creds: others.map(strings),
        })
    }

    fn snapshot_of(store: &xdk::store::TokenStore) -> StoreSnapshot {
        StoreSnapshot::new(store, "work", false)
    }

    #[test]
    fn a_mismatch_the_store_can_answer_names_the_app_or_the_sign_in() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let mut store =
            xdk::store::TokenStore::new_with_path(&tmp.path().join("auth.yml").to_string_lossy());
        store
            .add_app("work", "CLIENT-ID", "SECRET")
            .expect("add app");
        let snapshot = snapshot_of(&store);
        let invocation: Vec<String> = ["xr", "whoami"].iter().map(|a| (*a).to_string()).collect();

        let elsewhere = mismatch(&["oauth2"], None, &[], Some(&["personal"]));
        let step = mismatch_hint(&snapshot, &elsewhere, &invocation, true).expect("a step");
        assert_eq!(
            step.next_step,
            NextStep::select_app("xr --app personal whoami".to_string())
        );

        let no_login = mismatch(&["oauth2", "oauth1"], None, &["app"], None);
        let step = mismatch_hint(&snapshot, &no_login, &invocation, true).expect("a step");
        assert_eq!(step.next_step, NextStep::sign_in(None, true));
    }

    #[test]
    fn a_mismatch_only_the_caller_can_answer_carries_no_step() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let store =
            xdk::store::TokenStore::new_with_path(&tmp.path().join("auth.yml").to_string_lossy());
        let snapshot = snapshot_of(&store);
        let invocation: Vec<String> = ["xr", "usage"].iter().map(|a| (*a).to_string()).collect();

        let named = mismatch(&["oauth2"], Some("app"), &["app"], None);
        assert!(mismatch_hint(&snapshot, &named, &invocation, true).is_none());
        let bearer_only = mismatch(&["app"], None, &["oauth2"], None);
        assert!(mismatch_hint(&snapshot, &bearer_only, &invocation, true).is_none());
        assert!(mismatch_hint(&snapshot, &Error::auth("x"), &invocation, true).is_none());
    }

    /// A rerun against another app names that app once, under the name
    /// every hint gives the binary: the `--app` the caller passed is
    /// dropped in either spelling, since clap refuses the flag twice.
    #[test]
    fn a_rerun_replaces_the_app_the_invocation_named() {
        use clap::Parser as _;
        let rerun = |args: &[&str]| {
            let invocation: Vec<String> = args.iter().map(|arg| (*arg).to_string()).collect();
            rerun_with_app(&invocation, "personal")
        };
        assert!(
            crate::cli::Cli::try_parse_from(["xr", "--app", "a", "--app", "b", "whoami"]).is_err(),
            "clap takes --app once"
        );
        assert_eq!(
            rerun(&["xr", "--app", "work", "whoami"]),
            "xr --app personal whoami"
        );
        assert_eq!(
            rerun(&["xr", "--output", "json", "whoami", "--app=work"]),
            "xr --app personal --output json whoami"
        );
        assert_eq!(
            rerun(&["/usr/local/bin/xr", "whoami"]),
            "xr --app personal whoami"
        );
        assert_eq!(
            rerun(&["xr", "post", "--", "--app", "is text"]),
            "xr --app personal post -- --app 'is text'"
        );
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
