//! The steps errors carry: a usage error points at the help of the command
//! it belongs to, a store that could not be loaded points at the page that
//! says how to recover it, and the README's table of steps agrees with the
//! closed sets and with what the golden fixtures show.
//!
//! Parallel-safe by construction, like `tests/cli_tests.rs`: every case runs
//! the library entrypoint against its own `TempDir`-rooted store and supplies
//! its environment as `EnvOverrides`, so nothing here reads or writes the
//! process environment.

mod common;

use std::path::Path;

use tempfile::TempDir;
use xdk::config::EnvOverrides;
use xdk::store::TokenStore;
use xurl::cli;

/// The page the `inspect-store` step names.
const STORE_DOCS: &str = "https://github.com/brettdavies/xurl-rs/blob/main/crates/xurl-cli/README.md#token-store-could-not-be-read";
#[cfg(unix)]
const STORE_WRITE_DOCS: &str = "https://github.com/brettdavies/xurl-rs/blob/main/crates/xurl-cli/README.md#token-store-could-not-be-written";

async fn run_at(store: &Path, args: &[&str]) -> (i32, String, String) {
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let code = cli::runner::run_with_overrides(
        args,
        &mut stdout,
        &mut stderr,
        store,
        &EnvOverrides::default(),
    )
    .await;
    (
        code,
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

/// Runs against a store path nothing has written.
async fn run_isolated(args: &[&str]) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    run_at(&tmp.path().join(".xurl"), args).await
}

fn envelope(stderr: &str) -> serde_json::Value {
    serde_json::from_str(stderr.trim())
        .unwrap_or_else(|e| panic!("stderr is not a JSON envelope ({e}): {stderr}"))
}

/// A store file that exists and is not a store.
fn unloadable_store(tmp: &TempDir) -> std::path::PathBuf {
    let store = tmp.path().join(".xurl");
    std::fs::write(&store, ":::: not yaml [\n").expect("write the broken store");
    store
}

// ── invalid-args: show-help ────────────────────────────────────────────

/// Whoever raised the usage error (clap, the classifier, or a secret flag's
/// own check), its envelope carries the help invocation its message names.
#[rstest::rstest]
#[case::root_flag(&["xr", "--output", "json", "--frobnicate"], "xr --help")]
#[case::command_argument(&["xr", "--output", "json", "whoami", "extra"], "xr whoami --help")]
#[case::word_after_a_raw_target(&["xr", "--output", "json", "/2/users/me", "extra"], "xr --help")]
#[case::word_after_a_raw_only_flag(&["xr", "--output", "json", "-X", "POST", "suth", "status"], "xr --help")]
#[case::family_flag(&["xr", "--output", "json", "auth", "--frob"], "xr auth --help")]
#[case::invalid_value(&["xr", "--output", "json", "skill", "install", "bogus_host"], "xr skill install --help")]
#[case::unsupported_format(&["xr", "--output", "toml", "whoami"], "xr whoami --help")]
#[case::no_command(&["xr", "--output", "json"], "xr --help")]
#[case::two_secrets_on_stdin(
    &["xr", "--output", "json", "auth", "oauth1", "--consumer-key", "ck", "--consumer-secret-file", "-",
      "--access-token-file", "-", "--token-secret-file", "-"],
    "xr auth oauth1 --help"
)]
#[tokio::test]
async fn a_usage_error_carries_the_help_to_read(#[case] args: &[&str], #[case] help: &str) {
    let (code, _stdout, stderr) = run_isolated(args).await;
    assert_eq!(code, 2, "args {args:?}; stderr: {stderr}");
    let v = envelope(&stderr);
    assert_eq!(v["reason"], "invalid-args", "args {args:?}: {v}");
    assert_eq!(
        v["next_step"],
        serde_json::json!({"action": "show-help", "command": help}),
        "args {args:?}: {v}"
    );
    let message = v["message"].as_str().expect("the message is a string");
    assert!(
        message.ends_with(&format!("Try '{help}'.")),
        "the step and the message name one command: {v}"
    );
}

// ── token-store: inspect-store ─────────────────────────────────────────

