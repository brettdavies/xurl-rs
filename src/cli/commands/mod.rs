/// Command execution — dispatches CLI commands to API functions.
use crate::api::{self, ApiClient, RequestOptions};
use crate::auth::Auth;
use crate::cli::{AppCommands, AuthCommands, Cli, Commands, MediaCommands};
use crate::config::Config;
use crate::error::{Result, XurlError};
use crate::output::OutputConfig;
use crate::store::TokenStore;

/// Runs the CLI — dispatches to the appropriate handler.
///
/// # Errors
///
/// Returns an error if the command fails.
pub fn run(cli: Cli, out: &OutputConfig) -> Result<()> {
    let cfg = Config::new();
    let mut auth = Auth::new(&cfg);

    // Apply --app override
    if let Some(ref app_name) = cli.app {
        auth.with_app_name(app_name);
    }

    let no_interactive = cli.no_interactive;
    match cli.command {
        Some(cmd) => run_subcommand(cmd, &cfg, &mut auth, no_interactive, out),
        None => run_raw_mode(&cli, &cfg, &mut auth, out),
    }
}

/// Runs raw curl-style mode.
fn run_raw_mode(cli: &Cli, cfg: &Config, auth: &mut Auth, out: &OutputConfig) -> Result<()> {
    let url = if let Some(u) = &cli.url {
        u.clone()
    } else {
        return Err(XurlError::Api(
            "No URL provided. Usage: xr [OPTIONS] [URL] [COMMAND]. Try 'xr --help' for more information.".to_string(),
        ));
    };

    let method = cli.method.clone().unwrap_or_else(|| "GET".to_string());
    let media_file = cli.file.clone().unwrap_or_default();

    let mut client = ApiClient::new(cfg, auth);

    let options = RequestOptions {
        method,
        endpoint: url.clone(),
        headers: cli.headers.clone(),
        data: cli.data.clone().unwrap_or_default(),
        auth_type: cli.auth_type.clone().unwrap_or_default(),
        username: cli.username.clone().unwrap_or_default(),
        verbose: cli.verbose,
        trace: cli.trace,
    };

    // Check for media append request
    if api::is_media_append_request(&options.endpoint, &media_file) {
        let response = api::handle_media_append_request(&options, &media_file, &mut client)?;
        out.print_response(&response);
        return Ok(());
    }

    let should_stream = cli.stream || api::is_streaming_endpoint(&options.endpoint);

    if should_stream {
        stream_request_with_output(&mut client, &options, out)
    } else {
        let response = client.send_request(&options)?;
        out.print_response(&response);
        Ok(())
    }
}

/// Sends a streaming request with output-format awareness.
fn stream_request_with_output(
    client: &mut ApiClient,
    options: &RequestOptions,
    out: &OutputConfig,
) -> Result<()> {
    use std::io::{BufRead, BufReader};

    let method = options.method.to_uppercase();
    let method = if method.is_empty() { "GET" } else { &method };
    let url = client.build_url_public(&options.endpoint);

    let req_method = reqwest::Method::from_bytes(method.as_bytes())
        .map_err(|_| XurlError::InvalidMethod(method.to_string()))?;

    let mut builder = reqwest::blocking::Client::builder()
        .timeout(None)
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
        .request(req_method, &url);

    if !options.data.is_empty() {
        if serde_json::from_str::<serde_json::Value>(&options.data).is_ok() {
            builder = builder
                .header("Content-Type", "application/json")
                .body(options.data.clone());
        } else {
            builder = builder
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(options.data.clone());
        }
    }

    for header in &options.headers {
        if let Some((key, value)) = header.split_once(':') {
            builder = builder.header(key.trim(), value.trim());
        }
    }

    if let Ok(auth_header) =
        client.get_auth_header_public(method, &url, &options.auth_type, &options.username)
    {
        builder = builder.header("Authorization", auth_header);
    }

    builder = builder.header("User-Agent", format!("xurl/{}", env!("CARGO_PKG_VERSION")));

    if options.trace {
        builder = builder.header("X-B3-Flags", "1");
    }

    if options.verbose {
        eprintln!("\x1b[1;34m> {method}\x1b[0m {url}");
    }

    out.status(&format!(
        "Connecting to streaming endpoint: {}",
        options.endpoint
    ));

    let resp = builder.send()?;

    if options.verbose {
        eprintln!("\x1b[1;31m< {}\x1b[0m", resp.status());
        for (key, value) in resp.headers() {
            eprintln!(
                "\x1b[1;32m< {}\x1b[0m: {}",
                key,
                value.to_str().unwrap_or("")
            );
        }
        eprintln!();
    }

    if resp.status().as_u16() >= 400 {
        let body = resp.text().unwrap_or_default();
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
            return Err(XurlError::api(json.to_string()));
        }
        return Err(XurlError::api(body));
    }

    out.status("--- Streaming response started ---");
    out.status("--- Press Ctrl+C to stop ---");

    let reader = BufReader::with_capacity(1024 * 1024, resp);
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }
                out.print_stream_line(&line);
            }
            Err(e) => {
                return Err(XurlError::Io(e.to_string()));
            }
        }
    }

    out.status("--- End of stream ---");
    Ok(())
}

