/// Command execution — dispatches CLI commands to API functions.
use std::process;

use crate::api::{self, ApiClient, RequestOptions};
use crate::api::response::format_and_print_response;
use crate::auth::Auth;
use crate::cli::{
    AppCommands, AuthCommands, Cli, Commands, MediaCommands,
};
use crate::config::Config;
use crate::store::TokenStore;

/// Runs the CLI — dispatches to the appropriate handler.
pub fn run(cli: Cli) {
    let cfg = Config::new();
    let mut auth = Auth::new(&cfg);

    // Apply --app override
    if let Some(ref app_name) = cli.app {
        auth.with_app_name(app_name);
    }

    match cli.command {
        Some(cmd) => run_subcommand(cmd, &cfg, &mut auth),
        None => run_raw_mode(&cli, &cfg, &mut auth),
    }
}

/// Runs raw curl-style mode.
fn run_raw_mode(cli: &Cli, cfg: &Config, auth: &mut Auth) {
    let url = match &cli.url {
        Some(u) => u.clone(),
        None => {
            println!("No URL provided");
            println!("Usage: xurl [OPTIONS] [URL] [COMMAND]");
            println!("Try 'xurl --help' for more information.");
            process::exit(1);
        }
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
        match api::handle_media_append_request(&options, &media_file, &mut client) {
            Ok(response) => format_and_print_response(&response),
            Err(e) => {
                print_error(&e);
                process::exit(1);
            }
        }
        return;
    }

    let should_stream =
        cli.stream || api::is_streaming_endpoint(&options.endpoint);

    if should_stream {
        if let Err(e) = client.stream_request(&options) {
            print_error(&e);
            process::exit(1);
        }
    } else {
        match client.send_request(&options) {
            Ok(response) => format_and_print_response(&response),
            Err(e) => {
                handle_api_error(&e);
                process::exit(1);
            }
        }
    }
}

