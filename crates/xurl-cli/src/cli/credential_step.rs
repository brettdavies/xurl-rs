//! The step that gets a request the credential its endpoint accepts.
//!
//! One answer serves the message tail of an `auth-method-mismatch` and the
//! step of every credential refusal: it reads the schemes the endpoint
//! accepts, the schemes each app stores, and the scheme the caller named, so
//! a step never names a scheme the endpoint refuses or a setup command
//! without the flags it needs.

use xdk::api::auth_matrix::WireScheme;
use xdk::error::{
    Error, MismatchShape, NO_AUTH_METHOD, NO_BEARER_TOKEN, NO_OAUTH1_TOKEN, NO_OAUTH2_TOKEN,
};
use xdk::store::snapshot::StoreSnapshot;

use crate::cli::accepted_auth::{accepted_by_command, accepted_by_request};
use crate::cli::classify::{invoked_credential, requested_login};

use crate::cli::hints::{Hint, NextStep, choose_hint, quote_app_name, rerun_command};

/// The invocation that stores a credential of `scheme` in `app`, or the
/// default app when none is named, with each value only the caller has left
/// as an angle-bracket placeholder.
pub(crate) fn setup_invocation(scheme: WireScheme, app: Option<&str>) -> String {
    let app = app
        .filter(|name| !name.is_empty())
        .map(|name| format!(" --app {}", quote_app_name(name)))
        .unwrap_or_default();
    match scheme {
        WireScheme::OAuth2 => format!("xr auth oauth2{app}"),
        WireScheme::OAuth1 => format!(
            "xr auth oauth1{app} --consumer-key <consumer-key> --consumer-secret-file <path> \
             --access-token-file <path> --token-secret-file <path>"
        ),
        WireScheme::App => format!("xr auth app{app} --bearer-token-file <path>"),
    }
}

/// What a request found missing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Missing {
    /// No credential of any scheme the request could go out under.
    Anything,
    /// No stored credential of the one scheme the caller named.
    Scheme(WireScheme),
}

/// The missing credential `error` reports, when it reports one.
///
/// The messages are constants shared with their construction sites, so the
/// seam and the library agree on the exact strings.
pub(crate) fn missing_credential(error: &Error) -> Option<Missing> {
    let Error::Auth { message, .. } = error else {
        return None;
    };
    match message.as_str() {
        NO_AUTH_METHOD => Some(Missing::Anything),
        NO_OAUTH2_TOKEN => Some(Missing::Scheme(WireScheme::OAuth2)),
        NO_OAUTH1_TOKEN => Some(Missing::Scheme(WireScheme::OAuth1)),
        NO_BEARER_TOKEN => Some(Missing::Scheme(WireScheme::App)),
        _ => None,
    }
}

/// The hint for a request that found no credential.
///
/// A request whose command has a row takes the schemes that row accepts,
/// since the error names no endpoint. A raw request takes the schemes the
/// auth matrix lists for its method and path, and accepts every scheme when
/// the matrix does not list them or the invocation names no URL; a command
/// with no row falls back to the sign-in chooser.
#[must_use]
pub(crate) fn no_credential_hint(
    missing: Missing,
    snapshot: &StoreSnapshot,
    invocation: &[String],
    headless: bool,
) -> Hint {
    let Some(invoked) = invoked_credential(invocation) else {
        return choose_hint(snapshot, invocation, headless);
    };
    let accepted = if invoked.command.is_empty() {
        match &invoked.request {
            Some(request) => accepted_by_request(&request.method, &request.path),
            None => WireScheme::ALL_BY_PREFERENCE.to_vec(),
        }
    } else {
        match accepted_by_command(&invoked.command, invoked.of) {
            Some(accepted) => accepted,
            None => return choose_hint(snapshot, invocation, headless),
        }
    };
    let requested = match missing {
        Missing::Scheme(scheme) => Some(scheme),
        Missing::Anything => invoked.auth.as_deref().and_then(WireScheme::from_wire),
    };
    credential_hint(
        snapshot,
        &Need {
            accepted: &accepted,
            requested,
        },
        invocation,
        headless,
    )
}

