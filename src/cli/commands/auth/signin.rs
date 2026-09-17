//! Sign-in flows: interactive and headless `OAuth2`, `OAuth1` credential save,
//! and bearer-token save.

use std::io::IsTerminal;

use serde_json::json;

use super::{AuthCtx, AuthGlobalFlags};
use crate::auth::Auth;
use crate::cli::envelope::ErrorBody;
use crate::cli::hints::NextStep;
use crate::error::{EXIT_USAGE_ERROR, Result, XurlError};

/// Arguments of `xr auth oauth2`: whether to suppress the browser, which
/// manual step to run, the redirect URL that step 2 exchanges, and the
/// username label for the saved token.
pub(super) struct Oauth2Args {
    pub(super) no_browser: bool,
    pub(super) step: Option<u8>,
    pub(super) auth_url: Option<String>,
    pub(super) username: Option<String>,
}

pub(super) fn oauth2(args: Oauth2Args, ctx: AuthCtx<'_>) -> Result<()> {
    let Oauth2Args {
        no_browser,
        step,
        auth_url,
        username,
    } = args;
    let AuthCtx {
        auth,
        flags,
        out,
        stdout,
        stderr,
    } = ctx;
    let AuthGlobalFlags {
        dry_run,
        app_explicit,
        ..
    } = flags;
    if dry_run {
        let ctx = json!({
            "command": "auth-oauth2",
            "no_browser": no_browser,
            "step": step,
            "username": username,
        });
        out.print_dry_run(stdout, true, 0, &ctx);
        return Ok(());
    }
    let username_arg = username.as_deref().unwrap_or("");
    // R15: refuse before any URL is built or pending file written when
    // the target app has no client id to sign in with. The old
    // credential-less warning is unreachable behind this guard.
    if let Some(body) = client_credentials_missing(auth, app_explicit, out.format.is_structured()) {
        out.emit_error_envelope(stderr, body);
        return Err(XurlError::EnvelopeAlreadyEmitted {
            exit_code: EXIT_USAGE_ERROR,
        });
    }
    // Headless auto-engage: stdout is not a TTY (piped run, CI, agent
    // harness) and the user did not pass `--no-browser` (or set
    // `XURL_NO_BROWSER`). Opening a browser the caller can't see would
    // silently strand the flow, so promote to the remote two-step
    // path and surface the auth URL on stdout.
    let auto_engage_no_browser = !no_browser && !std::io::stdout().is_terminal();
    let effective_no_browser = no_browser || auto_engage_no_browser;
    if !effective_no_browser {
        // Standard interactive flow. The opener records the URL it could not
        // open, so the paste-the-URL advice can name it once the flow returns.
        let unopened = std::sync::Arc::new(std::sync::Mutex::new(None::<String>));
        let opener = {
            let unopened = std::sync::Arc::clone(&unopened);
            move |url: &str| {
                let result = open::that(url);
                if result.is_err()
                    && let Ok(mut slot) = unopened.lock()
                {
                    *slot = Some(url.to_string());
                }
                result
            }
        };
        if let Err(e) = auth.oauth2_flow(username_arg, opener) {
            if let Some(url) = unopened.lock().ok().and_then(|mut slot| slot.take()) {
                out.print_message(
                    stdout,
                    "Failed to open browser automatically. Re-run with --no-browser to use the paste-the-URL flow:",
                );
                out.print_message(stdout, &url);
            }
            return Err(e);
        }
        out.print_ok_message(stdout, "\x1b[32mOAuth2 authentication successful!\x1b[0m");
    } else {
        let pending_path =
            crate::auth::pending::pending_path_for_store(&auth.token_store.file_path);
        // When the user opted into `--no-browser` without an explicit
        // `--step`, or when auto-engage promoted us here, run step 1
        // and emit the canonical `awaiting_callback` envelope.
        let effective_step = if step.is_none() && auth_url.is_none() {
            Some(1u8)
        } else {
            step
        };
        match effective_step {
            Some(1) => {
                if auth_url.is_some() {
                    return Err(crate::error::XurlError::auth(
                        "--auth-url is only used with --step 2, not --step 1",
                    ));
                }
                let url = auth.remote_oauth2_step1(&pending_path)?;
                if out.format.is_structured() {
                    // U9: explicit `--no-browser` (no `--step`) and
                    // the auto-engaged path both emit the canonical
                    // `awaiting_callback` envelope; the existing
                    // `--step 1` path keeps its legacy shape so
                    // agents that pinned against it don't drift.
                    if step.is_none() {
                        // The awaiting-callback envelope carries its
                        // own status: the flow is not finished, so
                        // reporting `ok` would be wrong.
                        out.print_response(
                            stdout,
                            &serde_json::json!({
                                "status": "awaiting_callback",
                                "url": url,
                                "instructions": "Open the URL in a browser, authorize, then run 'xr auth oauth2 --no-browser --step 2 --auth-url <redirect-url>'",
                            }),
                        );
                    } else {
                        out.print_success(
                            stdout,
                            &serde_json::json!({
                                "auth_url": url,
                                "instructions": "Open the URL in a browser, authorize, then copy the redirect URL and run step 2"
                            }),
                        );
                    }
                } else {
                    out.print_message(
                        stdout,
                        "Open this URL in a browser on a machine with a display:",
                    );
                    out.print_message(stdout, "");
                    out.print_message(stdout, &format!("  {url}"));
                    out.print_message(stdout, "");
                    out.print_message(
                        stdout,
                        "After authorizing, copy the redirect URL from your browser's address bar",
                    );
                    out.print_message(stdout, "(it will show an error page — that's expected).");
                    out.print_message(stdout, "");
                    out.print_message(stdout, "Then run:");
                    out.print_message(
                        stdout,
                        "  echo '<redirect-url>' | xr auth oauth2 --no-browser --step 2 --auth-url -",
                    );
                }
            }
            Some(2) => {
                let url_value = auth_url.ok_or_else(|| {
                    crate::error::XurlError::auth(
                        "--auth-url is required for step 2. Pass the redirect URL from your browser, \
                         or use --auth-url - to read from stdin",
                    )
                })?;

                let redirect_url = if url_value == "-" {
                    let mut line = String::new();
                    std::io::stdin().read_line(&mut line).map_err(|e| {
                        crate::error::XurlError::auth_with_cause(
                            "Failed to read redirect URL from stdin",
                            &e,
                        )
                    })?;
                    let trimmed = line.trim().to_string();
                    if trimmed.is_empty() {
                        return Err(crate::error::XurlError::auth(
                            "No redirect URL provided on stdin. \
                             Pipe the URL or paste it and press Enter",
                        ));
                    }
                    trimmed
                } else {
                    url_value
                };

                auth.remote_oauth2_step2(&redirect_url, username_arg, &pending_path)?;
                out.print_message(stdout, "\x1b[32mOAuth2 authentication successful!\x1b[0m");
            }
            None => {
                // Unreachable in practice: when `--step` is omitted and
                // `--auth-url` is also omitted, `effective_step` is set
                // to `Some(1)` above; when `--auth-url` is given without
                // `--step`, clap rejects it via `requires = "step"`.
                return Err(crate::error::XurlError::auth(
                    "--no-browser requires --step 1 or --step 2",
                ));
            }
            _ => unreachable!("clap value_parser restricts to 1..=2"),
        }
    }
    Ok(())
}

