//! Command execution — dispatches CLI commands to API functions.

mod auth;
mod broadcasts;
mod dms;
mod engagement;
pub mod examples;
mod graph;
mod media;
mod posts;
mod reads;
pub mod schema;
pub mod skill;
mod streaming;
mod usage;
pub mod validate;

use std::io::{IsTerminal, Write};

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;

use crate::cli::envelope::Reason;
use crate::cli::failure::{CommandResult, Failure};
use crate::cli::output::OutputConfig;
use crate::cli::{Cli, Commands, CommonFlags};
use xdk::api::shortcuts;
use xdk::api::{self, Call, Client, RequestOptions, RequestTarget};
use xdk::auth::Auth;
use xdk::config::Config;
use xdk::error::{EXIT_GENERAL_ERROR, Error, Result};

/// Default page size when neither `--max-results` nor `--limit` is supplied.
const DEFAULT_PAGE_SIZE: i32 = 10;

/// Resolves the effective result limit.
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
/// Rules:
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
    cfg.rate_limit_max_wait = cli
        .wait_on_rate_limit
        .then(|| std::time::Duration::from_secs(cli.rate_limit_max_wait));

    // Capture whether the user passed `--app` BEFORE `cli.app` is
    // collapsed into `auth.with_app_name(...)` below. The collapsed
    // `Config.app_name` is always `"default"` in normal runtime paths and
    // therefore cannot distinguish "user passed --app default" from "user
    // passed nothing"; the boolean derived here threads through to
    // the auth commands so the credential-less-default warning gates
    // correctly.
    let app_explicit = cli.app.is_some();

    // Apply --app override
    if let Some(ref app_name) = cli.app {
        auth.with_app_name(app_name);
    }

    // Resolve cursor from --cursor or --after. --page is rejected upstream
    // because the X API does not offer offset-style pagination.
    if cli.page.is_some() {
        out.print_error_envelope(
            stderr,
            Reason::UnsupportedPagination,
            EXIT_GENERAL_ERROR,
            "X API does not support offset-style pagination; pass --cursor <token> from the previous response's meta.next_token instead.",
        );
        return Err(Failure::Emitted {
            exit_code: EXIT_GENERAL_ERROR,
        });
    }
    let flags = GlobalFlags {
        no_interactive: cli.no_interactive,
        verbose: cli.verbose,
        dry_run: cli.dry_run,
        global_limit: cli.limit,
        quiet: cli.quiet,
        app_explicit,
        cursor: cli
            .cursor
            .clone()
            .or_else(|| cli.after.clone())
            .filter(|token| !token.is_empty()),
    };

    match cli.command {
        Some(cmd) => {
            let run = Run {
                cfg: &cfg,
                auth,
                flags,
                out,
                stdout,
                stderr,
            };
            run_subcommand(cmd, run).await
        }
        None => run_raw_mode(&cli, &cfg, auth, out, stdout, stderr)
            .await
            .map_err(Failure::from),
    }
}

/// The global flags every subcommand needs.
#[derive(Debug, Clone)]
struct GlobalFlags {
    no_interactive: bool,
    verbose: bool,
    dry_run: bool,
    global_limit: Option<i32>,
    quiet: bool,
    app_explicit: bool,
    /// The `pagination_token` from `--cursor` or `--after`, sent with every
    /// list call; `None` when neither flag gave one.
    cursor: Option<String>,
}

/// What a command runs with: the configuration and credentials the run
/// resolved, its global flags, and where its output goes.
struct Run<'a> {
    cfg: &'a Config,
    auth: Auth,
    flags: GlobalFlags,
    out: &'a OutputConfig,
    stdout: &'a mut dyn Write,
    stderr: &'a mut dyn Write,
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
            "No URL provided. Usage: xr [OPTIONS] [URL] [COMMAND]. Try 'xr --help'.",
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
        // A body that is not JSON arrives as a string holding it. Text mode
        // prints those bytes as they are; a structured mode still prints
        // JSON, so it keeps the string form.
        match &response {
            serde_json::Value::String(text) if !out.format.is_structured() => {
                out.print_message(stdout, text);
            }
            _ => out.print_response(stdout, &response),
        }
        Ok(())
    }
}