/// Runs a subcommand.
#[allow(clippy::too_many_lines)]
fn run_subcommand(
    cmd: Commands,
    cfg: &Config,
    auth: &mut Auth,
    no_interactive: bool,
    out: &OutputConfig,
) -> Result<()> {
    match cmd {
        // ── Posting ──────────────────────────────────────────────────
        Commands::Post {
            text,
            media_ids,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let response = api::create_post(&mut client, &text, &media_ids, &opts)?;
            out.print_response(&response);
        }
        Commands::Reply {
            post_id,
            text,
            media_ids,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let response = api::reply_to_post(&mut client, &post_id, &text, &media_ids, &opts)?;
            out.print_response(&response);
        }
        Commands::Quote {
            post_id,
            text,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let response = api::quote_post(&mut client, &post_id, &text, &opts)?;
            out.print_response(&response);
        }
        Commands::Delete { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let response = api::delete_post(&mut client, &post_id, &opts)?;
            out.print_response(&response);
        }

        // ── Reading ──────────────────────────────────────────────────
        Commands::Read { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let response = api::read_post(&mut client, &post_id, &opts)?;
            out.print_response(&response);
        }
        Commands::Search {
            query,
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let response = api::search_posts(&mut client, &query, max_results, &opts)?;
            out.print_response(&response);
        }

        // ── User Info ────────────────────────────────────────────────
        Commands::Whoami { common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let response = api::get_me(&mut client, &opts)?;
            out.print_response(&response);
        }
        Commands::User {
            target_username,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let response = api::lookup_user(&mut client, &target_username, &opts)?;
            out.print_response(&response);
        }

        // ── Timeline & Mentions ──────────────────────────────────────
        Commands::Timeline {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::get_timeline(&mut client, &user_id, max_results, &opts)?;
            out.print_response(&response);
        }
        Commands::Mentions {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::get_mentions(&mut client, &user_id, max_results, &opts)?;
            out.print_response(&response);
        }

        // ── Engagement ───────────────────────────────────────────────
        Commands::Like { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::like_post(&mut client, &user_id, &post_id, &opts)?;
            out.print_response(&response);
        }
        Commands::Unlike { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::unlike_post(&mut client, &user_id, &post_id, &opts)?;
            out.print_response(&response);
        }
        Commands::Repost { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::repost(&mut client, &user_id, &post_id, &opts)?;
            out.print_response(&response);
        }
        Commands::Unrepost { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::unrepost(&mut client, &user_id, &post_id, &opts)?;
            out.print_response(&response);
        }
        Commands::Bookmark { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::bookmark(&mut client, &user_id, &post_id, &opts)?;
            out.print_response(&response);
        }
        Commands::Unbookmark { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::unbookmark(&mut client, &user_id, &post_id, &opts)?;
            out.print_response(&response);
        }
        Commands::Bookmarks {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::get_bookmarks(&mut client, &user_id, max_results, &opts)?;
            out.print_response(&response);
        }
        Commands::Likes {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::get_liked_posts(&mut client, &user_id, max_results, &opts)?;
            out.print_response(&response);
        }

        // ── Social Graph ─────────────────────────────────────────────
        Commands::Follow {
            target_username,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let my_id = resolve_my_user_id(&mut client, &opts)?;
            let target_id = resolve_user_id(&mut client, &target_username, &opts)?;
            let response = api::follow_user(&mut client, &my_id, &target_id, &opts)?;
            out.print_response(&response);
        }
        Commands::Unfollow {
            target_username,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let my_id = resolve_my_user_id(&mut client, &opts)?;
            let target_id = resolve_user_id(&mut client, &target_username, &opts)?;
            let response = api::unfollow_user(&mut client, &my_id, &target_id, &opts)?;
            out.print_response(&response);
        }
        Commands::Following {
            max_results,
            of,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = if let Some(ref target) = of {
                resolve_user_id(&mut client, target, &opts)?
            } else {
                resolve_my_user_id(&mut client, &opts)?
            };
            let response = api::get_following(&mut client, &user_id, max_results, &opts)?;
            out.print_response(&response);
        }
        Commands::Followers {
            max_results,
            of,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = if let Some(ref target) = of {
                resolve_user_id(&mut client, target, &opts)?
            } else {
                resolve_my_user_id(&mut client, &opts)?
            };
            let response = api::get_followers(&mut client, &user_id, max_results, &opts)?;
            out.print_response(&response);
        }
        Commands::Block {
            target_username,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let my_id = resolve_my_user_id(&mut client, &opts)?;
            let target_id = resolve_user_id(&mut client, &target_username, &opts)?;
            let response = api::block_user(&mut client, &my_id, &target_id, &opts)?;
            out.print_response(&response);
        }
        Commands::Unblock {
            target_username,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let my_id = resolve_my_user_id(&mut client, &opts)?;
            let target_id = resolve_user_id(&mut client, &target_username, &opts)?;
            let response = api::unblock_user(&mut client, &my_id, &target_id, &opts)?;
            out.print_response(&response);
        }
        Commands::Mute {
            target_username,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let my_id = resolve_my_user_id(&mut client, &opts)?;
            let target_id = resolve_user_id(&mut client, &target_username, &opts)?;
            let response = api::mute_user(&mut client, &my_id, &target_id, &opts)?;
            out.print_response(&response);
        }
        Commands::Unmute {
            target_username,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let my_id = resolve_my_user_id(&mut client, &opts)?;
            let target_id = resolve_user_id(&mut client, &target_username, &opts)?;
            let response = api::unmute_user(&mut client, &my_id, &target_id, &opts)?;
            out.print_response(&response);
        }

        // ── Direct Messages ──────────────────────────────────────────
        Commands::Dm {
            target_username,
            text,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let target_id = resolve_user_id(&mut client, &target_username, &opts)?;
            let response = api::send_dm(&mut client, &target_id, &text, &opts)?;
            out.print_response(&response);
        }
        Commands::Dms {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let response = api::get_dm_events(&mut client, max_results, &opts)?;
            out.print_response(&response);
        }

        // ── Auth ─────────────────────────────────────────────────────
        Commands::Auth { command } => {
            return run_auth_command(command, auth, no_interactive, out);
        }

        // ── Media ────────────────────────────────────────────────────
        Commands::Media { command } => {
            return run_media_command(command, cfg, auth, out);
        }

        // ── Meta (handled before config init in main) ───────────────
        Commands::Completions { .. } => {
            unreachable!("completions is handled before config init in main()")
        }
        Commands::Version => {
            unreachable!("version is handled before config init in main()")
        }
    }
    Ok(())
}