/// A store that exists and could not be loaded is the case `inspect-store`
/// names. The step carries the page and no command: the one command that
/// reports on the store is the one that just failed.
#[rstest::rstest]
#[case::status(&["xr", "--output", "json", "auth", "status"])]
#[case::apps_list(&["xr", "--output", "json", "auth", "apps", "list"])]
#[case::apps_add(&["xr", "--output", "json", "auth", "apps", "add", "work", "--client-id", "abcdefgh", "--client-secret-file"])]
#[case::clear_all(&["xr", "--output", "json", "auth", "clear", "--all", "--force"])]
#[case::clear_oauth1(&["xr", "--output", "json", "auth", "clear", "--oauth1", "--force"])]
#[case::clear_bearer(&["xr", "--output", "json", "auth", "clear", "--bearer", "--force"])]
#[case::clear_oauth2(&["xr", "--output", "json", "auth", "clear", "--oauth2-username", "alice", "--force"])]
#[tokio::test]
async fn a_store_that_could_not_be_loaded_carries_inspect_store(#[case] args: &[&str]) {
    let tmp = TempDir::new().expect("tempdir");
    let store = unloadable_store(&tmp);
    // The one case that ends on a flag takes its secret from a file.
    let secret = tmp.path().join("client-secret");
    std::fs::write(&secret, "shh\n").expect("write the secret");
    let mut args = args.to_vec();
    if args.last() == Some(&"--client-secret-file") {
        args.push(secret.to_str().expect("utf-8 path"));
    }
    let before = std::fs::read(&store).expect("read the broken store");
    let (code, stdout, stderr) = run_at(&store, &args).await;
    assert_eq!(code, 77, "args {args:?}; stderr: {stderr}");
    assert_eq!(stdout, "", "args {args:?}: nothing is reported as done");
    assert_eq!(
        std::fs::read(&store).expect("read the broken store"),
        before,
        "args {args:?}: the file is left as it was"
    );
    let v = envelope(&stderr);
    assert_eq!(v["reason"], "token-store", "args {args:?}: {v}");
    assert_eq!(
        v["next_step"],
        serde_json::json!({"action": "inspect-store", "docs": STORE_DOCS}),
        "args {args:?}: {v}"
    );
    assert!(
        v["message"]
            .as_str()
            .expect("the message is a string")
            .contains(store.to_str().expect("utf-8 path")),
        "the message names the file: {v}"
    );
}

/// A store whose directory cannot be created is a filesystem refusal like a
/// directory that takes no new file: `io`, with the page on an unwritable
/// store, and a message naming the directory and both ways out.
#[cfg(unix)]
#[tokio::test]
async fn a_store_whose_directory_cannot_be_created_is_io_with_the_page() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().join("home");
    std::fs::create_dir(&home).expect("home");
    let store = home.join(".xurl").join("auth.yml");
    let secret = tmp.path().join("client-secret");
    std::fs::write(&secret, "shh\n").expect("write the secret");
    std::fs::set_permissions(&home, std::fs::Permissions::from_mode(0o500)).expect("chmod");
    // A process that may write a read-only directory (root) cannot be
    // refused this way.
    let writable = std::fs::write(home.join("probe"), b"").is_ok();

    let (code, _stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "--output",
            "json",
            "auth",
            "apps",
            "add",
            "work",
            "--client-id",
            "abcdefgh",
            "--client-secret-file",
            secret.to_str().expect("utf-8 path"),
        ],
    )
    .await;
    std::fs::set_permissions(&home, std::fs::Permissions::from_mode(0o700)).expect("chmod");
    if writable {
        return;
    }

    assert_eq!(code, 5, "stderr: {stderr}");
    let v = envelope(&stderr);
    assert_eq!(v["reason"], "io", "{v}");
    assert_eq!(
        v["next_step"],
        serde_json::json!({"action": "fix-input", "docs": STORE_WRITE_DOCS}),
        "{v}"
    );
    let message = v["message"].as_str().unwrap_or_default();
    let dir = home.join(".xurl");
    assert!(
        message.contains(&dir.display().to_string()) && message.contains("XURL_TOKEN_STORE"),
        "the message names the directory and both ways out: {v}"
    );
}

/// `token-store` is also the reason for a name the store does not hold. The
/// file loaded, so there is nothing to inspect: the step lists the names it
/// does hold.
#[tokio::test]
async fn a_name_the_store_does_not_hold_points_at_the_apps_it_holds() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.add_app("work", "abcdefgh", "shh").expect("add app");

    let (code, _stdout, stderr) = run_at(
        &store,
        &["xr", "--output", "json", "auth", "default", "nope"],
    )
    .await;
    assert_eq!(code, 77, "stderr: {stderr}");
    let v = envelope(&stderr);
    assert_eq!(v["reason"], "token-store", "{v}");
    assert_eq!(
        v["next_step"],
        serde_json::json!({"action": "fix-input", "command": "xr auth status"}),
        "{v}"
    );
}