/// Routes a subcommand to the group that owns it. Each group's module
/// matches the variants its line names here.
async fn run_subcommand(cmd: Commands, run: Run<'_>) -> CommandResult<()> {
    match cmd {
        Commands::Post { .. }
        | Commands::Reply { .. }
        | Commands::Quote { .. }
        | Commands::Delete { .. } => posts::run(cmd, run).await,
        Commands::Read { .. }
        | Commands::Search { .. }
        | Commands::Whoami { .. }
        | Commands::User { .. }
        | Commands::Timeline { .. }
        | Commands::Mentions { .. } => reads::run(cmd, run).await,
        Commands::Like { .. }
        | Commands::Unlike { .. }
        | Commands::Repost { .. }
        | Commands::Unrepost { .. }
        | Commands::Bookmark { .. }
        | Commands::Unbookmark { .. }
        | Commands::Bookmarks { .. }
        | Commands::Likes { .. } => engagement::run(cmd, run).await,
        Commands::Follow { .. }
        | Commands::Unfollow { .. }
        | Commands::Following { .. }
        | Commands::Followers { .. }
        | Commands::Mute { .. }
        | Commands::Unmute { .. }
        | Commands::Muted { .. }
        | Commands::Block { .. }
        | Commands::Unblock { .. }
        | Commands::Blocked { .. } => graph::run(cmd, run).await,
        Commands::Dm { .. } | Commands::Dms { .. } => dms::run(cmd, run).await,
        Commands::Usage { target, common } => usage::run(target, &common, run).await,
        Commands::Broadcasts { target } => broadcasts::run(target, run).await,
        Commands::Auth { command } => auth::run(command, run).await,
        Commands::Media { command } => media::run(command, run).await,
        Commands::Schema { .. }
        | Commands::Completions { .. }
        | Commands::Version
        | Commands::Skill { .. }
        | Commands::Examples
        | Commands::Validate { .. } => {
            unreachable!("the runner handles the tooling commands before it loads configuration")
        }
    }
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
        call = call.auth_wire_unchecked(auth_type.clone());
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

/// A paged read of a list that belongs to one user: resolve the user (the
/// handle `of` names, or the caller), call one shortcut with that id and the
/// page size, and print the typed response.
async fn list_for_user<T>(
    run: Run<'_>,
    max_results: Option<i32>,
    of: Option<&str>,
    common: &CommonFlags,
    shortcut: impl FnOnce(&Client, &str, i32) -> Call<api::ApiResponse<T>>,
) -> Result<()>
where
    T: Serialize + DeserializeOwned + Default,
{
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    let n = effective_limit(max_results, flags.global_limit);
    let client = make_client(cfg, auth)?;
    let user_id = match of {
        Some(target) => resolve_user_id(&client, target, common).await?,
        None => resolve_my_user_id(&client, common).await?,
    };
    let call = shortcut(&client, &user_id, n);
    let response = with_flags(call, common, flags.cursor.as_deref())
        .send()
        .await?;
    print_typed(out, stdout, &response)
}

/// A verb that acts on another user from the caller's account: gate on the
/// handle, resolve the caller's id and the target's, call one shortcut with
/// both, and print the typed response.
async fn act_from_me_on_user<T>(
    run: Run<'_>,
    command: &str,
    target_username: &str,
    common: &CommonFlags,
    shortcut: impl FnOnce(&Client, &str, &str) -> Call<api::ApiResponse<T>>,
) -> Result<()>
where
    T: Serialize + DeserializeOwned + Default,
{
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    let ctx = json!({"command": command, "target_username": target_username});
    let proceed = dry_run_or_validate(out, stdout, flags.dry_run, ctx, || {
        shortcuts::validate_target_username(target_username)
    })?;
    if !proceed {
        return Ok(());
    }
    let client = make_client(cfg, auth)?;
    let my_id = resolve_my_user_id(&client, common).await?;
    let target_id = resolve_user_id(&client, target_username, common).await?;
    let response = with_flags(shortcut(&client, &my_id, &target_id), common, None)
        .send()
        .await?;
    print_typed(out, stdout, &response)
}

/// A verb that acts on another user without naming the caller: gate on the
/// handle, resolve it, call one shortcut, and print the typed response.
async fn act_on_user<T>(
    run: Run<'_>,
    command: &str,
    target_username: &str,
    common: &CommonFlags,
    shortcut: impl FnOnce(&Client, &str) -> Call<api::ApiResponse<T>>,
) -> Result<()>
where
    T: Serialize + DeserializeOwned + Default,
{
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    let ctx = json!({"command": command, "target_username": target_username});
    let proceed = dry_run_or_validate(out, stdout, flags.dry_run, ctx, || {
        shortcuts::validate_target_username(target_username)
    })?;
    if !proceed {
        return Ok(());
    }
    let client = make_client(cfg, auth)?;
    let target_id = resolve_user_id(&client, target_username, common).await?;
    let response = with_flags(shortcut(&client, &target_id), common, None)
        .send()
        .await?;
    print_typed(out, stdout, &response)
}

/// A verb that acts on a post from the caller's account: gate on the post
/// id, resolve the caller's id, call one shortcut, and print the typed
/// response.
async fn act_from_me_on_post<T>(
    run: Run<'_>,
    command: &str,
    post_id: &str,
    common: &CommonFlags,
    shortcut: impl FnOnce(&Client, &str, &str) -> Call<api::ApiResponse<T>>,
) -> Result<()>
where
    T: Serialize + DeserializeOwned + Default,
{
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    let ctx = json!({"command": command, "post_id": post_id});
    let proceed = dry_run_or_validate(out, stdout, flags.dry_run, ctx, || {
        shortcuts::validate_post_id(post_id)
    })?;
    if !proceed {
        return Ok(());
    }
    let client = make_client(cfg, auth)?;
    let my_id = resolve_my_user_id(&client, common).await?;
    let response = with_flags(shortcut(&client, &my_id, post_id), common, None)
        .send()
        .await?;
    print_typed(out, stdout, &response)
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
