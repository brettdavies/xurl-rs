//! Command execution — dispatches CLI commands to API functions.

mod auth;
pub mod examples;
mod media;
pub mod schema;
pub mod skill;
mod streaming;
pub mod validate;

use std::io::{IsTerminal, Write};

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;

use crate::cli::failure::{CommandResult, Failure};
use crate::cli::output::OutputConfig;
use crate::cli::{Cli, Commands, CommonFlags, UsageCommands};
use xdk::api::shortcuts;
use xdk::api::{self, Call, Client, RequestOptions, RequestTarget};
use xdk::auth::Auth;
use xdk::config::Config;
use xdk::error::{EXIT_GENERAL_ERROR, Error, Result};

/// Default page size when neither `--max-results` nor `--limit` is supplied.
const DEFAULT_PAGE_SIZE: i32 = 10;

/// Resolves the effective result limit per U7's rule.
///
/// Per-command `-n/--max-results` takes precedence when set. Otherwise the
/// global `--limit` applies. Otherwise the default falls back to
/// [`DEFAULT_PAGE_SIZE`]. The result is clamped to `1..=100`.
fn effective_limit(per_cmd: Option<i32>, global: Option<i32>) -> i32 {
    per_cmd
        .or(global)
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, 100)
}

/// Returns true when the active session is a real interactive terminal.
///
/// Used to gate dialoguer prompts so they never fire under `--no-interactive`,
/// when stdin/stderr are not TTYs (piped runs, CI), or when `--quiet` is set.
fn is_interactive(no_interactive: bool, quiet: bool) -> bool {
    !no_interactive && !quiet && std::io::stdin().is_terminal() && std::io::stderr().is_terminal()
}

/// Confirms a destructive op interactively via stdin.
///
/// Writes the prompt with `[y/N]` to stderr, reads one line from stdin,
/// returns `Ok(true)` on a `y`/`yes` answer (case-insensitive), `Ok(false)`
/// otherwise (including EOF). `Err` only on IO failure.
///
/// Stays dialoguer-free so the binary doesn't carry an interactive prompt
/// library dependency — gating already happens in [`is_interactive`].
fn confirm_destructive(prompt: &str) -> Result<bool> {
    use std::io::{BufRead, Write as _};
    let stderr = std::io::stderr();
    let mut handle = stderr.lock();
    write!(handle, "{prompt} [y/N]: ")
        .map_err(|e| Error::validation(format!("confirmation prompt failed: {e}")))?;
    handle
        .flush()
        .map_err(|e| Error::validation(format!("confirmation prompt failed: {e}")))?;
    drop(handle);

    let stdin = std::io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_err() {
        return Ok(false);
    }
    let answer = line.trim().to_ascii_lowercase();
    Ok(matches!(answer.as_str(), "y" | "yes"))
}

/// Outcome of the force/confirmation gate for a destructive op.
pub(super) enum Gate {
    /// User confirmed (or supplied `--force`); proceed with the op.
    Proceed,
    /// User declined / unable to prompt: emit nothing further, return Ok(()).
    Declined,
    /// `--no-interactive` set without `--force`: caller must emit a
    /// `confirmation-required` envelope on stderr and return an error code.
    ConfirmationRequired,
}

/// Gates a destructive op on `--force` / TTY confirmation.
///
/// Rules per U7:
/// - `--force` → proceed.
/// - `--no-interactive` without `--force` → ConfirmationRequired.
/// - Interactive terminal without `--force` → dialoguer confirm.
pub(super) fn gate_destructive(
    force: bool,
    no_interactive: bool,
    quiet: bool,
    prompt: &str,
) -> Result<Gate> {
    if force {
        return Ok(Gate::Proceed);
    }
    if no_interactive {
        return Ok(Gate::ConfirmationRequired);
    }
    if !is_interactive(no_interactive, quiet) {
        return Ok(Gate::ConfirmationRequired);
    }
    if confirm_destructive(prompt)? {
        Ok(Gate::Proceed)
    } else {
        Ok(Gate::Declined)
    }
}