/// The hint for a scheme mismatch the store can answer, from the schemes
/// the endpoint accepts and the scheme the caller named; `None` when the
/// error carries no app context.
#[must_use]
pub(crate) fn mismatch_hint(
    snapshot: &StoreSnapshot,
    error: &Error,
    invocation: &[String],
    headless: bool,
) -> Option<Hint> {
    let Error::AuthMethodMismatch(mismatch) = error else {
        return None;
    };
    if matches!(mismatch.shape(), MismatchShape::Unknown) {
        return None;
    }
    let refused_here: Vec<WireScheme> = mismatch
        .supported
        .iter()
        .filter_map(|scheme| WireScheme::from_wire(scheme))
        .collect();
    let accepted = serving_the_command(refused_here, invocation);
    let requested = mismatch
        .requested
        .as_deref()
        .and_then(WireScheme::from_wire);
    Some(credential_hint(
        snapshot,
        &Need {
            accepted: &accepted,
            requested,
        },
        invocation,
        headless,
    ))
}

/// The schemes of `at_the_failing_endpoint` that the whole command accepts.
///
/// A command may send a request to an endpoint that takes more schemes than
/// a later one does, so a step that names a scheme the failing endpoint takes
/// can send the caller to a refusal at the next request. The command's row
/// holds what every one of its requests accepts. A command with no row, or
/// whose row shares no scheme with the failing endpoint, keeps the endpoint's
/// own set.
fn serving_the_command(
    at_the_failing_endpoint: Vec<WireScheme>,
    invocation: &[String],
) -> Vec<WireScheme> {
    let Some(whole) = invoked_credential(invocation)
        .and_then(|invoked| accepted_by_command(&invoked.command, invoked.of))
    else {
        return at_the_failing_endpoint;
    };
    let shared: Vec<WireScheme> = at_the_failing_endpoint
        .iter()
        .copied()
        .filter(|scheme| whole.contains(scheme))
        .collect();
    if shared.is_empty() {
        at_the_failing_endpoint
    } else {
        shared
    }
}

/// What a credential refusal needs from the store: the schemes the failing
/// request's endpoint accepts and the scheme the caller named.
pub(crate) struct Need<'a> {
    pub(crate) accepted: &'a [WireScheme],
    pub(crate) requested: Option<WireScheme>,
}

/// The hint for a request that no stored credential serves.
///
/// ```text
/// load failed                              -> inspect-store, naming the path
/// the app holds an accepted scheme         -> fix-input, rerunning with --auth set to it
/// another app holds an accepted scheme     -> select-app, rerunning against that app
/// the scheme to get is OAuth2              -> the step that gets a login: see choose_hint
/// the app is named and not registered      -> register-app
/// otherwise                                -> fix-input, the template that stores the scheme to get
/// ```
///
/// A named login (`-u`) is served only by OAuth2 held by an app that stores
/// it, or by a bearer token on an endpoint that accepts no OAuth2. A caller
/// who named OAuth1 or the bearer token with `--auth` has not asked for a
/// login, and every accepted scheme serves.
#[must_use]
pub(crate) fn credential_hint(
    snapshot: &StoreSnapshot,
    need: &Need<'_>,
    invocation: &[String],
    headless: bool,
) -> Hint {
    if snapshot.load_failed() {
        return choose_hint(snapshot, invocation, headless);
    }
    let target = snapshot.active_app.as_str();
    let login = requested_login(invocation);
    // The request reads `-u` only under OAuth2, so a caller who named another
    // scheme has not asked for a login, as long as the rerun keeps that scheme.
    let names_another_scheme = need
        .requested
        .is_some_and(|scheme| scheme != WireScheme::OAuth2);
    let serves = |app: &str| -> Option<WireScheme> {
        snapshot.schemes_held(app).into_iter().find(|scheme| {
            need.accepted.contains(scheme)
                && login.as_ref().is_none_or(|login| match scheme {
                    WireScheme::OAuth2 => snapshot
                        .apps_holding_login(login)
                        .iter()
                        .any(|holder| holder == app),
                    _ if names_another_scheme => true,
                    // A bearer token acts as no account, so it stands in for
                    // a login only where the endpoint offers none to pick.
                    WireScheme::App => !need.accepted.contains(&WireScheme::OAuth2),
                    WireScheme::OAuth1 => false,
                })
        })
    };

    if let Some(scheme) = serves(target) {
        let rerun = rerun_command(invocation, None, Some(scheme));
        return Hint {
            text_lines: vec![format!(
                "App {target:?} holds {} credentials this endpoint accepts. Run: {}",
                scheme.pretty(),
                rerun.line
            )],
            next_step: rerun.fix_input(),
        };
    }
    let other = snapshot
        .apps
        .keys()
        .filter(|name| name.as_str() != target)
        .find_map(|name| serves(name).map(|scheme| (name, scheme)));
    if let Some((app, scheme)) = other {
        let rerun = rerun_command(
            invocation,
            Some(app),
            Some(scheme).filter(|_| need.requested.is_some()),
        );
        let command = &rerun.line;
        let line = match scheme {
            WireScheme::OAuth2 => format!("App {app:?} is already signed in. Run: {command}"),
            _ => format!(
                "App {app:?} holds {} credentials this endpoint accepts. Run: {command}",
                scheme.pretty()
            ),
        };
        return Hint {
            text_lines: vec![line],
            next_step: rerun.select_app(),
        };
    }
    let wanted = need
        .requested
        .filter(|scheme| need.accepted.contains(scheme))
        .or_else(|| need.accepted.first().copied());
    let Some(wanted) = wanted else {
        return choose_hint(snapshot, invocation, headless);
    };
    if wanted == WireScheme::OAuth2 {
        return choose_hint(snapshot, invocation, headless);
    }
    let registered = snapshot.apps.contains_key(target);
    if !registered && target != snapshot.default_app {
        let next_step = NextStep::register_app();
        let template = next_step.template.clone().unwrap_or_default();
        return Hint {
            text_lines: vec![format!(
                "App {target:?} is not registered. Register it first. Run: {template}"
            )],
            next_step,
        };
    }
    let template = setup_invocation(wanted, registered.then_some(target));
    Hint {
        text_lines: vec![format!(
            "This endpoint accepts {}. Add the credential first. Run: {template}",
            listed(need.accepted)
        )],
        next_step: NextStep::fix_input_template(template),
    }
}

