/// Command execution — dispatches CLI commands to API functions.
mod auth;
mod media;
pub mod schema;
pub mod skill;
mod streaming;

use std::io::Write;

use serde::Serialize;

use crate::api::{self, ApiClient, CallOptions, RequestOptions};
use crate::auth::Auth;
use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::error::{Result, XurlError};
use crate::output::OutputConfig;

/// Converts a typed response to Value and prints it.
fn print_typed<T: Serialize>(
    out: &OutputConfig,
    stdout: &mut dyn Write,
    response: &T,
) -> Result<()> {
    let value = serde_json::to_value(response)?;
    out.print_response(stdout, &value);
    Ok(())
}

/// Runs the CLI — dispatches to the appropriate handler.
///
/// `auth` is constructed by the caller (typically `xurl::cli::runner`) so the
/// token-store path is injected explicitly rather than resolved here. `stdout`
/// and `stderr` are the runner's writers; passing them through every command
/// handler lets library tests capture all output into `Vec<u8>`.
///
/// # Errors
///
/// Returns an error if the command fails.
pub fn run(
    cli: Cli,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    mut auth: Auth,
) -> Result<()> {
    let cfg = Config::new();

    // KTD6: capture whether the user passed `--app` BEFORE `cli.app` is
    // collapsed into `auth.with_app_name(...)` below. The collapsed
    // `Config.app_name` is always `"default"` in normal runtime paths and
    // therefore cannot distinguish "user passed --app default" from "user
    // passed nothing"; the boolean derived here threads through to
    // `run_auth_command` so the credential-less-default warning gates
    // correctly (R13).
    let app_explicit = cli.app.is_some();

    // Apply --app override
    if let Some(ref app_name) = cli.app {
        auth.with_app_name(app_name);
    }

    let no_interactive = cli.no_interactive;
    let verbose = cli.verbose;
    match cli.command {
        Some(cmd) => run_subcommand(
            cmd,
            &cfg,
            auth,
            no_interactive,
            verbose,
            out,
            stdout,
            stderr,
            app_explicit,
        ),
        None => run_raw_mode(&cli, &cfg, auth, out, stdout, stderr),
    }
}

/// Runs raw curl-style mode.
fn run_raw_mode(
    cli: &Cli,
    cfg: &Config,
    auth: Auth,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<()> {
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
        out.print_response(stdout, &response);
        return Ok(());
    }

    let should_stream = cli.stream || api::is_streaming_endpoint(&options.endpoint);

    if should_stream {
        streaming::stream_request_with_output(&mut client, &options, out, stdout, stderr)
    } else {
        let response = client.send_request(&options)?;
        out.print_response(stdout, &response);
        Ok(())
    }
}