/// A command that needs a credential from the unloadable store keeps the step
/// it had, a runnable `xr auth status`, and gains the page.
#[tokio::test]
async fn a_credential_lookup_on_an_unloadable_store_keeps_its_command() {
    let tmp = TempDir::new().expect("tempdir");
    let store = unloadable_store(&tmp);
    let (code, _stdout, stderr) = run_at(&store, &["xr", "--output", "json", "whoami"]).await;
    assert_eq!(code, 77, "stderr: {stderr}");
    let v = envelope(&stderr);
    assert_eq!(v["reason"], "auth-required", "{v}");
    assert_eq!(
        v["next_step"],
        serde_json::json!({"action": "inspect-store", "command": "xr auth status", "docs": STORE_DOCS}),
        "{v}"
    );
}

/// Text mode says where to read, after the error line.
#[tokio::test]
async fn text_mode_names_the_page_for_an_unloadable_store() {
    let tmp = TempDir::new().expect("tempdir");
    let store = unloadable_store(&tmp);
    let (code, _stdout, stderr) = run_at(&store, &["xr", "auth", "status"]).await;
    assert_eq!(code, 77, "stderr: {stderr}");
    assert!(stderr.contains(STORE_DOCS), "stderr: {stderr}");
}

// ── A store write's dry run ────────────────────────────────────────────

/// What a store-write case runs against.
#[derive(Clone, Copy)]
enum Held {
    /// A file that exists and is not a store.
    Unloadable,
    /// A store holding the one app `work`.
    Work,
    /// No store file yet: its directory exists and holds nothing.
    Empty,
    /// A store holding `work`, in a directory that takes no new file.
    #[cfg(unix)]
    ReadOnly,
}

/// Seeds `tmp` with a store as `held` describes and a secret file.
fn seed_store_write(held: Held, tmp: &TempDir) {
    std::fs::create_dir(tmp.path().join("home")).expect("home");
    let store = tmp.path().join("home").join(".xurl");
    match held {
        Held::Unloadable => {
            std::fs::write(&store, ":::: not yaml [\n").expect("write the broken store")
        }
        Held::Empty => {}
        _ => {
            let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
            ts.add_app("work", "abcdefgh", "shh").expect("add app");
        }
    }
    std::fs::write(tmp.path().join("secret"), "shh\n").expect("write the secret");
}

/// `xr --output json [--dry-run] <args>` on the store [`seed_store_write`]
/// left in `tmp`, with each `SECRET` in `args` read from its file: the exit
/// code, the document the run printed, and whether the store file is as it
/// was.
async fn store_write(
    #[cfg_attr(not(unix), allow(unused_variables))] held: Held,
    tmp: &TempDir,
    args: &[&str],
    dry_run: bool,
) -> (i32, serde_json::Value, bool) {
    let store = tmp.path().join("home").join(".xurl");
    let secret = tmp.path().join("secret");
    let secret = secret.to_str().expect("utf-8 path");
    let mut argv = vec!["xr", "--output", "json"];
    if dry_run {
        argv.push("--dry-run");
    }
    argv.extend(
        args.iter()
            .map(|arg| if *arg == "SECRET" { secret } else { arg }),
    );
    let before = std::fs::read(&store).ok();
    #[cfg(unix)]
    let home_mode = |mode| {
        use std::os::unix::fs::PermissionsExt;
        if matches!(held, Held::ReadOnly) {
            std::fs::set_permissions(
                tmp.path().join("home"),
                std::fs::Permissions::from_mode(mode),
            )
            .expect("chmod");
        }
    };
    #[cfg(unix)]
    home_mode(0o500);
    let (code, stdout, stderr) = run_at(&store, &argv).await;
    #[cfg(unix)]
    home_mode(0o700);
    let unchanged = std::fs::read(&store).ok() == before;
    (
        code,
        envelope(if dry_run { &stdout } else { &stderr }),
        unchanged,
    )
}