/// The schemes as the messages name them: `OAuth 2.0, OAuth 1.0a`.
fn listed(schemes: &[WireScheme]) -> String {
    schemes
        .iter()
        .map(|scheme| scheme.pretty())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use xdk::error::NextAction;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(ToString::to_string).collect()
    }

    fn mismatch(
        supported: &[&str],
        requested: Option<&str>,
        available: &[&str],
        others: Option<&[&str]>,
    ) -> Error {
        Error::from(xdk::error::AuthMismatch {
            endpoint: "/2/usage/tweets".into(),
            rendered_url: None,
            method: "GET".into(),
            requested: requested.map(str::to_string),
            supported: strings(supported),
            available_in_app: Some(strings(available)),
            app: Some("work".into()),
            other_apps_with_creds: others.map(strings),
        })
    }

    /// The apps `work` (the default) and `personal`, each registered with a
    /// client id and holding what `held` names.
    fn snapshot(
        tmp: &tempfile::TempDir,
        held: &[(&str, &[WireScheme])],
        active: &str,
    ) -> StoreSnapshot {
        let mut store =
            xdk::store::TokenStore::new_with_path(&tmp.path().join("auth.yml").to_string_lossy());
        for app in ["work", "personal"] {
            store.add_app(app, "CLIENT-ID", "SECRET").expect("add app");
        }
        store.set_default_app("work").expect("default");
        for (app, schemes) in held {
            for scheme in *schemes {
                match scheme {
                    WireScheme::OAuth2 => store
                        .save_oauth2_token_for_app(app, "alice", "AT", "RT", u64::MAX)
                        .expect("save oauth2"),
                    WireScheme::OAuth1 => store
                        .save_oauth1_tokens_for_app(app, "AT", "TS", "CK", "CS")
                        .expect("save oauth1"),
                    WireScheme::App => store
                        .save_bearer_token_for_app(app, "BEARER")
                        .expect("save bearer"),
                }
            }
        }
        StoreSnapshot::new(&store, active, false)
    }

    fn words(args: &[&str]) -> Vec<String> {
        strings(args)
    }

    fn step_of(hint: Option<Hint>) -> (NextAction, Option<String>, Option<String>) {
        let step = hint.expect("a step").next_step;
        (step.action, step.command, step.template)
    }

    #[test]
    fn an_app_that_holds_an_accepted_scheme_reruns_with_that_scheme() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let snapshot = snapshot(&tmp, &[("work", &[WireScheme::App])], "work");
        let named = mismatch(&["app"], Some("oauth2"), &[], None);

        let step = step_of(mismatch_hint(
            &snapshot,
            &named,
            &words(&["xr", "usage", "--auth", "oauth2"]),
            true,
        ));

        assert_eq!(
            step,
            (
                NextAction::FixInput,
                Some("xr usage --auth app".into()),
                None
            )
        );
    }

    #[test]
    fn another_app_that_holds_an_accepted_scheme_is_selected() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let snapshot = snapshot(&tmp, &[("personal", &[WireScheme::App])], "work");
        let wrong_app = mismatch(&["app"], None, &[], Some(&["personal"]));

        let step = step_of(mismatch_hint(
            &snapshot,
            &wrong_app,
            &words(&["xr", "usage"]),
            true,
        ));

        assert_eq!(
            step,
            (
                NextAction::SelectApp,
                Some("xr --app personal usage".into()),
                None
            )
        );
    }

    /// An app that holds only a scheme the endpoint refuses is not offered.
    #[test]
    fn an_app_that_holds_only_a_refused_scheme_is_not_selected() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let snapshot = snapshot(&tmp, &[("personal", &[WireScheme::OAuth2])], "work");
        let wrong_app = mismatch(&["app"], None, &[], Some(&["personal"]));

        let step = step_of(mismatch_hint(
            &snapshot,
            &wrong_app,
            &words(&["xr", "usage"]),
            true,
        ));

        assert_eq!(
            step,
            (
                NextAction::FixInput,
                None,
                Some("xr auth app --app work --bearer-token-file <path>".into())
            )
        );
    }

    #[test]
    fn an_endpoint_that_takes_only_one_scheme_gets_the_command_that_stores_it() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let snapshot = snapshot(&tmp, &[("work", &[WireScheme::OAuth2])], "work");

        let app_only = mismatch(&["app"], None, &["oauth2"], None);
        assert_eq!(
            step_of(mismatch_hint(
                &snapshot,
                &app_only,
                &words(&["xr", "usage"]),
                true
            ))
            .2,
            Some("xr auth app --app work --bearer-token-file <path>".into())
        );
        let oauth1_only = mismatch(&["oauth1"], None, &["oauth2"], None);
        assert_eq!(
            step_of(mismatch_hint(
                &snapshot,
                &oauth1_only,
                &words(&["xr", "usage"]),
                true
            ))
            .2,
            Some(
                "xr auth oauth1 --app work --consumer-key <consumer-key> --consumer-secret-file <path> \
                 --access-token-file <path> --token-secret-file <path>"
                    .into()
            )
        );
    }

    #[test]
    fn an_endpoint_that_takes_a_login_the_app_lacks_signs_in() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let snapshot = snapshot(&tmp, &[("work", &[WireScheme::App])], "work");
        let no_login = mismatch(&["oauth2", "oauth1"], None, &["app"], None);

        let hint =
            mismatch_hint(&snapshot, &no_login, &words(&["xr", "whoami"]), true).expect("a step");

        assert_eq!(hint.next_step, NextStep::sign_in(None, true));
    }

    #[test]
    fn an_app_named_and_not_registered_is_registered_first() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let snapshot = snapshot(&tmp, &[], "ghost");
        let app_only = mismatch(&["app"], None, &[], None);

        let hint = mismatch_hint(
            &snapshot,
            &app_only,
            &words(&["xr", "--app", "ghost", "usage"]),
            true,
        )
        .expect("a step");

        assert_eq!(hint.next_step, NextStep::register_app());
    }

    #[test]
    fn a_command_that_takes_only_the_app_credential_needs_no_login() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let snapshot = snapshot(&tmp, &[], "work");

        let hint = no_credential_hint(
            Missing::Anything,
            &snapshot,
            &words(&["xr", "webhooks", "subscriptions", "remove", "123", "@jack"]),
            true,
        );

        assert_eq!(
            hint.next_step,
            NextStep::fix_input_template(
                "xr auth app --app work --bearer-token-file <path>".into()
            )
        );
    }

    #[test]
    fn a_request_the_matrix_does_not_know_takes_the_sign_in_chooser() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let snapshot = snapshot(&tmp, &[], "work");

        let hint = no_credential_hint(
            Missing::Anything,
            &snapshot,
            &words(&["xr", "/2/users/me"]),
            true,
        );

        assert_eq!(
            hint,
            choose_hint(&snapshot, &words(&["xr", "/2/users/me"]), true)
        );
    }

    #[test]
    fn a_named_scheme_with_nothing_stored_gets_the_command_that_stores_it() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let snapshot = snapshot(&tmp, &[], "work");
        let invocation = words(&["xr", "whoami", "--auth", "oauth1"]);

        let hint = no_credential_hint(
            Missing::Scheme(WireScheme::OAuth1),
            &snapshot,
            &invocation,
            true,
        );

        assert_eq!(
            hint.next_step,
            NextStep::fix_input_template(setup_invocation(WireScheme::OAuth1, Some("work")))
        );
    }

    /// `-u` selects a login, which only `OAuth2` stores, so no other scheme
    /// the app holds can stand in for it.
    #[test]
    fn a_login_the_caller_named_is_never_answered_with_another_scheme() {
        for held in [WireScheme::OAuth1, WireScheme::App] {
            let tmp = tempfile::TempDir::new().expect("tempdir");
            let snapshot = snapshot(&tmp, &[("work", &[held])], "work");
            let invocation = words(&["xr", "post", "hello", "--auth", "oauth2", "-u", "alice"]);

            let hint = no_credential_hint(
                Missing::Scheme(WireScheme::OAuth2),
                &snapshot,
                &invocation,
                true,
            );

            let step = hint.next_step;
            assert_eq!(step.action, NextAction::SignIn, "{held:?}: {step:?}");
            let named = format!("{:?}{:?}", step.command, step.template);
            assert!(!named.contains("--auth oauth1"), "{held:?}: {named}");
            assert!(!named.contains("--auth app"), "{held:?}: {named}");
        }
    }

    /// A bearer token acts as no account, so on an endpoint with no OAuth2
    /// login to pick it serves a request that names one, and the step is the
    /// rerun under it, not a template that stores the token again.
    #[test]
    fn a_bearer_serves_a_named_login_where_the_endpoint_takes_no_oauth2() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let snapshot = snapshot(&tmp, &[("work", &[WireScheme::App])], "work");

        let named = mismatch(&["app"], Some("oauth2"), &[], None);
        let step = step_of(mismatch_hint(
            &snapshot,
            &named,
            &words(&["xr", "usage", "--auth", "oauth2", "-u", "alice"]),
            true,
        ));
        assert_eq!(
            step,
            (
                NextAction::FixInput,
                Some("xr usage --auth app -u alice".into()),
                None
            )
        );

        let raw = no_credential_hint(
            Missing::Anything,
            &snapshot,
            &words(&["xr", "-u", "alice", "/2/usage/tweets"]),
            true,
        );
        assert_eq!(raw.next_step.action, NextAction::FixInput, "{raw:?}");
        assert_eq!(
            raw.next_step.command.as_deref(),
            Some("xr -u alice /2/usage/tweets --auth app"),
            "{raw:?}"
        );
    }

    /// The request reads `-u` only under OAuth2, so a caller who named
    /// OAuth1 or the bearer token is served by another app that holds it.
    #[test]
    fn a_scheme_named_with_auth_outranks_a_login() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let snapshot = snapshot(&tmp, &[("personal", &[WireScheme::OAuth1])], "work");
        let invocation = words(&["xr", "post", "hi", "--auth", "oauth1", "-u", "alice"]);

        let hint = no_credential_hint(
            Missing::Scheme(WireScheme::OAuth1),
            &snapshot,
            &invocation,
            true,
        );

        assert_eq!(hint.next_step.action, NextAction::SelectApp, "{hint:?}");
        assert_eq!(
            hint.next_step.command.as_deref(),
            Some("xr --app personal post hi --auth oauth1 -u alice"),
            "{hint:?}"
        );
    }

    /// A raw request to a path that carries an id is held to the schemes of
    /// the template it fits.
    #[test]
    fn a_raw_request_with_an_id_is_held_to_its_templates_schemes() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let snapshot = snapshot(&tmp, &[("work", &[WireScheme::App])], "work");

        for invocation in [
            words(&["xr", "-X", "DELETE", "/2/tweets/123", "--auth", "oauth2"]),
            words(&["xr", "-X", "POST", "/2/users/123/likes", "--auth", "oauth1"]),
        ] {
            let hint = no_credential_hint(
                Missing::Scheme(WireScheme::OAuth2),
                &snapshot,
                &invocation,
                true,
            );
            let step = hint.next_step;
            let named = format!("{:?}{:?}", step.command, step.template);
            assert!(!named.contains("--auth app"), "{invocation:?}: {step:?}");
            assert_ne!(
                step.action,
                NextAction::FixInput,
                "{invocation:?}: {step:?}"
            );
        }
    }

    /// A raw request is held to what the matrix lists for its method and
    /// path: `GET /2/users/me` takes no bearer token.
    #[test]
    fn a_raw_request_is_not_sent_to_a_scheme_its_endpoint_refuses() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let snapshot = snapshot(&tmp, &[("work", &[WireScheme::App])], "work");
        let invocation = words(&["xr", "/2/users/me", "--auth", "oauth2"]);

        let hint = no_credential_hint(
            Missing::Scheme(WireScheme::OAuth2),
            &snapshot,
            &invocation,
            true,
        );

        assert_eq!(hint.next_step.action, NextAction::SignIn, "{hint:?}");
        let held_by_another_endpoint = words(&["xr", "-X", "GET", "/2/usage/tweets"]);
        let hint = no_credential_hint(
            Missing::Anything,
            &snapshot,
            &held_by_another_endpoint,
            true,
        );
        assert_eq!(hint.next_step.action, NextAction::FixInput, "{hint:?}");
        assert_eq!(
            hint.next_step.command.as_deref(),
            Some("xr -X GET /2/usage/tweets --auth app")
        );
    }

    /// A header often carries a credential, so a step that reruns the
    /// invocation names the header and leaves its value to the caller.
    #[test]
    fn a_step_that_reruns_the_invocation_leaves_header_values_out() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let snapshot = snapshot(&tmp, &[("work", &[WireScheme::App])], "work");
        let invocation = words(&[
            "xr",
            "--auth",
            "oauth1",
            "-H",
            "X-Api-Key: SEKRET",
            "/2/usage/tweets",
        ]);

        let hint = no_credential_hint(
            Missing::Scheme(WireScheme::OAuth1),
            &snapshot,
            &invocation,
            true,
        );

        let shown = format!("{hint:?}");
        assert!(!shown.contains("SEKRET"), "{shown}");
        assert_eq!(hint.next_step.command, None, "{shown}");
        assert_eq!(
            hint.next_step.template.as_deref(),
            Some("xr --auth app -H 'X-Api-Key: <value>' /2/usage/tweets"),
            "{shown}"
        );
    }

    #[test]
    fn a_named_scheme_the_app_lacks_reruns_with_one_it_holds() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let snapshot = snapshot(&tmp, &[("work", &[WireScheme::OAuth1])], "work");
        let invocation = words(&["xr", "whoami", "--auth", "oauth2"]);

        let step = step_of(Some(no_credential_hint(
            Missing::Scheme(WireScheme::OAuth2),
            &snapshot,
            &invocation,
            true,
        )));

        assert_eq!(
            step,
            (
                NextAction::FixInput,
                Some("xr whoami --auth oauth1".into()),
                None
            )
        );
    }

    #[test]
    fn the_missing_credential_is_read_from_the_library_constants() {
        let missing = |message: &str| missing_credential(&Error::auth(message));

        assert_eq!(missing(NO_AUTH_METHOD), Some(Missing::Anything));
        assert_eq!(
            missing(NO_OAUTH1_TOKEN),
            Some(Missing::Scheme(WireScheme::OAuth1))
        );
        assert_eq!(
            missing(NO_BEARER_TOKEN),
            Some(Missing::Scheme(WireScheme::App))
        );
        assert_eq!(
            missing(NO_OAUTH2_TOKEN),
            Some(Missing::Scheme(WireScheme::OAuth2))
        );
        assert_eq!(missing("something else"), None);
    }

    #[test]
    fn a_mismatch_without_an_app_context_carries_no_step() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let snapshot = snapshot(&tmp, &[], "work");
        let unknown = Error::from(xdk::error::AuthMismatch {
            endpoint: "/2/usage/tweets".into(),
            rendered_url: None,
            method: "GET".into(),
            requested: None,
            supported: strings(&["app"]),
            available_in_app: None,
            app: None,
            other_apps_with_creds: None,
        });
        let invocation = words(&["xr", "usage"]);

        assert!(mismatch_hint(&snapshot, &unknown, &invocation, true).is_none());
        assert!(mismatch_hint(&snapshot, &Error::auth("x"), &invocation, true).is_none());
    }
}
