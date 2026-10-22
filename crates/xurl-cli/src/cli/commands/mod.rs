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
mod webhooks;
mod webhooks_listen;

use std::io::{IsTerminal, Write};

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;

use crate::cli::envelope::Reason;
use crate::cli::failure::{CommandResult, Failure};
use crate::cli::output::OutputConfig;
use crate::cli::{Cli, Commands, CommonFlags};
use xdk::api::auth_matrix::WireScheme;
use xdk::api::shortcuts;
use xdk::api::{self, AuthPreflight, Call, Client, RequestOptions, RequestTarget};
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

/// The dry-run context of a destructive command: `ctx`, with
/// `confirmation_required` set when the invocation carries no `--force`.
///
/// `--dry-run` is answered before the confirmation gate, since nothing can be
/// destroyed under it. The key tells the caller that the same invocation
/// without the flag would stop to ask.
pub(super) fn destructive_dry_run_context(
    mut ctx: serde_json::Value,
    force: bool,
) -> serde_json::Value {
    if !force {
        ctx["confirmation_required"] = json!(true);
    }
    ctx
}

/// Gates a destructive op on `--force` / TTY confirmation.
///
/// Callers answer `--dry-run` before they reach this gate.
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

/// The id a dry run puts where the real run would have looked one up.
/// Scheme selection reads a request's method and path template, never the
/// id the path carries.
const UNRESOLVED_ID: &str = "0";

/// One request's credential, read without sending the request.
type Credential = Result<Option<AuthPreflight>>;

/// A dry run under way: the context its envelope carries, and the reason
/// its inputs were refused, when they were.
struct DryRun {
    ctx: serde_json::Value,
    invalid: Option<&'static str>,
}

impl DryRun {
    /// The dry run of a command with no inputs to check offline.
    fn new(ctx: serde_json::Value) -> Self {
        Self { ctx, invalid: None }
    }

    /// Prints the answer. `would_succeed` is whether the inputs were valid
    /// and every request the command would send finds a credential.
    ///
    /// `credentials` holds one entry per request, in the order the real run
    /// sends them. The last is the request that does the command's work,
    /// and its credential is the one the `auth` key reports. Whether X
    /// still honors that credential is left to a real request.
    fn answer(
        self,
        out: &OutputConfig,
        stdout: &mut dyn Write,
        credentials: impl IntoIterator<Item = Credential>,
    ) {
        let Self { mut ctx, invalid } = self;
        if let Some(reason) = invalid {
            ctx["reason"] = json!(reason);
            out.print_dry_run(stdout, false, EXIT_GENERAL_ERROR, &ctx);
            return;
        }
        let mut reported = None;
        for credential in credentials {
            match credential {
                Ok(found) => reported = found,
                Err(error) => {
                    out.print_dry_run_refusal(stdout, &ctx, &error);
                    return;
                }
            }
        }
        if let Some(found) = reported {
            ctx["auth"] = credential_value(&found);
        }
        out.print_dry_run(stdout, true, 0, &ctx);
    }
}

/// The `auth` object of a dry-run envelope. `app` is the app whose stored
/// credential would be sent, absent for a bearer token the environment
/// supplies. `username` and `token_expired` describe an `OAuth2` login, the
/// one credential that has either.
fn credential_value(found: &AuthPreflight) -> serde_json::Value {
    let mut auth = json!({"scheme": found.scheme.as_wire()});
    if let Some(app) = &found.app {
        auth["app"] = json!(app);
    }
    if found.scheme == WireScheme::OAuth2 {
        if let Some(username) = &found.username {
            auth["username"] = json!(username);
        }
        auth["token_expired"] = json!(found.token_expired);
    }
    auth
}