/// A dry run of a command that writes the store answers what its real run
/// fails with: `would_succeed: false` with the same reason, exit code,
/// message, and step. It writes nothing.
#[rstest::rstest]
#[case::app_unloadable(Held::Unloadable, &["auth", "app", "--bearer-token-file", "SECRET"])]
#[case::oauth1_unloadable(Held::Unloadable, &["auth", "oauth1", "--consumer-key", "ck", "--consumer-secret-file", "SECRET", "--access-token-file", "SECRET", "--token-secret-file", "SECRET"])]
#[case::apps_add_unloadable(Held::Unloadable, &["auth", "apps", "add", "new", "--client-id", "abcdefgh", "--client-secret-file", "SECRET"])]
#[case::apps_update_unloadable(Held::Unloadable, &["auth", "apps", "update", "work", "--client-id", "abcdefgh"])]
#[case::apps_remove_unloadable(Held::Unloadable, &["auth", "apps", "remove", "work", "--force"])]
#[case::redirect_uri_unloadable(Held::Unloadable, &["auth", "apps", "redirect-uri", "set", "work", "http://localhost:8080/callback"])]
#[case::clear_unloadable(Held::Unloadable, &["auth", "clear", "--all", "--force"])]
#[case::default_unloadable(Held::Unloadable, &["auth", "default", "work"])]
#[case::app_for_a_missing_app(Held::Work, &["--app", "ghost", "auth", "app", "--bearer-token-file", "SECRET"])]
#[case::oauth1_for_a_missing_app(Held::Work, &["--app", "ghost", "auth", "oauth1", "--consumer-key", "ck", "--consumer-secret-file", "SECRET", "--access-token-file", "SECRET", "--token-secret-file", "SECRET"])]
#[case::apps_add_taken(Held::Work, &["auth", "apps", "add", "work", "--client-id", "abcdefgh", "--client-secret-file", "SECRET"])]
#[case::apps_add_bad_redirect_uri(Held::Work, &["auth", "apps", "add", "new", "--client-id", "abcdefgh", "--client-secret-file", "SECRET", "--redirect-uri", "not a url"])]
#[case::apps_update_missing(Held::Work, &["auth", "apps", "update", "ghost", "--client-id", "abcdefgh"])]
#[case::apps_remove_missing(Held::Work, &["auth", "apps", "remove", "ghost", "--force"])]
#[case::redirect_uri_missing(Held::Work, &["auth", "apps", "redirect-uri", "set", "ghost", "http://localhost:8080/callback"])]
#[case::default_missing(Held::Work, &["auth", "default", "ghost"])]
#[case::default_user_missing(Held::Work, &["auth", "default", "work", "alice"])]
#[case::clear_nothing(Held::Work, &["auth", "clear"])]
#[cfg_attr(unix, case::app_read_only(Held::ReadOnly, &["auth", "app", "--bearer-token-file", "SECRET"]))]
#[cfg_attr(unix, case::app_for_a_missing_app_read_only(Held::ReadOnly, &["--app", "ghost", "auth", "app", "--bearer-token-file", "SECRET"]))]
#[cfg_attr(unix, case::oauth1_for_a_missing_app_read_only(Held::ReadOnly, &["--app", "ghost", "auth", "oauth1", "--consumer-key", "ck", "--consumer-secret-file", "SECRET", "--access-token-file", "SECRET", "--token-secret-file", "SECRET"]))]
#[cfg_attr(unix, case::apps_add_read_only(Held::ReadOnly, &["auth", "apps", "add", "new", "--client-id", "abcdefgh", "--client-secret-file", "SECRET"]))]
#[cfg_attr(unix, case::clear_read_only(Held::ReadOnly, &["auth", "clear", "--all", "--force"]))]
#[cfg_attr(unix, case::default_read_only(Held::ReadOnly, &["auth", "default", "work"]))]
#[tokio::test]
async fn a_store_write_s_dry_run_refuses_as_its_real_run_does(
    #[case] held: Held,
    #[case] args: &[&str],
) {
    let tmp = TempDir::new().expect("tempdir");
    seed_store_write(held, &tmp);
    let (code, real, _) = store_write(held, &tmp, args, false).await;
    if code == 0 {
        // A process that may write a read-only directory (root) cannot be
        // refused this way.
        assert!(
            !matches!(held, Held::Unloadable | Held::Work | Held::Empty),
            "{args:?}: the real run succeeded: {real}"
        );
        return;
    }
    let (dry_code, dry, unchanged) = store_write(held, &tmp, args, true).await;
    assert_eq!(dry_code, 0, "{args:?}: {dry}");
    assert!(unchanged, "{args:?}: the dry run changed the store");
    assert_eq!(
        (&dry["status"], &dry["would_succeed"]),
        (&serde_json::json!("dry_run"), &serde_json::json!(false)),
        "{args:?}: the dry run does not refuse where the real run says {real}: {dry}"
    );
    for key in ["reason", "exit_code", "message", "next_step"] {
        assert_eq!(
            dry[key], real[key],
            "{args:?}: {key} differs\n  dry:  {dry}\n  real: {real}"
        );
    }
    assert!(
        dry.get("confirmation_required").is_none(),
        "{args:?}: the real run refused without asking, and the dry run says it would ask: {dry}"
    );
}