/// Arguments of `xr auth oauth1`: the consumer key/secret pair identifying the
/// app and the access token/secret pair identifying the user.
pub(super) struct Oauth1Args {
    pub(super) consumer_key: String,
    pub(super) consumer_secret: String,
    pub(super) access_token: String,
    pub(super) token_secret: String,
}

pub(super) fn oauth1(args: Oauth1Args, ctx: AuthCtx<'_>) -> Result<()> {
    let Oauth1Args {
        consumer_key,
        consumer_secret,
        access_token,
        token_secret,
    } = args;
    let AuthCtx {
        auth,
        flags,
        out,
        stdout,
        ..
    } = ctx;
    let AuthGlobalFlags { dry_run, .. } = flags;
    if dry_run {
        let ctx = json!({"command": "auth-oauth1"});
        out.print_dry_run(stdout, true, 0, &ctx);
        return Ok(());
    }
    // Multi-app save: route OAuth1 tokens to the active app
    // (set by `--app NAME` or default). The no-arg variant fell
    // back to the default app even with `--app NAME` set, so a
    // user running `xr auth oauth1 --app NAME …` would silently
    // overwrite default's OAuth1 instead of populating NAME.
    let candidate = auth.app_name().to_string();
    auth.token_store.save_oauth1_tokens_for_app(
        &candidate,
        &access_token,
        &token_secret,
        &consumer_key,
        &consumer_secret,
    )?;
    // Auto-default the first signed-in app so the user does not
    // need an explicit `xr auth default <name>` follow-up. Clone
    // the app name first; `promote_...` takes `&mut self` on the
    // store which would otherwise alias the immutable `&str`
    // borrow from `auth.app_name()`.
    let _ = auth
        .token_store
        .promote_to_default_if_first_credentialed(&candidate)?;
    out.print_message(
        stdout,
        "\x1b[32mOAuth1 credentials saved successfully!\x1b[0m",
    );
    Ok(())
}