/// Validation+envelope helper used by every write handler.
///
/// `validator` produces `Ok(())` for valid inputs or `Err(reason)` with a
/// kebab-case reason. When `dry_run` is set the helper emits the canonical
/// envelope and returns `false` (caller must skip the API call). When
/// `dry_run` is false the helper returns `Ok(true)` (proceed).
///
/// Validation errors surface as either:
///   - Dry-run envelope with `would_succeed: false` and the kebab-case
///     reason when `dry_run` is true.
///   - A `Error::validation` when `dry_run` is false (so the runtime
///     path still rejects bad input).
fn dry_run_or_validate(
    out: &OutputConfig,
    stdout: &mut dyn Write,
    dry_run: bool,
    ctx: serde_json::Value,
    validator: impl FnOnce() -> std::result::Result<(), &'static str>,
) -> Result<bool> {
    let validation = validator();
    if dry_run {
        match validation {
            Ok(()) => out.print_dry_run(stdout, true, 0, &ctx),
            Err(reason) => {
                let mut ctx_with_reason = ctx
                    .as_object()
                    .cloned()
                    .unwrap_or_else(serde_json::Map::new);
                ctx_with_reason.insert(
                    "reason".to_string(),
                    serde_json::Value::String(reason.to_string()),
                );
                out.print_dry_run(
                    stdout,
                    false,
                    1,
                    &serde_json::Value::Object(ctx_with_reason),
                );
            }
        }
        return Ok(false);
    }
    if let Err(reason) = validation {
        return Err(Error::validation(reason.to_string()));
    }
    Ok(true)
}

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

/// The `User-Agent` `xr` sends: the binary's own name and version, not the
/// library's, so X sees the same client identity the CLI has always sent.
const USER_AGENT: &str = concat!("xurl/", env!("CARGO_PKG_VERSION"));

/// The one place the binary builds its API client.
pub(crate) fn make_client(cfg: &Config, auth: Auth) -> Result<Client> {
    Client::new_with_user_agent(cfg, auth, USER_AGENT)
}

/// Runs the CLI — dispatches to the appropriate handler.
///
/// `auth` is constructed by the caller (typically `xurl::cli::runner`) so the
/// token-store path is injected explicitly rather than resolved here. `stdout`
/// and `stderr` are the runner's writers; passing them through every command
/// handler lets library tests capture all output into `Vec<u8>`.
///
/// `overrides` carries the environment the run resolved at its entrypoint, so
/// nothing below this call reads the process.
///
/// # Errors
///
/// Returns an error if the command fails.
pub(crate) async fn run(
    cli: Cli,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    mut auth: Auth,
    overrides: &xdk::config::EnvOverrides,
) -> CommandResult<()> {
    let mut cfg = Config::from_overrides(overrides);
    // Honour --timeout / XURL_TIMEOUT for every HTTP path: API client,
    // OAuth2 token exchange/refresh, and the `/2/users/me` lookup.
    cfg.http_timeout_secs = cli.timeout;

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
    let dry_run = cli.dry_run;
    let global_limit = cli.limit;
    let quiet = cli.quiet;

    // Resolve cursor from --cursor or --after. --page is rejected upstream
    // because the X API does not offer offset-style pagination.
    if cli.page.is_some() {
        out.print_error_envelope(
            stderr,
            "unsupported-pagination",
            EXIT_GENERAL_ERROR,
            "X API does not support offset-style pagination; pass --cursor <token> from the previous response's meta.next_token instead.",
        );
        return Err(Failure::Emitted {
            exit_code: EXIT_GENERAL_ERROR,
        });
    }
    let cursor = cli
        .cursor
        .clone()
        .or_else(|| cli.after.clone())
        .unwrap_or_default();

    match cli.command {
        Some(cmd) => {
            run_subcommand(
                cmd,
                &cfg,
                auth,
                GlobalFlags {
                    no_interactive,
                    verbose,
                    dry_run,
                    global_limit,
                    quiet,
                    app_explicit,
                    cursor,
                },
                out,
                stdout,
                stderr,
            )
            .await
        }
        None => run_raw_mode(&cli, &cfg, auth, out, stdout, stderr)
            .await
            .map_err(Failure::from),
    }
}

/// Bundle of the global flags every subcommand needs.
///
/// Avoids a 12-arg `run_subcommand` signature by collapsing the cross-cutting
/// agentic flags into one record.
#[derive(Debug, Clone)]
struct GlobalFlags {
    no_interactive: bool,
    verbose: bool,
    dry_run: bool,
    global_limit: Option<i32>,
    quiet: bool,
    app_explicit: bool,
    /// Resolved cursor / `pagination_token` from `--cursor` / `--after`.
    /// Empty string when neither flag was supplied. Threaded into
    /// every list call's pagination token.
    cursor: String,
}