/// A redirect URI the store refuses leaves the store as it was, so the same
/// command with a URI it accepts then registers the app.
#[tokio::test]
async fn apps_add_with_a_refused_redirect_uri_registers_nothing() {
    let tmp = TempDir::new().expect("tempdir");
    seed_store_write(Held::Empty, &tmp);
    let add = [
        "auth",
        "apps",
        "add",
        "new",
        "--client-id",
        "abcdefgh",
        "--client-secret-file",
        "SECRET",
        "--redirect-uri",
    ];
    let with = |uri| [&add[..], &[uri]].concat();

    let (code, refused, unchanged) =
        store_write(Held::Empty, &tmp, &with("not a url"), false).await;
    assert_eq!(code, 1, "{refused}");
    assert_eq!(refused["reason"], "validation", "{refused}");
    assert_eq!(refused["rule"], "invalid-redirect-uri", "{refused}");
    assert!(unchanged, "the refused add changed the store: {refused}");

    let store = tmp.path().join("home").join(".xurl");
    let secret = tmp.path().join("secret");
    let mut argv = vec!["xr", "--output", "json"];
    argv.extend(with("https://example.com/cb").into_iter().map(|arg| {
        if arg == "SECRET" {
            secret.to_str().expect("utf-8 path")
        } else {
            arg
        }
    }));
    let (code, _stdout, stderr) = run_at(&store, &argv).await;
    assert_eq!(code, 0, "the same command with a good URI failed: {stderr}");
    let held = TokenStore::load_from_path(store.to_str().expect("utf-8 path"));
    assert_eq!(
        held.get_app_redirect_uri("new"),
        Some("https://example.com/cb")
    );
}

/// A dry run of a store write its real run makes answers `would_succeed:
/// true` and writes nothing, and the real run then succeeds.
#[rstest::rstest]
#[case::app(Held::Work, &["auth", "app", "--bearer-token-file", "SECRET"])]
#[case::oauth1(Held::Work, &["auth", "oauth1", "--consumer-key", "ck", "--consumer-secret-file", "SECRET", "--access-token-file", "SECRET", "--token-secret-file", "SECRET"])]
#[case::apps_add(Held::Work, &["auth", "apps", "add", "new", "--client-id", "abcdefgh", "--client-secret-file", "SECRET"])]
#[case::apps_update(Held::Work, &["auth", "apps", "update", "work", "--client-id", "abcdefgh"])]
#[case::apps_remove(Held::Work, &["auth", "apps", "remove", "work", "--force"])]
#[case::redirect_uri(Held::Work, &["auth", "apps", "redirect-uri", "set", "work", "http://localhost:8080/callback"])]
#[case::clear(Held::Work, &["auth", "clear", "--all", "--force"])]
#[case::default(Held::Work, &["auth", "default", "work"])]
#[case::app_on_a_new_store(Held::Empty, &["auth", "app", "--bearer-token-file", "SECRET"])]
#[case::oauth1_on_a_new_store(Held::Empty, &["auth", "oauth1", "--consumer-key", "ck", "--consumer-secret-file", "SECRET", "--access-token-file", "SECRET", "--token-secret-file", "SECRET"])]
#[case::apps_add_on_a_new_store(Held::Empty, &["auth", "apps", "add", "new", "--client-id", "abcdefgh", "--client-secret-file", "SECRET"])]
#[tokio::test]
async fn a_store_write_s_dry_run_answers_success_where_its_real_run_succeeds(
    #[case] held: Held,
    #[case] args: &[&str],
) {
    let tmp = TempDir::new().expect("tempdir");
    seed_store_write(held, &tmp);
    let (dry_code, dry, unchanged) = store_write(held, &tmp, args, true).await;
    assert_eq!(dry_code, 0, "{args:?}: {dry}");
    assert!(unchanged, "{args:?}: the dry run changed the store");
    assert_eq!(
        (&dry["status"], &dry["would_succeed"]),
        (&serde_json::json!("dry_run"), &serde_json::json!(true)),
        "{args:?}: {dry}"
    );
    let mut argv = vec!["xr"];
    let secret = tmp.path().join("secret");
    let secret = secret.to_str().expect("utf-8 path");
    argv.extend(
        args.iter()
            .map(|arg| if *arg == "SECRET" { secret } else { arg }),
    );
    let (code, _stdout, stderr) = run_at(&tmp.path().join("home").join(".xurl"), &argv).await;
    assert_eq!(code, 0, "{args:?}: the real run failed: {stderr}");
}

