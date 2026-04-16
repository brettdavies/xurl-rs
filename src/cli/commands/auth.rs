/// Auth subcommand handlers — OAuth2, OAuth1, Bearer, app management.
use crate::auth::Auth;
use crate::cli::{AppCommands, AuthCommands};
use crate::error::{Result, XurlError};
use crate::output::OutputConfig;
use crate::store::TokenStore;

#[allow(clippy::too_many_lines)]
pub(super) fn run_auth_command(
    cmd: AuthCommands,
    mut auth: Auth,
    no_interactive: bool,
    out: &OutputConfig,
) -> Result<()> {
    match cmd {
        AuthCommands::Oauth2 {
            no_browser,
            step,
            auth_url,
        } => {
            if !no_browser {
                // Standard interactive flow
                auth.oauth2_flow("")?;
                out.print_message("\x1b[32mOAuth2 authentication successful!\x1b[0m");
            } else {
                let pending_path = crate::auth::pending::default_pending_path()?;
                match step {
                    Some(1) => {
                        if auth_url.is_some() {
                            return Err(crate::error::XurlError::auth(
                                "--auth-url is only used with --step 2, not --step 1",
                            ));
                        }
                        let url = auth.remote_oauth2_step1(&pending_path)?;
                        match out.format {
                            crate::output::OutputFormat::Json
                            | crate::output::OutputFormat::Jsonl => {
                                let json = serde_json::json!({
                                    "auth_url": url,
                                    "instructions": "Open the URL in a browser, authorize, then copy the redirect URL and run step 2"
                                });
                                println!("{json}");
                            }
                            crate::output::OutputFormat::Text => {
                                out.print_message(
                                    "Open this URL in a browser on a machine with a display:",
                                );
                                out.print_message("");
                                out.print_message(&format!("  {url}"));
                                out.print_message("");
                                out.print_message(
                                    "After authorizing, copy the redirect URL from your browser's address bar",
                                );
                                out.print_message(
                                    "(it will show an error page — that's expected).",
                                );
                                out.print_message("");
                                out.print_message("Then run:");
                                out.print_message(
                                    "  echo '<redirect-url>' | xr auth oauth2 --no-browser --step 2 --auth-url -",
                                );
                            }
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

                        auth.remote_oauth2_step2(&redirect_url, "", &pending_path)?;
                        out.print_message("\x1b[32mOAuth2 authentication successful!\x1b[0m");
                    }
                    None => {
                        return Err(crate::error::XurlError::auth(
                            "--no-browser requires --step 1 or --step 2",
                        ));
                    }
                    _ => unreachable!("clap value_parser restricts to 1..=2"),
                }
            }
        }
        AuthCommands::Oauth1 {
            consumer_key,
            consumer_secret,
            access_token,
            token_secret,
        } => {
            auth.token_store.save_oauth1_tokens(
                &access_token,
                &token_secret,
                &consumer_key,
                &consumer_secret,
            )?;
            out.print_message("\x1b[32mOAuth1 credentials saved successfully!\x1b[0m");
        }
        AuthCommands::App { bearer_token } => {
            auth.token_store.save_bearer_token(&bearer_token)?;
            out.print_message("\x1b[32mApp authentication successful!\x1b[0m");
        }
        AuthCommands::Status => {
            let ts = TokenStore::new();
            let apps = ts.list_apps();
            let default_app = ts.get_default_app();

            if apps.is_empty() {
                out.print_message("No apps registered. Use 'xr auth apps add' to register one.");
                return Ok(());
            }

            for (i, name) in apps.iter().enumerate() {
                if let Some(app) = ts.get_app(name) {
                    let marker = if name == default_app { "\u{25b8}" } else { " " };
                    let client_hint = if app.client_id.is_empty() {
                        "(no credentials)".to_string()
                    } else {
                        format!("client_id: {}...", truncate(&app.client_id, 8))
                    };
                    out.print_message(&format!("{marker} {name}  [{client_hint}]"));

                    let usernames = ts.get_oauth2_usernames_for_app(name);
                    if usernames.is_empty() {
                        out.print_message("      oauth2: (none)");
                    } else {
                        for u in &usernames {
                            if *u == app.default_user {
                                out.print_message(&format!("    \u{25b8} oauth2: {u}"));
                            } else {
                                out.print_message(&format!("      oauth2: {u}"));
                            }
                        }
                    }

                    if app.oauth1_token.is_some() {
                        out.print_message("      oauth1: \u{2713}");
                    } else {
                        out.print_message("      oauth1: \u{2013}");
                    }

                    if app.bearer_token.is_some() {
                        out.print_message("      bearer: \u{2713}");
                    } else {
                        out.print_message("      bearer: \u{2013}");
                    }

                    if i < apps.len() - 1 {
                        out.print_message("");
                    }
                }
            }
        }
        AuthCommands::Clear {
            all,
            oauth1,
            oauth2_username,
            bearer,
        } => {
            if all {
                auth.token_store.clear_all()?;
                out.print_message("All authentication cleared!");
            } else if oauth1 {
                auth.token_store.clear_oauth1_tokens()?;
                out.print_message("OAuth1 tokens cleared!");
            } else if let Some(username) = oauth2_username {
                auth.token_store.clear_oauth2_token(&username)?;
                out.print_message(&format!("OAuth2 token cleared for {username}!"));
            } else if bearer {
                auth.token_store.clear_bearer_token()?;
                out.print_message("Bearer token cleared!");
            } else {
                return Err(XurlError::validation(
                    "No authentication cleared! Use --all to clear all authentication.",
                ));
            }
        }
        AuthCommands::Apps { command } => {
            return run_app_command(command, &mut auth, out);
        }
        AuthCommands::Default { app_name, username } => {
            if let Some(app_name) = app_name {
                auth.token_store.set_default_app(&app_name)?;
                out.print_message(&format!("\x1b[32mDefault app set to {app_name:?}\x1b[0m"));
                if let Some(user) = username {
                    auth.token_store.set_default_user(&app_name, &user)?;
                    out.print_message(&format!("\x1b[32mDefault user set to {user:?}\x1b[0m"));
                }
            } else {
                // Interactive picker
                if no_interactive {
                    return Err(XurlError::auth(
                        "Interactive prompt required. Pass app name as argument: xr auth default <app-name>",
                    ));
                }

                let apps = auth.token_store.list_apps();
                if apps.is_empty() {
                    out.print_message(
                        "No apps registered. Use 'xr auth apps add' to register one.",
                    );
                    return Ok(());
                }

                let app_choice = match dialoguer::Select::new()
                    .with_prompt("Select default app")
                    .items(&apps)
                    .interact_opt()
                {
                    Ok(Some(idx)) => apps[idx].clone(),
                    Ok(None) => return Ok(()),
                    Err(e) => {
                        return Err(XurlError::validation(format!("Selection error: {e}")));
                    }
                };

                auth.token_store.set_default_app(&app_choice)?;
                out.print_message(&format!("\x1b[32mDefault app set to {app_choice:?}\x1b[0m"));

                let users = auth.token_store.get_oauth2_usernames_for_app(&app_choice);
                if !users.is_empty()
                    && let Ok(Some(idx)) = dialoguer::Select::new()
                        .with_prompt("Select default OAuth2 user")
                        .items(&users)
                        .interact_opt()
                {
                    let user = &users[idx];
                    auth.token_store.set_default_user(&app_choice, user)?;
                    out.print_message(&format!("\x1b[32mDefault user set to {user:?}\x1b[0m"));
                }
            }
        }
    }
    Ok(())
}

fn run_app_command(cmd: AppCommands, auth: &mut Auth, out: &OutputConfig) -> Result<()> {
    match cmd {
        AppCommands::Add {
            name,
            client_id,
            client_secret,
        } => {
            auth.token_store
                .add_app(&name, &client_id, &client_secret)?;
            out.print_message(&format!("\x1b[32mApp {name:?} registered!\x1b[0m"));
            if auth.token_store.list_apps().len() == 1 {
                out.print_message("  (set as default app)");
            }
        }
        AppCommands::Update {
            name,
            client_id,
            client_secret,
        } => {
            if client_id.is_none() && client_secret.is_none() {
                return Err(XurlError::validation(
                    "Nothing to update. Provide --client-id and/or --client-secret.",
                ));
            }
            auth.token_store.update_app(
                &name,
                &client_id.unwrap_or_default(),
                &client_secret.unwrap_or_default(),
            )?;
            out.print_message(&format!("\x1b[32mApp {name:?} updated.\x1b[0m"));
        }
        AppCommands::Remove { name } => {
            auth.token_store.remove_app(&name)?;
            out.print_message(&format!("\x1b[32mApp {name:?} removed.\x1b[0m"));
        }
        AppCommands::List => {
            let ts = TokenStore::new();
            let apps = ts.list_apps();
            let default_app = ts.get_default_app();

            if apps.is_empty() {
                out.print_message("No apps registered. Use 'xr auth apps add' to register one.");
                return Ok(());
            }

            for name in &apps {
                if let Some(app) = ts.get_app(name) {
                    let marker = if name == default_app {
                        "\u{25b8} "
                    } else {
                        "  "
                    };
                    let client_hint = if app.client_id.is_empty() {
                        String::new()
                    } else {
                        format!(" (client_id: {}...)", truncate(&app.client_id, 8))
                    };
                    out.print_message(&format!("{marker}{name}{client_hint}"));
                }
            }
        }
    }
    Ok(())
}

/// Truncates a string to a maximum length.
fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        match s.char_indices().nth(max_len) {
            Some((byte_idx, _)) => &s[..byte_idx],
            None => s,
        }
    }
}