/// Runs raw curl-style mode.
async fn run_raw_mode(
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
        return Err(Error::validation(
            "No URL provided. Usage: xr [OPTIONS] [URL] [COMMAND]. Try 'xr --help' for more information.",
        ));
    };

    let method = cli.method.clone().unwrap_or_else(|| "GET".to_string());
    let media_file = cli.file.clone().unwrap_or_default();

    let client = make_client(cfg, auth)?;
    // Raw mode accepts either an absolute http(s) URL OR an absolute path
    // (e.g. `xr POST /2/users/me`). Pre-v2.0.0 `build_url` prepended
    // `api_base_url` whenever the input did not start with `http`; with the
    // typed RequestTarget, RawUrl values must be absolute URLs (the scheme
    // allowlist enforces http/https), so the prepend now happens here.
    let absolute_url = if url.starts_with("http://") || url.starts_with("https://") {
        url.clone()
    } else if url.starts_with('/') {
        format!("{}{}", cfg.api_base_url, url)
    } else {
        return Err(Error::validation(format!(
            "URL {url:?} must be an absolute http(s) URL or an absolute path starting with `/`."
        )));
    };
    // Raw mode threads the (now absolute) URL through as a `RawUrl` target.
    // The matrix validator short-circuits for RawUrl; the
    // streaming/media-append heuristics below inspect the URL string
    // directly — they pre-date the typed target and still make sense for
    // raw curl-style mode.
    let options = RequestOptions {
        method,
        target: RequestTarget::RawUrl(absolute_url.clone()),
        headers: cli.headers.clone(),
        data: cli.data.clone().unwrap_or_default(),
        auth_type: cli.auth_type.clone().unwrap_or_default(),
        username: cli.username.clone().unwrap_or_default(),
        no_auth: false,
        trace: cli.trace,
        pagination_token: cli.cursor.clone().unwrap_or_default(),
    };

    // Check for media append request
    if api::is_media_append_request(&absolute_url, &media_file) {
        let response = api::handle_media_append_request(&options, &media_file, &client).await?;
        out.print_response(stdout, &response);
        return Ok(());
    }

    let should_stream = cli.stream || api::is_streaming_endpoint(&absolute_url);

    if should_stream {
        streaming::stream_request_with_output(&client, &options, out, stdout, stderr).await
    } else {
        let response = client.send_request(&options).await?;
        out.print_response(stdout, &response);
        Ok(())
    }
}