// ── The page exists ────────────────────────────────────────────────────

/// The step's page is a section of the CLI README, so the heading it anchors
/// to has to be there.
#[test]
fn the_page_inspect_store_names_is_in_the_readme() {
    let (file, anchor) = STORE_DOCS
        .strip_prefix("https://github.com/brettdavies/xurl-rs/blob/main/")
        .and_then(|rest| rest.split_once('#'))
        .expect("a README path and an anchor");
    let readme = std::fs::read_to_string(common::workspace_root().join(file)).expect("the README");
    let found = readme
        .lines()
        .filter_map(|line| line.strip_prefix("### "))
        .any(|heading| heading.to_lowercase().replace(' ', "-") == anchor);
    assert!(found, "{file} has no heading that anchors to #{anchor}");
}

/// The `docs` of a token endpoint 403 name a section of the repository README,
/// so the heading it anchors to has to be there.
#[test]
fn the_page_a_token_endpoint_403_names_is_in_the_readme() {
    let docs = xdk::Error::TokenEndpointForbidden {
        body: String::new(),
    }
    .docs_url()
    .expect("the 403 names a page");
    let anchor = docs
        .strip_prefix("https://github.com/brettdavies/xurl-rs#")
        .expect("the repository README and an anchor");
    let readme = std::fs::read_to_string(common::workspace_root().join("README.md"))
        .expect("the repository README");
    let found = readme
        .lines()
        .filter_map(|line| line.strip_prefix("### "))
        .any(|heading| {
            heading
                .to_lowercase()
                .replace(' ', "-")
                .replace(|c: char| !c.is_alphanumeric() && c != '-', "")
                == anchor
        });
    assert!(found, "README.md has no heading that anchors to #{anchor}");
}

// ── A step leads somewhere ─────────────────────────────────────────────

/// Two signed-in apps: `personal`, the default, holding `alice`, and `work`
/// holding `bob`. Neither token expires, so a dry run finds each offline.
fn two_signed_in_apps(tmp: &TempDir) -> std::path::PathBuf {
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    for (app, login) in [("personal", "alice"), ("work", "bob")] {
        ts.add_app(app, "abcdefgh", "shh").expect("add app");
        ts.save_oauth2_token_for_app(app, login, "AT", "RT", u64::MAX)
            .expect("save token");
    }
    ts.set_default_app("personal").expect("default");
    store
}

/// Runs a step's `command` as a dry run, which sends nothing, and returns
/// the envelope it prints.
async fn dry_run_of(store: &Path, command: &str) -> serde_json::Value {
    let mut words: Vec<&str> = command.split_whitespace().collect();
    assert_eq!(words.first(), Some(&"xr"), "{command}");
    words.insert(1, "--dry-run");
    let (code, stdout, stderr) = run_at(store, &words).await;
    assert_eq!(code, 0, "{command}: {stderr}");
    serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("{command} printed no envelope ({e}): {stdout}"))
}

/// Every credential step a refusal hands out is one that serves the
/// invocation: a rerun against another app finds the login there, and a
/// sign-in stores the login under the name the invocation asked for. A
/// step that pointed at an app holding only other logins would refuse the
/// same way, and that app's step would point back.
#[rstest::rstest]
#[case::login_no_app_holds(&["whoami", "-u", "nobody"])]
#[case::login_no_app_holds_from_the_other_app(&["--app", "work", "whoami", "-u", "nobody"])]
#[case::login_the_default_lacks(&["whoami", "-u", "bob"])]
#[case::login_the_named_app_lacks(&["--app", "work", "whoami", "-u", "alice"])]
#[tokio::test]
async fn a_credential_step_serves_the_invocation_it_answers(#[case] args: &[&str]) {
    let tmp = TempDir::new().expect("tempdir");
    let store = two_signed_in_apps(&tmp);
    let login = args[args.len() - 1];
    let argv: Vec<&str> = ["xr", "--output", "json"]
        .into_iter()
        .chain(args.iter().copied())
        .collect();
    let (code, _stdout, stderr) = run_at(&store, &argv).await;
    assert_eq!(code, 77, "stderr: {stderr}");
    let step = envelope(&stderr)["next_step"].clone();
    let command = step["command"].as_str().expect("a runnable step");
    match step["action"].as_str() {
        Some("select-app") => {
            let answer = dry_run_of(&store, command).await;
            assert_eq!(answer["would_succeed"], true, "{command}: {answer}");
            assert_eq!(answer["auth"]["username"], login, "{command}: {answer}");
        }
        Some("sign-in") => assert!(
            command.starts_with(&format!("xr auth oauth2 {login} ")),
            "the sign-in stores the login under another name: {command}"
        ),
        other => panic!("{argv:?}: unexpected step {other:?}: {step}"),
    }
}