/// Checks a write's inputs: `validator` answers `Ok(())` or a kebab-case
/// reason.
///
/// A real run with invalid inputs is refused here and one with valid inputs
/// gets `None`. A dry run gets its [`DryRun`] either way, and carries a
/// refusal to its answer as `would_succeed: false` with the reason.
fn dry_run_or_validate(
    dry_run: bool,
    ctx: serde_json::Value,
    validator: impl FnOnce() -> std::result::Result<(), &'static str>,
) -> CommandResult<Option<DryRun>> {
    let invalid = validator().err();
    if dry_run {
        return Ok(Some(DryRun { ctx, invalid }));
    }
    match invalid {
        Some(reason) => Err(Failure::refused(Error::validation(reason.to_string()))),
        None => Ok(None),
    }
}

/// Sends `call` and prints its typed response. Under `--dry-run` nothing is
/// sent, and the answer names the credential the call would go out with.
async fn send_or_report<T>(
    out: &OutputConfig,
    stdout: &mut dyn Write,
    dry_run: Option<DryRun>,
    call: Call<T>,
) -> Result<()>
where
    T: Serialize + DeserializeOwned,
{
    if let Some(dry_run) = dry_run {
        dry_run.answer(out, stdout, [call.auth_preflight().await]);
        return Ok(());
    }
    print_typed(out, stdout, &call.send().await?)
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
        return Err(Failure::Refused {
            error: Error::validation(
                "X API does not support offset-style pagination; pass --cursor <token> from the previous response's meta.next_token instead.",
            ),
            reason: Some(Reason::UnsupportedPagination),
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
        None => run_raw_mode(&cli, &cfg, auth, out, stdout, stderr).await,
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
) -> CommandResult<()> {
    let url = if let Some(u) = &cli.url {
        u.clone()
    } else {
        return Err(Failure::refused(Error::validation(
            "No URL provided. Usage: xr [OPTIONS] [URL] [COMMAND].",
        )));
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
        return Err(Failure::refused(Error::validation(format!(
            "URL {url:?} must be an absolute http(s) URL or an absolute path starting with `/`."
        ))));
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

    // A raw request is whatever the caller wrote, a write as readily as a
    // read, so under `--dry-run` none is sent.
    if cli.dry_run {
        let ctx = json!({"method": options.method, "url": url});
        DryRun::new(ctx).answer(out, stdout, [client.auth_preflight(&options).await]);
        return Ok(());
    }

    // Check for media append request
    if api::is_media_append_request(&absolute_url, &media_file) {
        let response = api::handle_media_append_request(&options, &media_file, &client).await?;
        out.print_response(stdout, &response);
        return Ok(());
    }

    let should_stream = cli.stream || api::is_streaming_endpoint(&absolute_url);

    if should_stream {
        streaming::stream_request_with_output(&client, &options, out, stdout, stderr).await?;
        Ok(())
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
        Commands::Webhooks { action } => webhooks::run(action, run).await,
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

/// The request that resolves the caller's id.
///
/// With no `--username` it is `/2/users/me`, the default identity for the
/// active credential. With one it is `/2/users/by/username/<u>`, so the
/// shortcut works when `/me` is unavailable or when the caller wants to act
/// under a known handle without consulting `/me`.
fn my_id_call(client: &Client, common: &CommonFlags) -> Call<api::ApiResponse<api::User>> {
    let call = match common.username.as_deref().filter(|u| !u.is_empty()) {
        None => client.get_me(),
        Some(username) => client.lookup_user(username),
    };
    with_flags(call, common, None)
}

/// Resolves the authenticated user's ID.
///
/// The store holds it beside the login once `/2/users/me` has answered
/// under that login, and an account's id never changes, so a stored id is
/// used as it is and no request is sent. Without one the lookup is sent, and
/// its answer is stored for the next command.
async fn resolve_my_user_id(client: &Client, common: &CommonFlags) -> Result<String> {
    let call = my_id_call(client, common);
    if let Ok(Some(found)) = call.auth_preflight().await
        && let Some(id) = found.user_id
    {
        return Ok(id);
    }
    let id = call.send_saving_identity().await?.data.id;
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
    command: &str,
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
    if flags.dry_run {
        let mut ctx = json!({"command": command});
        if let Some(n) = max_results {
            ctx["max_results"] = json!(n);
        }
        if let Some(target) = of {
            ctx["of"] = json!(target);
        }
        let owner = match of {
            Some(target) => user_id_call(&client, target, common),
            None => my_id_call(&client, common),
        };
        let list = with_flags(
            shortcut(&client, UNRESOLVED_ID, n),
            common,
            flags.cursor.as_deref(),
        );
        let credentials = [owner.auth_preflight().await, list.auth_preflight().await];
        DryRun::new(ctx).answer(out, stdout, credentials);
        return Ok(());
    }
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
) -> CommandResult<()>
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
    let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
        shortcuts::validate_target_username(target_username)
    })?;
    let client = make_client(cfg, auth)?;
    if let Some(dry_run) = dry_run {
        let act = shortcut(&client, UNRESOLVED_ID, UNRESOLVED_ID);
        let credentials = [
            my_id_call(&client, common).auth_preflight().await,
            user_id_call(&client, target_username, common)
                .auth_preflight()
                .await,
            with_flags(act, common, None).auth_preflight().await,
        ];
        dry_run.answer(out, stdout, credentials);
        return Ok(());
    }
    let my_id = resolve_my_user_id(&client, common).await?;
    let target_id = resolve_user_id(&client, target_username, common).await?;
    let response = with_flags(shortcut(&client, &my_id, &target_id), common, None)
        .send()
        .await?;
    print_typed(out, stdout, &response)?;
    Ok(())
}

/// A verb that acts on another user without naming the caller: gate on the
/// handle, resolve it, call one shortcut, and print the typed response.
async fn act_on_user<T>(
    run: Run<'_>,
    command: &str,
    target_username: &str,
    common: &CommonFlags,
    shortcut: impl FnOnce(&Client, &str) -> Call<api::ApiResponse<T>>,
) -> CommandResult<()>
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
    let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
        shortcuts::validate_target_username(target_username)
    })?;
    let client = make_client(cfg, auth)?;
    if let Some(dry_run) = dry_run {
        let act = shortcut(&client, UNRESOLVED_ID);
        let credentials = [
            user_id_call(&client, target_username, common)
                .auth_preflight()
                .await,
            with_flags(act, common, None).auth_preflight().await,
        ];
        dry_run.answer(out, stdout, credentials);
        return Ok(());
    }
    let target_id = resolve_user_id(&client, target_username, common).await?;
    let response = with_flags(shortcut(&client, &target_id), common, None)
        .send()
        .await?;
    print_typed(out, stdout, &response)?;
    Ok(())
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
) -> CommandResult<()>
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
    let dry_run = dry_run_or_validate(flags.dry_run, ctx, || shortcuts::validate_post_id(post_id))?;
    let client = make_client(cfg, auth)?;
    if let Some(dry_run) = dry_run {
        let act = shortcut(&client, UNRESOLVED_ID, post_id);
        let credentials = [
            my_id_call(&client, common).auth_preflight().await,
            with_flags(act, common, None).auth_preflight().await,
        ];
        dry_run.answer(out, stdout, credentials);
        return Ok(());
    }
    let my_id = resolve_my_user_id(&client, common).await?;
    let response = with_flags(shortcut(&client, &my_id, post_id), common, None)
        .send()
        .await?;
    print_typed(out, stdout, &response)?;
    Ok(())
}

/// The request that resolves a username to a user id.
fn user_id_call(
    client: &Client,
    username: &str,
    common: &CommonFlags,
) -> Call<api::ApiResponse<api::User>> {
    with_flags(client.lookup_user(username), common, None)
}

/// Resolves a username to a user ID.
async fn resolve_user_id(client: &Client, username: &str, common: &CommonFlags) -> Result<String> {
    let resp = user_id_call(client, username, common).send().await?;
    let id = &resp.data.id;
    if id.is_empty() {
        let clean = username.trim_start_matches('@');
        return Err(Error::validation(format!("user @{clean} not found")));
    }
    Ok(id.clone())
}