/// Runs a subcommand.
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
async fn run_subcommand(
    cmd: Commands,
    cfg: &Config,
    auth: Auth,
    flags: GlobalFlags,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> CommandResult<()> {
    let GlobalFlags {
        no_interactive,
        verbose,
        dry_run,
        global_limit,
        quiet,
        app_explicit,
        cursor,
    } = flags;
    let cursor_opt = if cursor.is_empty() {
        None
    } else {
        Some(cursor.as_str())
    };
    match cmd {
        // ── Posting ──────────────────────────────────────────────────
        Commands::Post {
            text,
            media_ids,
            common,
        } => {
            let ctx = json!({
                "command": "post",
                "body": text,
                "media_ids": media_ids,
            });
            let body = text.clone();
            let attachments = media_ids.clone();
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_post_body(&body)?;
                shortcuts::validate_media_attachments(&attachments)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let response = with_flags(client.create_post(&text, &media_ids), &common, None)
                .send()
                .await?;
            // NOTE: All match arms below follow this same pattern — auth is moved
            // into Client::new(). The compiler ensures only one arm executes.
            print_typed(out, stdout, &response)?;
        }
        Commands::Reply {
            post_id,
            text,
            media_ids,
            common,
        } => {
            let ctx = json!({
                "command": "reply",
                "post_id": post_id,
                "body": text,
                "media_ids": media_ids,
            });
            let body = text.clone();
            let pid = post_id.clone();
            let attachments = media_ids.clone();
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_post_id(&pid)?;
                shortcuts::validate_post_body(&body)?;
                shortcuts::validate_media_attachments(&attachments)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let response = with_flags(
                client.reply_to_post(&post_id, &text, &media_ids),
                &common,
                None,
            )
            .send()
            .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Quote {
            post_id,
            text,
            common,
        } => {
            let ctx = json!({
                "command": "quote",
                "post_id": post_id,
                "body": text,
            });
            let body = text.clone();
            let pid = post_id.clone();
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_post_id(&pid)?;
                shortcuts::validate_post_body(&body)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let response = with_flags(client.quote_post(&post_id, &text), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Delete {
            post_id,
            force,
            common,
        } => {
            let ctx = json!({"command": "delete", "post_id": post_id});
            let pid = post_id.clone();
            // Force/confirmation gate runs BEFORE dry-run so an unconfirmed
            // destructive op in interactive mode does not leak a dry-run
            // envelope. Dry-run still composes with --force.
            match gate_destructive(
                force,
                no_interactive,
                quiet,
                &format!("Delete post {post_id}?"),
            )? {
                Gate::Proceed => {}
                Gate::Declined => return Ok(()),
                Gate::ConfirmationRequired => {
                    out.print_confirmation_required(stderr, &ctx, EXIT_GENERAL_ERROR);
                    return Err(Failure::Emitted {
                        exit_code: EXIT_GENERAL_ERROR,
                    });
                }
            }
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_post_id(&pid)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let response = with_flags(client.delete_post(&post_id), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }

        // ── Reading ──────────────────────────────────────────────────
        Commands::Read { post_id, common } => {
            let client = make_client(cfg, auth)?;
            let response = with_flags(client.read_post(&post_id), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Search {
            query,
            max_results,
            common,
        } => {
            let n = effective_limit(max_results, global_limit);
            let client = make_client(cfg, auth)?;
            let response = with_flags(client.search_posts(&query, n), &common, cursor_opt)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }

        // ── User Info ────────────────────────────────────────────────
        Commands::Whoami { common } => {
            let client = make_client(cfg, auth)?;
            let response = with_flags(client.get_me(), &common, None).send().await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::User {
            target_username,
            common,
        } => {
            let client = make_client(cfg, auth)?;
            let response = with_flags(client.lookup_user(&target_username), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }

        // ── Timeline & Mentions ──────────────────────────────────────
        Commands::Timeline {
            max_results,
            common,
        } => {
            let n = effective_limit(max_results, global_limit);
            let client = make_client(cfg, auth)?;
            let user_id = resolve_my_user_id(&client, &common).await?;
            let response = with_flags(client.get_timeline(&user_id, n), &common, cursor_opt)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Mentions {
            max_results,
            common,
        } => {
            let n = effective_limit(max_results, global_limit);
            let client = make_client(cfg, auth)?;
            let user_id = resolve_my_user_id(&client, &common).await?;
            let response = with_flags(client.get_mentions(&user_id, n), &common, cursor_opt)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }

        // ── Engagement ───────────────────────────────────────────────
        Commands::Like { post_id, common } => {
            let ctx = json!({"command": "like", "post_id": post_id});
            let pid = post_id.clone();
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_post_id(&pid)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let user_id = resolve_my_user_id(&client, &common).await?;
            let response = with_flags(client.like_post(&user_id, &post_id), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Unlike { post_id, common } => {
            let ctx = json!({"command": "unlike", "post_id": post_id});
            let pid = post_id.clone();
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_post_id(&pid)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let user_id = resolve_my_user_id(&client, &common).await?;
            let response = with_flags(client.unlike_post(&user_id, &post_id), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Repost { post_id, common } => {
            let ctx = json!({"command": "repost", "post_id": post_id});
            let pid = post_id.clone();
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_post_id(&pid)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let user_id = resolve_my_user_id(&client, &common).await?;
            let response = with_flags(client.repost(&user_id, &post_id), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Unrepost { post_id, common } => {
            let ctx = json!({"command": "unrepost", "post_id": post_id});
            let pid = post_id.clone();
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_post_id(&pid)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let user_id = resolve_my_user_id(&client, &common).await?;
            let response = with_flags(client.unrepost(&user_id, &post_id), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Bookmark { post_id, common } => {
            let ctx = json!({"command": "bookmark", "post_id": post_id});
            let pid = post_id.clone();
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_post_id(&pid)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let user_id = resolve_my_user_id(&client, &common).await?;
            let response = with_flags(client.bookmark(&user_id, &post_id), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Unbookmark { post_id, common } => {
            let ctx = json!({"command": "unbookmark", "post_id": post_id});
            let pid = post_id.clone();
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_post_id(&pid)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let user_id = resolve_my_user_id(&client, &common).await?;
            let response = with_flags(client.unbookmark(&user_id, &post_id), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Bookmarks {
            max_results,
            common,
        } => {
            let n = effective_limit(max_results, global_limit);
            let client = make_client(cfg, auth)?;
            let user_id = resolve_my_user_id(&client, &common).await?;
            let response = with_flags(client.get_bookmarks(&user_id, n), &common, cursor_opt)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Likes {
            max_results,
            common,
        } => {
            let n = effective_limit(max_results, global_limit);
            let client = make_client(cfg, auth)?;
            let user_id = resolve_my_user_id(&client, &common).await?;
            let response = with_flags(client.get_liked_posts(&user_id, n), &common, cursor_opt)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }

        // ── Social Graph ─────────────────────────────────────────────
        Commands::Follow {
            target_username,
            common,
        } => {
            let ctx = json!({"command": "follow", "target_username": target_username});
            let user = target_username.clone();
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_target_username(&user)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let my_id = resolve_my_user_id(&client, &common).await?;
            let target_id = resolve_user_id(&client, &target_username, &common).await?;
            let response = with_flags(client.follow_user(&my_id, &target_id), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Unfollow {
            target_username,
            common,
        } => {
            let ctx = json!({"command": "unfollow", "target_username": target_username});
            let user = target_username.clone();
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_target_username(&user)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let my_id = resolve_my_user_id(&client, &common).await?;
            let target_id = resolve_user_id(&client, &target_username, &common).await?;
            let response = with_flags(client.unfollow_user(&my_id, &target_id), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Following {
            max_results,
            of,
            common,
        } => {
            let n = effective_limit(max_results, global_limit);
            let client = make_client(cfg, auth)?;
            let user_id = if let Some(ref target) = of {
                resolve_user_id(&client, target, &common).await?
            } else {
                resolve_my_user_id(&client, &common).await?
            };
            let response = with_flags(client.get_following(&user_id, n), &common, cursor_opt)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Followers {
            max_results,
            of,
            common,
        } => {
            let n = effective_limit(max_results, global_limit);
            let client = make_client(cfg, auth)?;
            let user_id = if let Some(ref target) = of {
                resolve_user_id(&client, target, &common).await?
            } else {
                resolve_my_user_id(&client, &common).await?
            };
            let response = with_flags(client.get_followers(&user_id, n), &common, cursor_opt)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Mute {
            target_username,
            common,
        } => {
            let ctx = json!({"command": "mute", "target_username": target_username});
            let user = target_username.clone();
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_target_username(&user)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let my_id = resolve_my_user_id(&client, &common).await?;
            let target_id = resolve_user_id(&client, &target_username, &common).await?;
            let response = with_flags(client.mute_user(&my_id, &target_id), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Unmute {
            target_username,
            common,
        } => {
            let ctx = json!({"command": "unmute", "target_username": target_username});
            let user = target_username.clone();
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_target_username(&user)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let my_id = resolve_my_user_id(&client, &common).await?;
            let target_id = resolve_user_id(&client, &target_username, &common).await?;
            let response = with_flags(client.unmute_user(&my_id, &target_id), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Muted {
            max_results,
            common,
        } => {
            let n = effective_limit(max_results, global_limit);
            let client = make_client(cfg, auth)?;
            let user_id = resolve_my_user_id(&client, &common).await?;
            let response = with_flags(client.get_muted(&user_id, n), &common, cursor_opt)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Block {
            target_username,
            common,
        } => {
            let ctx = json!({"command": "block", "target_username": target_username});
            let user = target_username.clone();
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_target_username(&user)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let my_id = resolve_my_user_id(&client, &common).await?;
            let target_id = resolve_user_id(&client, &target_username, &common).await?;
            let response = with_flags(client.block_user(&my_id, &target_id), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Unblock {
            target_username,
            common,
        } => {
            let ctx = json!({"command": "unblock", "target_username": target_username});
            let user = target_username.clone();
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_target_username(&user)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let my_id = resolve_my_user_id(&client, &common).await?;
            let target_id = resolve_user_id(&client, &target_username, &common).await?;
            let response = with_flags(client.unblock_user(&my_id, &target_id), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Blocked {
            max_results,
            common,
        } => {
            let n = effective_limit(max_results, global_limit);
            let client = make_client(cfg, auth)?;
            let user_id = resolve_my_user_id(&client, &common).await?;
            let response = with_flags(client.get_blocked(&user_id, n), &common, cursor_opt)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }

        // ── Usage ─────────────────────────────────────────────────────
        Commands::Usage { target, common } => match target {
            Some(UsageCommands::Credits { common }) => {
                let client = make_client(cfg, auth)?;
                let response = with_flags(client.get_usage_credits(), &common, None)
                    .send()
                    .await?;
                print_typed(out, stdout, &response)?;
            }
            None => {
                let client = make_client(cfg, auth)?;
                let response = with_flags(client.get_usage(), &common, None).send().await?;
                print_typed(out, stdout, &response)?;
            }
        },

        // ── Direct Messages ──────────────────────────────────────────
        Commands::Dm {
            target_username,
            text,
            common,
        } => {
            let ctx = json!({
                "command": "dm",
                "target_username": target_username,
                "body": text,
            });
            let body = text.clone();
            let user = target_username.clone();
            let proceed = dry_run_or_validate(out, stdout, dry_run, ctx, || {
                shortcuts::validate_target_username(&user)?;
                shortcuts::validate_dm_body(&body)
            })?;
            if !proceed {
                return Ok(());
            }
            let client = make_client(cfg, auth)?;
            let target_id = resolve_user_id(&client, &target_username, &common).await?;
            let response = with_flags(client.send_dm(&target_id, &text), &common, None)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }
        Commands::Dms {
            max_results,
            common,
        } => {
            let n = effective_limit(max_results, global_limit);
            let client = make_client(cfg, auth)?;
            let response = with_flags(client.get_dm_events(n), &common, cursor_opt)
                .send()
                .await?;
            print_typed(out, stdout, &response)?;
        }

        // ── Auth ─────────────────────────────────────────────────────
        Commands::Auth { command } => {
            return auth::run_auth_command(
                command,
                auth,
                cfg,
                auth::AuthGlobalFlags {
                    no_interactive,
                    verbose,
                    dry_run,
                    quiet,
                    app_explicit,
                },
                out,
                stdout,
                stderr,
            )
            .await;
        }

        // ── Media ────────────────────────────────────────────────────
        Commands::Media { command } => {
            return media::run_media_command(command, cfg, auth, verbose, dry_run, out, stdout)
                .await
                .map_err(Failure::from);
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
        Commands::Examples => {
            unreachable!("examples is handled before config init in main()")
        }
        Commands::Validate { .. } => {
            unreachable!("validate is handled before config init in runner")
        }
    }
    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Applies the shared per-command flags to a call: `--auth`, `--username`,
/// `--trace`, and the resolved cursor.
fn with_flags<T: DeserializeOwned>(
    call: Call<T>,
    common: &CommonFlags,
    cursor: Option<&str>,
) -> Call<T> {
    let mut call = call.trace(common.trace);
    if let Some(auth_type) = &common.auth_type {
        call = call.auth_wire(auth_type.clone());
    }
    if let Some(username) = &common.username {
        call = call.username(username.clone());
    }
    if let Some(cursor) = cursor {
        call = call.pagination_token(cursor);
    }
    call
}

/// Resolves the authenticated user's ID.
///
/// With no `--username`, calls `/2/users/me` (the default identity for the
/// active credential). With one, calls `/2/users/by/username/<u>` directly,
/// bypassing `/me` so the shortcut works when `/me` is unavailable or when
/// the caller wants to act under a known handle without consulting `/me`.
async fn resolve_my_user_id(client: &Client, common: &CommonFlags) -> Result<String> {
    let id = match common.username.as_deref().filter(|u| !u.is_empty()) {
        None => {
            with_flags(client.get_me(), common, None)
                .send()
                .await?
                .data
                .id
        }
        Some(username) => {
            with_flags(client.lookup_user(username), common, None)
                .send()
                .await?
                .data
                .id
        }
    };
    if id.is_empty() {
        return Err(Error::auth("user ID was empty -- check your auth tokens"));
    }
    Ok(id)
}

/// Resolves a username to a user ID.
async fn resolve_user_id(client: &Client, username: &str, common: &CommonFlags) -> Result<String> {
    let resp = with_flags(client.lookup_user(username), common, None)
        .send()
        .await?;
    let id = &resp.data.id;
    if id.is_empty() {
        let clean = username.trim_start_matches('@');
        return Err(Error::validation(format!("user @{clean} not found")));
    }
    Ok(id.clone())
}
