/// Command execution — dispatches CLI commands to API functions.
mod auth;
mod media;
pub mod schema;
mod streaming;

use serde::Serialize;

use crate::api::{self, ApiClient, RequestOptions};
use crate::auth::Auth;
use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::error::{Result, XurlError};
use crate::output::OutputConfig;

/// Converts a typed response to Value and prints it.
fn print_typed<T: Serialize>(out: &OutputConfig, response: &T) -> Result<()> {
    let value = serde_json::to_value(response)?;
    out.print_response(&value);
    Ok(())
}

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
        Some(cmd) => run_subcommand(cmd, &cfg, auth, no_interactive, out),
        None => run_raw_mode(&cli, &cfg, auth, out),
    }
}

/// Runs raw curl-style mode.
fn run_raw_mode(cli: &Cli, cfg: &Config, auth: Auth, out: &OutputConfig) -> Result<()> {
    let url = if let Some(u) = &cli.url {
        u.clone()
    } else {
        return Err(XurlError::validation(
            "No URL provided. Usage: xr [OPTIONS] [URL] [COMMAND]. Try 'xr --help' for more information.",
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
        no_auth: false,
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
        streaming::stream_request_with_output(&mut client, &options, out)
    } else {
        let response = client.send_request(&options)?;
        out.print_response(&response);
        Ok(())
    }
}

/// Runs a subcommand.
#[allow(clippy::too_many_lines)]
fn run_subcommand(
    cmd: Commands,
    cfg: &Config,
    auth: Auth,
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
            // NOTE: All match arms below follow this same pattern — auth is moved
            // into ApiClient::new(). The compiler ensures only one arm executes.
            print_typed(out, &response)?;
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
            print_typed(out, &response)?;
        }
        Commands::Quote {
            post_id,
            text,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let response = api::quote_post(&mut client, &post_id, &text, &opts)?;
            print_typed(out, &response)?;
        }
        Commands::Delete { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let response = api::delete_post(&mut client, &post_id, &opts)?;
            print_typed(out, &response)?;
        }

        // ── Reading ──────────────────────────────────────────────────
        Commands::Read { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let response = api::read_post(&mut client, &post_id, &opts)?;
            print_typed(out, &response)?;
        }
        Commands::Search {
            query,
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let response = api::search_posts(&mut client, &query, max_results, &opts)?;
            print_typed(out, &response)?;
        }

        // ── User Info ────────────────────────────────────────────────
        Commands::Whoami { common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let response = api::get_me(&mut client, &opts)?;
            print_typed(out, &response)?;
        }
        Commands::User {
            target_username,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let response = api::lookup_user(&mut client, &target_username, &opts)?;
            print_typed(out, &response)?;
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
            print_typed(out, &response)?;
        }
        Commands::Mentions {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::get_mentions(&mut client, &user_id, max_results, &opts)?;
            print_typed(out, &response)?;
        }

        // ── Engagement ───────────────────────────────────────────────
        Commands::Like { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::like_post(&mut client, &user_id, &post_id, &opts)?;
            print_typed(out, &response)?;
        }
        Commands::Unlike { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::unlike_post(&mut client, &user_id, &post_id, &opts)?;
            print_typed(out, &response)?;
        }
        Commands::Repost { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::repost(&mut client, &user_id, &post_id, &opts)?;
            print_typed(out, &response)?;
        }
        Commands::Unrepost { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::unrepost(&mut client, &user_id, &post_id, &opts)?;
            print_typed(out, &response)?;
        }
        Commands::Bookmark { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::bookmark(&mut client, &user_id, &post_id, &opts)?;
            print_typed(out, &response)?;
        }
        Commands::Unbookmark { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::unbookmark(&mut client, &user_id, &post_id, &opts)?;
            print_typed(out, &response)?;
        }
        Commands::Bookmarks {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::get_bookmarks(&mut client, &user_id, max_results, &opts)?;
            print_typed(out, &response)?;
        }
        Commands::Likes {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = api::get_liked_posts(&mut client, &user_id, max_results, &opts)?;
            print_typed(out, &response)?;
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
            print_typed(out, &response)?;
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
            print_typed(out, &response)?;
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
            print_typed(out, &response)?;
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
            print_typed(out, &response)?;
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
            print_typed(out, &response)?;
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
            print_typed(out, &response)?;
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
            print_typed(out, &response)?;
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
            print_typed(out, &response)?;
        }

        // ── Usage ─────────────────────────────────────────────────────
        Commands::Usage { common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let response = api::get_usage(&mut client, &opts)?;
            print_typed(out, &response)?;
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
            print_typed(out, &response)?;
        }
        Commands::Dms {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_request_options();
            let response = api::get_dm_events(&mut client, max_results, &opts)?;
            print_typed(out, &response)?;
        }

        // ── Auth ─────────────────────────────────────────────────────
        Commands::Auth { command } => {
            return auth::run_auth_command(command, auth, no_interactive, out);
        }

        // ── Media ────────────────────────────────────────────────────
        Commands::Media { command } => {
            return media::run_media_command(command, cfg, auth, out);
        }

        // ── Meta (handled before config init in main) ───────────────
        Commands::Schema { .. } => {
            unreachable!("schema is handled before config init in main()")
        }
        Commands::Completions { .. } => {
            unreachable!("completions is handled before config init in main()")
        }
        Commands::Version => {
            unreachable!("version is handled before config init in main()")
        }
    }
    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Resolves the authenticated user's ID from /2/users/me.
fn resolve_my_user_id(client: &mut ApiClient, opts: &RequestOptions) -> Result<String> {
    let resp = api::get_me(client, opts)?;
    let id = &resp.data.id;
    if id.is_empty() {
        return Err(XurlError::auth(
            "user ID was empty -- check your auth tokens",
        ));
    }
    Ok(id.clone())
}

/// Resolves a username to a user ID.
fn resolve_user_id(
    client: &mut ApiClient,
    username: &str,
    opts: &RequestOptions,
) -> Result<String> {
    let resp = api::lookup_user(client, username, opts)?;
    let id = &resp.data.id;
    if id.is_empty() {
        let clean = username.trim_start_matches('@');
        return Err(XurlError::validation(format!("user @{clean} not found")));
    }
    Ok(id.clone())
}