/// A dry run that no stored credential serves carries the step its real
/// run's error carries. A rerun keeps `--dry-run`, so following a dry
/// run's step sends nothing either.
#[rstest::rstest]
#[case::nothing_registered(false, &["whoami"])]
#[case::login_no_app_holds(true, &["whoami", "-u", "nobody"])]
#[case::login_another_app_holds(true, &["whoami", "-u", "bob"])]
#[tokio::test]
async fn a_dry_run_refusal_carries_the_step_of_its_real_run(
    #[case] signed_in: bool,
    #[case] args: &[&str],
) {
    let tmp = TempDir::new().expect("tempdir");
    let store = if signed_in {
        two_signed_in_apps(&tmp)
    } else {
        tmp.path().join(".xurl")
    };
    let real: Vec<&str> = ["xr", "--output", "json"]
        .into_iter()
        .chain(args.iter().copied())
        .collect();
    let (_, _, stderr) = run_at(&store, &real).await;
    let expected = envelope(&stderr)["next_step"].clone();
    assert!(
        expected.is_object(),
        "the real run carries a step: {stderr}"
    );

    let dry: Vec<&str> = ["xr", "--output", "json", "--dry-run"]
        .into_iter()
        .chain(args.iter().copied())
        .collect();
    let (code, stdout, stderr) = run_at(&store, &dry).await;
    assert_eq!(code, 0, "stderr: {stderr}");
    let answer: serde_json::Value = serde_json::from_str(stdout.trim()).expect("an envelope");
    assert_eq!(answer["would_succeed"], false, "{answer}");
    let mut step = answer["next_step"].clone();
    if let Some(command) = step["command"].as_str() {
        step["command"] = command.replacen(" --dry-run", "", 1).into();
    }
    assert_eq!(step, expected, "{answer}");
}

/// A dry run that refuses an input carries the rule it broke and the step
/// its real run names in every structured format, YAML as well as JSON.
#[tokio::test]
async fn a_refused_input_carries_its_message_and_step_in_yaml() {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let (code, stdout, stderr) = run_at(
        &store,
        &[
            "xr",
            "--output",
            "yaml",
            "--dry-run",
            "webhooks",
            "add",
            "https://example.com:8443/hook",
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    for line in [
        "message: The webhook URL names a port; X takes one without a port.",
        "  action: show-help",
        "  command: xr webhooks add --help",
        "reason: validation",
        "rule: webhook-url-has-port",
    ] {
        assert!(
            stdout.lines().any(|l| l == line),
            "no {line:?} in:\n{stdout}"
        );
    }
}

/// A login flag ahead of the command word is refused with the invocation
/// that moves it after the word, and that invocation reaches the command.
#[tokio::test]
async fn a_misplaced_login_flag_moves_to_where_the_command_reads_it() {
    let tmp = TempDir::new().expect("tempdir");
    let store = two_signed_in_apps(&tmp);
    let (code, _stdout, stderr) = run_at(
        &store,
        &[
            "xr", "--output", "json", "--app", "work", "-u", "bob", "whoami",
        ],
    )
    .await;
    assert_eq!(code, 2, "stderr: {stderr}");
    let v = envelope(&stderr);
    assert_eq!(v["reason"], "invalid-args", "{v}");
    assert_eq!(v["next_step"]["action"], "fix-input", "{v}");
    let command = v["next_step"]["command"].as_str().expect("a runnable step");
    assert_eq!(command, "xr --output json --app work whoami -u bob");
    let answer = dry_run_of(&store, command).await;
    assert_eq!(answer["would_succeed"], true, "{answer}");
    assert_eq!(answer["auth"]["username"], "bob", "{answer}");
}

// ── The README places every reason ─────────────────────────────────────

/// The rows of the README's "Which Errors Carry a Step" table: each reason
/// with the cell that says which actions it carries.
fn readme_rows() -> Vec<(String, String)> {
    let readme =
        std::fs::read_to_string(common::workspace_root().join("crates/xurl-cli/README.md"))
            .expect("the README");
    let section = readme
        .split_once("### Which Errors Carry a Step\n")
        .expect("the section")
        .1;
    let section = section
        .split_once("\n### ")
        .map_or(section, |(body, _)| body);
    section
        .lines()
        .filter(|line| line.starts_with("| `") && !line.starts_with("| `reason`"))
        .filter_map(|line| {
            let mut cells = line.trim_matches('|').split('|');
            let reason = cells.next()?.trim().trim_matches('`').to_string();
            Some((reason, cells.next()?.trim().to_string()))
        })
        .collect()
}

/// Every name a closed set holds, read from the type's own schema.
fn names_in(schema: &serde_json::Value) -> Vec<String> {
    fn collect(node: &serde_json::Value, found: &mut Vec<String>) {
        match node {
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    match (key.as_str(), value) {
                        ("const", serde_json::Value::String(name)) => found.push(name.clone()),
                        ("enum", serde_json::Value::Array(names)) => found.extend(
                            names
                                .iter()
                                .filter_map(|name| name.as_str().map(str::to_string)),
                        ),
                        _ => collect(value, found),
                    }
                }
            }
            serde_json::Value::Array(items) => items.iter().for_each(|item| collect(item, found)),
            _ => {}
        }
    }
    let mut found = Vec::new();
    collect(schema, &mut found);
    found
}