pub(super) fn bearer(bearer_token: String, ctx: AuthCtx<'_>) -> Result<()> {
    let AuthCtx {
        auth,
        flags,
        out,
        stdout,
        ..
    } = ctx;
    let AuthGlobalFlags { dry_run, .. } = flags;
    if dry_run {
        let ctx = json!({"command": "auth-app"});
        out.print_dry_run(stdout, true, 0, &ctx);
        return Ok(());
    }
    // Multi-app save: route the bearer to the active app
    // (set by `--app NAME` or default). The no-arg variant
    // silently fell back to the default app even with `--app
    // NAME` set, so `xr auth app --bearer-token … --app NAME`
    // would overwrite default's bearer instead of populating
    // NAME's entry.
    let candidate = auth.app_name().to_string();
    auth.token_store
        .save_bearer_token_for_app(&candidate, &bearer_token)?;
    let _ = auth
        .token_store
        .promote_to_default_if_first_credentialed(&candidate)?;
    out.print_ok_message(stdout, "\x1b[32mApp authentication successful!\x1b[0m");
    Ok(())
}

/// Builds the refusal body when the target app has no client id to sign in
/// with, or `None` when sign-in may proceed.
///
/// The effective client id comes from the [`Auth`] accessor, so `CLIENT_ID`
/// exported in the environment counts regardless of how the app was
/// selected. When another app does carry credentials the hint names it;
/// otherwise the hint is registration, whose values only the caller has.
fn client_credentials_missing(
    auth: &Auth,
    app_explicit: bool,
    structured: bool,
) -> Option<ErrorBody> {
    if !auth.client_id().is_empty() {
        return None;
    }
    let target = auth.token_store.get_active_app_name(auth.app_name());
    let credentialed: Vec<String> = auth
        .token_store
        .list_apps()
        .into_iter()
        .filter(|name| name != target)
        .filter(|name| {
            auth.token_store
                .get_app(name)
                .is_some_and(|app| !app.client_id.is_empty())
        })
        .collect();

    let (prose, next_step) = match credentialed.first() {
        Some(alternative) => (
            format!("app {target:?} has no client credentials; app {alternative:?} does."),
            NextStep::select_app(
                NextStep::sign_in(Some(alternative), structured)
                    .command
                    .unwrap_or_default(),
            ),
        ),
        None => (
            if app_explicit {
                format!("app {target:?} has no client credentials.")
            } else {
                "no app carries client credentials.".to_string()
            },
            NextStep::register_app(),
        ),
    };
    // KTD12: the text derives from the built value, so the prose and the
    // machine-readable step cannot name different commands.
    let message = match next_step.display_invocation() {
        Some(invocation) => format!("{prose} Run: {invocation}"),
        None => prose,
    };

    Some(ErrorBody {
        reason: "client-credentials-missing".to_string(),
        exit_code: EXIT_USAGE_ERROR,
        message: Some(message),
        app: (!target.is_empty()).then(|| target.to_string()),
        next_step: Some(next_step),
        ..ErrorBody::default()
    })
}