/// Runs a subcommand.
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
fn run_subcommand(
    cmd: Commands,
    cfg: &Config,
    auth: Auth,
    no_interactive: bool,
    verbose: bool,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    app_explicit: bool,
) -> Result<()> {
    match cmd {
        // ── Posting ──────────────────────────────────────────────────
        Commands::Post {
            text,
            media_ids,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let response = client.create_post(&text, &media_ids, &opts)?;
            // NOTE: All match arms below follow this same pattern — auth is moved
            // into ApiClient::new(). The compiler ensures only one arm executes.
            print_typed(out, stdout, &response)?;
        }
        Commands::Reply {
            post_id,
            text,
            media_ids,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let response = client.reply_to_post(&post_id, &text, &media_ids, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Quote {
            post_id,
            text,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let response = client.quote_post(&post_id, &text, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Delete { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let response = client.delete_post(&post_id, &opts)?;
            print_typed(out, stdout, &response)?;
        }

        // ── Reading ──────────────────────────────────────────────────
        Commands::Read { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let response = client.read_post(&post_id, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Search {
            query,
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let response = client.search_posts(&query, max_results, &opts)?;
            print_typed(out, stdout, &response)?;
        }

        // ── User Info ────────────────────────────────────────────────
        Commands::Whoami { common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let response = client.get_me(&opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::User {
            target_username,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let response = client.lookup_user(&target_username, &opts)?;
            print_typed(out, stdout, &response)?;
        }

        // ── Timeline & Mentions ──────────────────────────────────────
        Commands::Timeline {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = client.get_timeline(&user_id, max_results, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Mentions {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = client.get_mentions(&user_id, max_results, &opts)?;
            print_typed(out, stdout, &response)?;
        }

        // ── Engagement ───────────────────────────────────────────────
        Commands::Like { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = client.like_post(&user_id, &post_id, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Unlike { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = client.unlike_post(&user_id, &post_id, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Repost { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = client.repost(&user_id, &post_id, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Unrepost { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = client.unrepost(&user_id, &post_id, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Bookmark { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = client.bookmark(&user_id, &post_id, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Unbookmark { post_id, common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = client.unbookmark(&user_id, &post_id, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Bookmarks {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = client.get_bookmarks(&user_id, max_results, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Likes {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let user_id = resolve_my_user_id(&mut client, &opts)?;
            let response = client.get_liked_posts(&user_id, max_results, &opts)?;
            print_typed(out, stdout, &response)?;
        }

        // ── Social Graph ─────────────────────────────────────────────
        Commands::Follow {
            target_username,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let my_id = resolve_my_user_id(&mut client, &opts)?;
            let target_id = resolve_user_id(&mut client, &target_username, &opts)?;
            let response = client.follow_user(&my_id, &target_id, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Unfollow {
            target_username,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let my_id = resolve_my_user_id(&mut client, &opts)?;
            let target_id = resolve_user_id(&mut client, &target_username, &opts)?;
            let response = client.unfollow_user(&my_id, &target_id, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Following {
            max_results,
            of,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let user_id = if let Some(ref target) = of {
                resolve_user_id(&mut client, target, &opts)?
            } else {
                resolve_my_user_id(&mut client, &opts)?
            };
            let response = client.get_following(&user_id, max_results, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Followers {
            max_results,
            of,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let user_id = if let Some(ref target) = of {
                resolve_user_id(&mut client, target, &opts)?
            } else {
                resolve_my_user_id(&mut client, &opts)?
            };
            let response = client.get_followers(&user_id, max_results, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Block {
            target_username,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let my_id = resolve_my_user_id(&mut client, &opts)?;
            let target_id = resolve_user_id(&mut client, &target_username, &opts)?;
            let response = client.block_user(&my_id, &target_id, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Unblock {
            target_username,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let my_id = resolve_my_user_id(&mut client, &opts)?;
            let target_id = resolve_user_id(&mut client, &target_username, &opts)?;
            let response = client.unblock_user(&my_id, &target_id, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Mute {
            target_username,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let my_id = resolve_my_user_id(&mut client, &opts)?;
            let target_id = resolve_user_id(&mut client, &target_username, &opts)?;
            let response = client.mute_user(&my_id, &target_id, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Unmute {
            target_username,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let my_id = resolve_my_user_id(&mut client, &opts)?;
            let target_id = resolve_user_id(&mut client, &target_username, &opts)?;
            let response = client.unmute_user(&my_id, &target_id, &opts)?;
            print_typed(out, stdout, &response)?;
        }

        // ── Usage ─────────────────────────────────────────────────────
        Commands::Usage { common } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let response = client.get_usage(&opts)?;
            print_typed(out, stdout, &response)?;
        }

        // ── Direct Messages ──────────────────────────────────────────
        Commands::Dm {
            target_username,
            text,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let target_id = resolve_user_id(&mut client, &target_username, &opts)?;
            let response = client.send_dm(&target_id, &text, &opts)?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Dms {
            max_results,
            common,
        } => {
            let mut client = ApiClient::new(cfg, auth);
            let opts = common.to_call_options(verbose);
            let response = client.get_dm_events(max_results, &opts)?;
            print_typed(out, stdout, &response)?;
        }

        // ── Auth ─────────────────────────────────────────────────────
        Commands::Auth { command } => {
            return auth::run_auth_command(
                command,
                auth,
                no_interactive,
                out,
                stdout,
                stderr,
                app_explicit,
            );
        }

        // ── Media ────────────────────────────────────────────────────
        Commands::Media { command } => {
            return media::run_media_command(command, cfg, auth, verbose, out, stdout, stderr);
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
        Commands::Skill { .. } => {
            unreachable!("skill is handled before config init in main()")
        }
    }
    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Resolves the authenticated user's ID.
///
/// When `opts.username` is empty, calls `/2/users/me` (the default identity
/// for the active credential). When non-empty, calls
/// `/2/users/by/username/<u>` directly, bypassing `/me` so the shortcut works
/// when `/me` is unavailable or when the caller wants to act under a known
/// handle without consulting `/me`.
fn resolve_my_user_id(client: &mut ApiClient, opts: &CallOptions) -> Result<String> {
    let id = if opts.username.is_empty() {
        client.get_me(opts)?.data.id
    } else {
        client.lookup_user(&opts.username, opts)?.data.id
    };
    if id.is_empty() {
        return Err(XurlError::auth(
            "user ID was empty -- check your auth tokens",
        ));
    }
    Ok(id)
}

/// Resolves a username to a user ID.
fn resolve_user_id(client: &mut ApiClient, username: &str, opts: &CallOptions) -> Result<String> {
    let resp = client.lookup_user(username, opts)?;
    let id = &resp.data.id;
    if id.is_empty() {
        let clean = username.trim_start_matches('@');
        return Err(XurlError::validation(format!("user @{clean} not found")));
    }
    Ok(id.clone())
}