fn every_reason() -> Vec<String> {
    names_in(&serde_json::to_value(schemars::schema_for!(cli::envelope::Reason)).expect("schema"))
}

fn every_action() -> Vec<String> {
    names_in(&serde_json::to_value(schemars::schema_for!(cli::hints::NextAction)).expect("schema"))
}

/// What each `reason-<name>` fixture shows: the reason, and the action of
/// the step its envelope carries, if it carries one.
fn fixture_steps() -> Vec<(String, Option<String>)> {
    let golden = common::workspace_root().join("crates/xurl-cli/tests/golden");
    let action = regex::Regex::new(r#""action": "([a-z-]+)""#).unwrap();
    let mut found = Vec::new();
    for entry in std::fs::read_dir(&golden).expect("the fixtures") {
        let path = entry.expect("a fixture").path();
        let Some(reason) = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .and_then(|stem| stem.strip_prefix("reason-"))
        else {
            continue;
        };
        let text = std::fs::read_to_string(&path).expect("read the fixture");
        let step = text
            .split_once("\"next_step\"")
            .and_then(|(_, rest)| action.captures(rest))
            .map(|found| found[1].to_string());
        found.push((reason.to_string(), step));
    }
    found
}

/// The README gives every reason one row, names no reason and no action
/// that does not exist, and agrees with the fixtures: the step a fixture
/// shows is one its row lists, and a fixture with no step has a row that
/// says when there is none.
#[test]
fn the_readme_gives_every_reason_a_row_the_fixtures_agree_with() {
    let rows = readme_rows();
    let reasons = every_reason();
    let actions = every_action();
    assert!(reasons.len() > 30, "the closed set: {reasons:?}");
    assert!(actions.len() > 10, "the closed set: {actions:?}");

    let name = regex::Regex::new(r"`([a-z-]+)`").unwrap();
    let mut misplaced = Vec::new();
    for reason in &reasons {
        let count = rows.iter().filter(|(named, _)| named == reason).count();
        if count != 1 {
            misplaced.push(format!("{reason}: {count} rows in the table"));
        }
    }
    for (reason, cell) in &rows {
        if !reasons.contains(reason) {
            misplaced.push(format!("{reason}: has a row and no reason has that name"));
        }
        let listed = cell.split(':').next().unwrap_or(cell);
        if !name
            .captures_iter(listed)
            .any(|found| actions.contains(&found[1].to_string()))
        {
            misplaced.push(format!("{reason}: its row names no action"));
        }
    }
    let fixtures = fixture_steps();
    assert!(fixtures.len() > 30, "the reason fixtures: {fixtures:?}");
    for (reason, step) in fixtures {
        let Some((_, cell)) = rows.iter().find(|(named, _)| *named == reason) else {
            continue;
        };
        match step {
            Some(action) if !cell.contains(&format!("`{action}`")) => misplaced.push(format!(
                "{reason}: its fixture carries `{action}`, which its row does not list"
            )),
            None if !cell.contains("none") => misplaced.push(format!(
                "{reason}: its fixture carries no step, and its row does not say when there is none"
            )),
            _ => {}
        }
    }
    assert!(
        misplaced.is_empty(),
        "crates/xurl-cli/README.md, \"Which Errors Carry a Step\", disagrees with the reasons:\n{}",
        misplaced.join("\n")
    );
}