/// Runs a subcommand.
fn run_subcommand(cmd: Commands, cfg: &Config, auth: &mut Auth) {
    match cmd {
        // ── Posting ──────────────────────────────────────────────────
        Commands::Post {
            text,
            media_ids,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            print_result(api::create_post(&mut client, &text, &media_ids, &opts));
        }
        Commands::Reply {
            post_id,
            text,
            media_ids,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            print_result(api::reply_to_post(
                &mut client,
                &post_id,
                &text,
                &media_ids,
                &opts,
            ));
        }
        Commands::Quote {
            post_id,
            text,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            print_result(api::quote_post(&mut client, &post_id, &text, &opts));
        }
        Commands::Delete { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            print_result(api::delete_post(&mut client, &post_id, &opts));
        }

        // ── Reading ──────────────────────────────────────────────────
        Commands::Read { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            print_result(api::read_post(&mut client, &post_id, &opts));
        }
        Commands::Search {
            query,
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            print_result(api::search_posts(&mut client, &query, max_results, &opts));
        }

        // ── User Info ────────────────────────────────────────────────
        Commands::Whoami { common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            print_result(api::get_me(&mut client, &opts));
        }
        Commands::User { target_username, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            print_result(api::lookup_user(&mut client, &target_username, &opts));
        }

        // ── Timeline & Mentions ──────────────────────────────────────
        Commands::Timeline {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts);
            print_result(api::get_timeline(&mut client, &user_id, max_results, &opts));
        }
        Commands::Mentions {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts);
            print_result(api::get_mentions(&mut client, &user_id, max_results, &opts));
        }

        // ── Engagement ───────────────────────────────────────────────
        Commands::Like { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts);
            print_result(api::like_post(&mut client, &user_id, &post_id, &opts));
        }
        Commands::Unlike { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts);
            print_result(api::unlike_post(&mut client, &user_id, &post_id, &opts));
        }
        Commands::Repost { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts);
            print_result(api::repost(&mut client, &user_id, &post_id, &opts));
        }
        Commands::Unrepost { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts);
            print_result(api::unrepost(&mut client, &user_id, &post_id, &opts));
        }
        Commands::Bookmark { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts);
            print_result(api::bookmark(&mut client, &user_id, &post_id, &opts));
        }
        Commands::Unbookmark { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts);
            print_result(api::unbookmark(&mut client, &user_id, &post_id, &opts));
        }
        Commands::Bookmarks {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts);
            print_result(api::get_bookmarks(&mut client, &user_id, max_results, &opts));
        }
        Commands::Likes {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts);
            print_result(api::get_liked_posts(&mut client, &user_id, max_results, &opts));
        }

        // ── Social Graph ─────────────────────────────────────────────
        Commands::Follow { target_username, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let my_id = resolve_my_user_id(&mut client, &opts);
            let target_id = resolve_user_id(&mut client, &target_username, &opts);
            print_result(api::follow_user(&mut client, &my_id, &target_id, &opts));
        }
        Commands::Unfollow { target_username, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let my_id = resolve_my_user_id(&mut client, &opts);
            let target_id = resolve_user_id(&mut client, &target_username, &opts);
            print_result(api::unfollow_user(&mut client, &my_id, &target_id, &opts));
        }
        Commands::Following {
            max_results,
            of,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = match of {
                Some(ref target) => resolve_user_id(&mut client, target, &opts),
                None => resolve_my_user_id(&mut client, &opts),
            };
            print_result(api::get_following(&mut client, &user_id, max_results, &opts));
        }
        Commands::Followers {
            max_results,
            of,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = match of {
                Some(ref target) => resolve_user_id(&mut client, target, &opts),
                None => resolve_my_user_id(&mut client, &opts),
            };
            print_result(api::get_followers(&mut client, &user_id, max_results, &opts));
        }
        Commands::Block { target_username, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let my_id = resolve_my_user_id(&mut client, &opts);
            let target_id = resolve_user_id(&mut client, &target_username, &opts);
            print_result(api::block_user(&mut client, &my_id, &target_id, &opts));
        }
        Commands::Unblock { target_username, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let my_id = resolve_my_user_id(&mut client, &opts);
            let target_id = resolve_user_id(&mut client, &target_username, &opts);
            print_result(api::unblock_user(&mut client, &my_id, &target_id, &opts));
        }
        Commands::Mute { target_username, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let my_id = resolve_my_user_id(&mut client, &opts);
            let target_id = resolve_user_id(&mut client, &target_username, &opts);
            print_result(api::mute_user(&mut client, &my_id, &target_id, &opts));
        }
        Commands::Unmute { target_username, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let my_id = resolve_my_user_id(&mut client, &opts);
            let target_id = resolve_user_id(&mut client, &target_username, &opts);
            print_result(api::unmute_user(&mut client, &my_id, &target_id, &opts));
        }

        // ── Direct Messages ──────────────────────────────────────────
        Commands::Dm {
            target_username,
            text,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let target_id = resolve_user_id(&mut client, &target_username, &opts);
            print_result(api::send_dm(&mut client, &target_id, &text, &opts));
        }
        Commands::Dms {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            print_result(api::get_dm_events(&mut client, max_results, &opts));
        }

        // ── Auth ─────────────────────────────────────────────────────
        Commands::Auth { command } => run_auth_command(command, auth),

        // ── Media ────────────────────────────────────────────────────
        Commands::Media { command } => run_media_command(command, cfg, auth),

        // ── Version ──────────────────────────────────────────────────
        Commands::Version => {
            println!("xurl {}", env!("CARGO_PKG_VERSION"));
        }
    }
}

// ── Auth subcommand handlers ─────────────────────────────────────────