// ── Auth subcommand handlers ─────────────────────────────────────────

#[allow(clippy::too_many_lines)]
fn run_auth_command(
    cmd: AuthCommands,
    auth: &mut Auth,
    no_interactive: bool,
    out: &OutputConfig,
) -> Result<()> {
    match cmd {
        AuthCommands::Oauth2 => {
            auth.oauth2_flow("")?;
            out.print_message("\x1b[32mOAuth2 authentication successful!\x1b[0m");
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
                return Err(XurlError::Api(
                    "No authentication cleared! Use --all to clear all authentication.".to_string(),
                ));
            }
        }
        AuthCommands::Apps { command } => {
            return run_app_command(command, auth, out);
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
                        return Err(XurlError::Api(format!("Selection error: {e}")));
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
                return Err(XurlError::Api(
                    "Nothing to update. Provide --client-id and/or --client-secret.".to_string(),
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

// ── Media subcommand handlers ────────────────────────────────────────

fn run_media_command(
    cmd: MediaCommands,
    cfg: &Config,
    auth: &mut Auth,
    out: &OutputConfig,
) -> Result<()> {
    match cmd {
        MediaCommands::Upload {
            file,
            media_type,
            category,
            wait,
            auth_type,
            username,
            verbose,
            trace,
            headers,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            api::execute_media_upload(
                &file,
                &media_type,
                &category,
                &auth_type.unwrap_or_default(),
                &username.unwrap_or_default(),
                verbose,
                trace,
                wait,
                &headers,
                &mut client,
                out,
            )
        }
        MediaCommands::Status {
            media_id,
            auth_type,
            username,
            verbose,
            wait,
            trace,
            headers,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            api::execute_media_status(
                &media_id,
                &auth_type.unwrap_or_default(),
                &username.unwrap_or_default(),
                verbose,
                wait,
                trace,
                &headers,
                &mut client,
                out,
            )
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Resolves the authenticated user's ID from /2/users/me.
fn resolve_my_user_id(client: &mut ApiClient, opts: &RequestOptions) -> Result<String> {
    let resp = api::get_me(client, opts)?;
    resp["data"]["id"]
        .as_str()
        .filter(|id| !id.is_empty())
        .map(std::string::ToString::to_string)
        .ok_or_else(|| XurlError::auth("user ID was empty -- check your auth tokens"))
}

/// Resolves a username to a user ID.
fn resolve_user_id(
    client: &mut ApiClient,
    username: &str,
    opts: &RequestOptions,
) -> Result<String> {
    let resp = api::lookup_user(client, username, opts)?;
    let clean = username.trim_start_matches('@');
    resp["data"]["id"]
        .as_str()
        .filter(|id| !id.is_empty())
        .map(std::string::ToString::to_string)
        .ok_or_else(|| XurlError::Api(format!("user @{clean} not found")))
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