fn run_auth_command(cmd: AuthCommands, auth: &mut Auth) {
    match cmd {
        AuthCommands::Oauth2 => {
            match auth.oauth2_flow("") {
                Ok(_) => println!("\x1b[32mOAuth2 authentication successful!\x1b[0m"),
                Err(e) => {
                    println!("OAuth2 authentication failed: {e}");
                    process::exit(1);
                }
            }
        }
        AuthCommands::Oauth1 {
            consumer_key,
            consumer_secret,
            access_token,
            token_secret,
        } => {
            match auth.token_store.save_oauth1_tokens(
                &access_token,
                &token_secret,
                &consumer_key,
                &consumer_secret,
            ) {
                Ok(()) => println!("\x1b[32mOAuth1 credentials saved successfully!\x1b[0m"),
                Err(e) => {
                    println!("Error saving OAuth1 tokens: {e}");
                    process::exit(1);
                }
            }
        }
        AuthCommands::App { bearer_token } => {
            match auth.token_store.save_bearer_token(&bearer_token) {
                Ok(()) => println!("\x1b[32mApp authentication successful!\x1b[0m"),
                Err(e) => {
                    println!("Error saving bearer token: {e}");
                    process::exit(1);
                }
            }
        }
        AuthCommands::Status => {
            let ts = TokenStore::new();
            let apps = ts.list_apps();
            let default_app = ts.get_default_app();

            if apps.is_empty() {
                println!("No apps registered. Use 'xurl auth apps add' to register one.");
                return;
            }

            for (i, name) in apps.iter().enumerate() {
                if let Some(app) = ts.get_app(name) {
                    let marker = if name == default_app { "\u{25b8}" } else { " " };
                    let client_hint = if !app.client_id.is_empty() {
                        format!("client_id: {}...", truncate(&app.client_id, 8))
                    } else {
                        "(no credentials)".to_string()
                    };
                    println!("{marker} {name}  [{client_hint}]");

                    let usernames = ts.get_oauth2_usernames_for_app(name);
                    if !usernames.is_empty() {
                        for u in &usernames {
                            if *u == app.default_user {
                                println!("    \u{25b8} oauth2: {u}");
                            } else {
                                println!("      oauth2: {u}");
                            }
                        }
                    } else {
                        println!("      oauth2: (none)");
                    }

                    if app.oauth1_token.is_some() {
                        println!("      oauth1: \u{2713}");
                    } else {
                        println!("      oauth1: \u{2013}");
                    }

                    if app.bearer_token.is_some() {
                        println!("      bearer: \u{2713}");
                    } else {
                        println!("      bearer: \u{2013}");
                    }

                    if i < apps.len() - 1 {
                        println!();
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
                match auth.token_store.clear_all() {
                    Ok(()) => println!("All authentication cleared!"),
                    Err(e) => {
                        println!("Error clearing all tokens: {e}");
                        process::exit(1);
                    }
                }
            } else if oauth1 {
                match auth.token_store.clear_oauth1_tokens() {
                    Ok(()) => println!("OAuth1 tokens cleared!"),
                    Err(e) => {
                        println!("Error clearing OAuth1 tokens: {e}");
                        process::exit(1);
                    }
                }
            } else if let Some(username) = oauth2_username {
                match auth.token_store.clear_oauth2_token(&username) {
                    Ok(()) => println!("OAuth2 token cleared for {username}!"),
                    Err(e) => {
                        println!("Error clearing OAuth2 token: {e}");
                        process::exit(1);
                    }
                }
            } else if bearer {
                match auth.token_store.clear_bearer_token() {
                    Ok(()) => println!("Bearer token cleared!"),
                    Err(e) => {
                        println!("Error clearing bearer token: {e}");
                        process::exit(1);
                    }
                }
            } else {
                println!("No authentication cleared! Use --all to clear all authentication.");
                process::exit(1);
            }
        }
        AuthCommands::Apps { command } => run_app_command(command, auth),
        AuthCommands::Default {
            app_name,
            username,
        } => {
            if let Some(app_name) = app_name {
                match auth.token_store.set_default_app(&app_name) {
                    Ok(()) => println!("\x1b[32mDefault app set to {app_name:?}\x1b[0m"),
                    Err(e) => {
                        println!("\x1b[31mError: {e}\x1b[0m");
                        process::exit(1);
                    }
                }
                if let Some(user) = username {
                    match auth.token_store.set_default_user(&app_name, &user) {
                        Ok(()) => println!("\x1b[32mDefault user set to {user:?}\x1b[0m"),
                        Err(e) => {
                            println!("\x1b[31mError: {e}\x1b[0m");
                            process::exit(1);
                        }
                    }
                }
            } else {
                // Interactive picker
                let apps = auth.token_store.list_apps();
                if apps.is_empty() {
                    println!("No apps registered. Use 'xurl auth apps add' to register one.");
                    return;
                }

                let app_choice = match dialoguer::Select::new()
                    .with_prompt("Select default app")
                    .items(&apps)
                    .interact_opt()
                {
                    Ok(Some(idx)) => apps[idx].clone(),
                    Ok(None) => return,
                    Err(e) => {
                        println!("\x1b[31mError: {e}\x1b[0m");
                        process::exit(1);
                    }
                };

                match auth.token_store.set_default_app(&app_choice) {
                    Ok(()) => println!("\x1b[32mDefault app set to {app_choice:?}\x1b[0m"),
                    Err(e) => {
                        println!("\x1b[31mError: {e}\x1b[0m");
                        process::exit(1);
                    }
                }

                let users = auth.token_store.get_oauth2_usernames_for_app(&app_choice);
                if !users.is_empty()
                    && let Ok(Some(idx)) = dialoguer::Select::new()
                        .with_prompt("Select default OAuth2 user")
                        .items(&users)
                        .interact_opt()
                    {
                        let user = &users[idx];
                        match auth.token_store.set_default_user(&app_choice, user) {
                            Ok(()) => println!("\x1b[32mDefault user set to {user:?}\x1b[0m"),
                            Err(e) => {
                                println!("\x1b[31mError: {e}\x1b[0m");
                                process::exit(1);
                            }
                        }
                    }
            }
        }
    }
}

fn run_app_command(cmd: AppCommands, auth: &mut Auth) {
    match cmd {
        AppCommands::Add {
            name,
            client_id,
            client_secret,
        } => {
            match auth.token_store.add_app(&name, &client_id, &client_secret) {
                Ok(()) => {
                    println!("\x1b[32mApp {name:?} registered!\x1b[0m");
                    if auth.token_store.list_apps().len() == 1 {
                        println!("  (set as default app)");
                    }
                }
                Err(e) => {
                    println!("\x1b[31mError: {e}\x1b[0m");
                    process::exit(1);
                }
            }
        }
        AppCommands::Update {
            name,
            client_id,
            client_secret,
        } => {
            if client_id.is_none() && client_secret.is_none() {
                println!("Nothing to update. Provide --client-id and/or --client-secret.");
                process::exit(1);
            }
            match auth.token_store.update_app(
                &name,
                &client_id.unwrap_or_default(),
                &client_secret.unwrap_or_default(),
            ) {
                Ok(()) => println!("\x1b[32mApp {name:?} updated.\x1b[0m"),
                Err(e) => {
                    println!("\x1b[31mError: {e}\x1b[0m");
                    process::exit(1);
                }
            }
        }
        AppCommands::Remove { name } => {
            match auth.token_store.remove_app(&name) {
                Ok(()) => println!("\x1b[32mApp {name:?} removed.\x1b[0m"),
                Err(e) => {
                    println!("\x1b[31mError: {e}\x1b[0m");
                    process::exit(1);
                }
            }
        }
        AppCommands::List => {
            let ts = TokenStore::new();
            let apps = ts.list_apps();
            let default_app = ts.get_default_app();

            if apps.is_empty() {
                println!("No apps registered. Use 'xurl auth apps add' to register one.");
                return;
            }

            for name in &apps {
                if let Some(app) = ts.get_app(name) {
                    let marker = if name == default_app {
                        "\u{25b8} "
                    } else {
                        "  "
                    };
                    let client_hint = if !app.client_id.is_empty() {
                        format!(" (client_id: {}...)", truncate(&app.client_id, 8))
                    } else {
                        String::new()
                    };
                    println!("{marker}{name}{client_hint}");
                }
            }
        }
    }
}

// ── Media subcommand handlers ────────────────────────────────────────

fn run_media_command(cmd: MediaCommands, cfg: &Config, auth: &mut Auth) {
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
            if let Err(e) = api::execute_media_upload(
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
            ) {
                println!("\x1b[31m{e}\x1b[0m");
                process::exit(1);
            }
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
            if let Err(e) = api::execute_media_status(
                &media_id,
                &auth_type.unwrap_or_default(),
                &username.unwrap_or_default(),
                verbose,
                wait,
                trace,
                &headers,
                &mut client,
            ) {
                println!("\x1b[31m{e}\x1b[0m");
                process::exit(1);
            }
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Resolves the authenticated user's ID from /2/users/me.
fn resolve_my_user_id(client: &mut ApiClient, opts: &RequestOptions) -> String {
    match api::get_me(client, opts) {
        Ok(resp) => {
            if let Some(id) = resp["data"]["id"].as_str()
                && !id.is_empty() {
                    return id.to_string();
                }
            eprintln!("\x1b[31mError: user ID was empty -- check your auth tokens\x1b[0m");
            process::exit(1);
        }
        Err(e) => {
            eprintln!(
                "\x1b[31mError: could not resolve your user ID (are you authenticated?): {e}\x1b[0m"
            );
            process::exit(1);
        }
    }
}

/// Resolves a username to a user ID.
fn resolve_user_id(client: &mut ApiClient, username: &str, opts: &RequestOptions) -> String {
    match api::lookup_user(client, username, opts) {
        Ok(resp) => {
            if let Some(id) = resp["data"]["id"].as_str()
                && !id.is_empty() {
                    return id.to_string();
                }
            let clean = username.trim_start_matches('@');
            eprintln!("\x1b[31mError: user @{clean} not found\x1b[0m");
            process::exit(1);
        }
        Err(e) => {
            let clean = username.trim_start_matches('@');
            eprintln!("\x1b[31mError: could not look up user @{clean}: {e}\x1b[0m");
            process::exit(1);
        }
    }
}

/// Pretty-prints a result or exits on error.
fn print_result(result: crate::error::Result<serde_json::Value>) {
    match result {
        Ok(response) => format_and_print_response(&response),
        Err(e) => {
            handle_api_error(&e);
            process::exit(1);
        }
    }
}

/// Handles API errors — tries to pretty-print JSON error bodies.
fn handle_api_error(e: &crate::error::XurlError) {
    let err_str = e.to_string();
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&err_str) {
        format_and_print_response(&json);
    } else {
        eprintln!("\x1b[31mError: {e}\x1b[0m");
    }
}

/// Prints an error in red.
fn print_error(e: &crate::error::XurlError) {
    eprintln!("\x1b[31mError: {e}\x1b[0m");
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
