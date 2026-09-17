//! Byte-for-byte golden fixtures for the `xr` binary.
//!
//! Every case spawns the built binary through the hermetic seam in
//! `tests/common/mod.rs`, captures stdout, stderr, and the exit code, and
//! compares all three against `tests/golden/<case>.golden`. The matrix holds
//! the root help and every subcommand's help (walked from each `Commands:`
//! block, so a new command demands a fixture and a removed one strands its
//! file), both version forms, the schema and example listings, one
//! `--verbose` exchange against a mock server in each colour mode, and one
//! `--output json` error envelope per reason the `reason` description in
//! `schema/output.schema.json` names. Reasons no argv can reach are listed
//! in [`UNTRIGGERABLE`] with the reason, and a completeness test fails when
//! the schema gains a reason that is neither captured nor listed.
//!
//! Three values vary between runs and are replaced by placeholders before
//! the comparison: the mock server origin, the scratch home directory, and
//! the crate version. Everything else must match byte for byte.
//!
//! Every case points `API_BASE_URL` at the mock server, so a trigger that
//! regresses into sending a request can never reach the live API.
//!
//! Re-capture with `XURL_GOLDEN_BLESS=1 cargo test --test golden_tests`.
//! `XURL_GOLDEN_BIN=<path>` compares a different build of `xr` against the
//! same fixtures.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use pretty_assertions::StrComparison;
use tempfile::TempDir;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const BASE_URL_PLACEHOLDER: &str = "{{API_BASE_URL}}";
const HOME_PLACEHOLDER: &str = "{{HOME}}";
/// Stands in for the run's scratch directory inside an `env:` value.
const SCRATCH_PLACEHOLDER: &str = "{{SCRATCH}}";
const VERSION_PLACEHOLDER: &str = "{{CRATE_VERSION}}";
const BEARER: &str = "GOLDEN-BEARER-TOKEN";
const FIXTURE_EXT: &str = "golden";

/// Reasons the schema names that no argv can reach, each with why.
const UNTRIGGERABLE: &[(&str, &str)] = &[
    (
        "invalid-url",
        "raw mode rejects a URL that is neither http(s) nor an absolute path as \
         `validation` before a raw-URL target exists, so the scheme allowlist \
         that raises this never runs from argv",
    ),
    (
        "internal",
        "raised only when a shortcut's path template names a parameter the \
         shortcut did not bind, or when a raw-URL target is asked for its \
         template; both are programmer errors with no argv path",
    ),
];

/// Variables the colour libraries consult that the seam does not strip.
const COLOR_FORCING_VARS: &[&str] = &["CLICOLOR", "CLICOLOR_FORCE"];

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn schema_path() -> PathBuf {
    common::workspace_root().join("schema/output.schema.json")
}

/// Which credential the spawned `xr` can reach.
#[derive(Clone, Copy, Default)]
enum Store {
    /// The seam's unwritable store: no credential anywhere.
    #[default]
    Empty,
    /// `XURL_BEARER_TOKEN` in the environment, no store file.
    BearerEnv,
    /// A seeded store carrying one app with an OAuth2 token.
    OAuth2,
}

#[derive(Default)]
struct Case {
    name: String,
    args: Vec<String>,
    env: Vec<(&'static str, String)>,
    unset_home: bool,
    /// A scratch directory to present as `HOME`; substituted on output.
    home: Option<PathBuf>,
    stdin: Option<&'static str>,
    /// Working directory for the spawn, so a file argument can stay relative.
    cwd: Option<PathBuf>,
    /// The envelope reason this case exists to capture.
    reason: Option<&'static str>,
    store: Store,
}

fn case(name: &str, args: &[&str]) -> Case {
    Case {
        name: name.to_string(),
        args: args.iter().map(|a| (*a).to_string()).collect(),
        ..Case::default()
    }
}

fn reason_case(reason: &'static str, args: &[&str]) -> Case {
    Case {
        reason: Some(reason),
        ..case(&format!("reason-{reason}"), args)
    }
}

struct Captured {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

/// A wiremock server on its own runtime, leaked so the spawned processes
/// can reach it for the whole test without an async test body.
struct MockApi {
    _rt: tokio::runtime::Runtime,
    server: &'static MockServer,
    uri: String,
}

impl MockApi {
    fn start() -> Self {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        let server = rt.block_on(async { Box::leak(Box::new(MockServer::start().await)) });
        let uri = server.uri();
        let api = Self {
            _rt: rt,
            server,
            uri,
        };
        api.mount_all();
        api
    }

    fn mount(&self, mock: Mock) {
        self._rt.block_on(async {
            mock.mount(self.server).await;
        });
    }

    /// One search endpoint whose query selects the status, plus the media
    /// initialize endpoint answering with an empty id. The `date` header is
    /// pinned because hyper fills it in with the wall clock otherwise.
    fn mount_all(&self) {
        let search = |query: &str, template: ResponseTemplate| {
            Mock::given(method("GET"))
                .and(path("/2/tweets/search/recent"))
                .and(query_param("query", query))
                .respond_with(template.insert_header("date", "Thu, 01 Jan 2026 00:00:00 GMT"))
        };
        self.mount(search(
            "hi",
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"id": "1", "text": "hi"}],
                "meta": {"result_count": 1}
            })),
        ));
        self.mount(search(
            "q429",
            ResponseTemplate::new(429).set_body_json(serde_json::json!({
                "title": "Too Many Requests",
                "detail": "Too Many Requests",
                "type": "about:blank",
                "status": 429
            })),
        ));
        self.mount(search(
            "q404",
            ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "errors": [{"title": "Not Found Error", "detail": "Could not find data"}]
            })),
        ));
        self.mount(search(
            "q500",
            ResponseTemplate::new(500).set_body_json(serde_json::json!({
                "title": "Internal Server Error",
                "status": 500
            })),
        ));
        self.mount(
            Mock::given(method("POST"))
                .and(path("/2/media/upload/initialize"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(serde_json::json!({"data": {"id": ""}})),
                ),
        );
    }
}

/// Everything a case may need that exists only for the duration of a run.
struct Scratch {
    dir: TempDir,
}

impl Scratch {
    fn new() -> Self {
        Self {
            dir: TempDir::new().expect("tempdir"),
        }
    }

    fn oauth2_store(&self) -> PathBuf {
        let store = self.dir.path().join("oauth2-store").join(".xurl");
        std::fs::create_dir_all(store.parent().expect("parent")).expect("store dir");
        let mut ts = xdk::store::TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
        ts.add_app("myapp", "CLIENT-ID-VALUE", "SECRET-VALUE")
            .expect("add_app");
        ts.set_default_app("myapp").expect("set_default_app");
        let _ = ts.remove_app("default");
        ts.save_oauth2_token_for_app(
            "myapp",
            "alice",
            "ACCESS-TOKEN-VALUE",
            "REFRESH-TOKEN-VALUE",
            4_000_000_000,
        )
        .expect("save_oauth2");
        store
    }

    /// A directory holding one small file to upload, named by the return.
    fn upload_dir(&self) -> (PathBuf, &'static str) {
        const NAME: &str = "upload.bin";
        let dir = self.dir.path().join("upload");
        std::fs::create_dir_all(&dir).expect("upload dir");
        std::fs::write(dir.join(NAME), b"golden media bytes").expect("upload file");
        (dir, NAME)
    }

    /// A home whose skill destination is a plain file, so removing the
    /// directory that should be there fails.
    fn home_with_blocked_skill_dir(&self) -> PathBuf {
        let home = self.dir.path().join("home-remove-failed");
        let skills = home.join(".claude").join("skills");
        std::fs::create_dir_all(&skills).expect("skills dir");
        std::fs::write(skills.join("xurl-rs"), b"").expect("blocking file");
        home
    }

    /// A home whose skill destination already holds a file, so an install
    /// refuses to clone over it.
    fn home_with_populated_skill_dir(&self) -> PathBuf {
        let home = self.dir.path().join("home-populated");
        let dest = home.join(".claude").join("skills").join("xurl-rs");
        std::fs::create_dir_all(&dest).expect("skill dir");
        std::fs::write(dest.join("SKILL.md"), b"").expect("existing file");
        home
    }

    /// An empty home: nothing installed, nothing in the way.
    fn empty_home(&self) -> PathBuf {
        let home = self.dir.path().join("home-empty");
        std::fs::create_dir_all(&home).expect("home dir");
        home
    }

    /// A `PATH` holding only a `git` that refuses every clone with a fixed
    /// message, so the clone-failed envelope is byte-stable. Returned as the
    /// placeholder form a case's `env` carries.
    fn path_with_refusing_git(&self) -> String {
        let bin = self.dir.path().join("refusing-git");
        std::fs::create_dir_all(&bin).expect("bin dir");
        let git = bin.join("git");
        std::fs::write(
            &git,
            "#!/bin/sh\necho 'fatal: golden clone refused' >&2\nexit 128\n",
        )
        .expect("fake git");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&git, std::fs::Permissions::from_mode(0o755))
                .expect("executable");
        }
        format!("{SCRATCH_PLACEHOLDER}/refusing-git")
    }

    /// A `PATH` with no `git` on it at all, in placeholder form.
    fn path_without_git(&self) -> String {
        std::fs::create_dir_all(self.dir.path().join("no-git")).expect("bin dir");
        format!("{SCRATCH_PLACEHOLDER}/no-git")
    }
}

fn static_cases() -> Vec<Case> {
    vec![
        case("version-flag", &["--version"]),
        case("version-subcommand", &["version"]),
        case("schema-list", &["schema", "--list"]),
        case("examples", &["examples"]),
    ]
}

fn verbose_cases() -> Vec<Case> {
    let bearer = |c: Case| Case {
        store: Store::BearerEnv,
        ..c
    };
    vec![
        bearer(case(
            "verbose-plain",
            &["--verbose", "--auth", "app", "search", "hi"],
        )),
        bearer(case(
            "verbose-color",
            &[
                "--verbose",
                "--color",
                "always",
                "--auth",
                "app",
                "search",
                "hi",
            ],
        )),
    ]
}

/// One `--output json` trigger per reachable reason.
fn reason_cases(scratch: &Scratch) -> Vec<Case> {
    let bearer = |c: Case| Case {
        store: Store::BearerEnv,
        ..c
    };
    let (upload_dir, upload_name) = scratch.upload_dir();
    vec![
        reason_case("invalid-args", &["--output", "json", "--bogus-flag"]),
        reason_case("unknown-command", &["--output", "json", "bogus"]),
        reason_case("auth-required", &["--output", "json", "/2/users/me"]),
        bearer(reason_case(
            "auth-method-mismatch",
            &["--output", "json", "--auth", "app", "whoami"],
        )),
        reason_case(
            "client-credentials-missing",
            &["--output", "json", "auth", "oauth2"],
        ),
        bearer(reason_case(
            "rate-limited",
            &["--output", "json", "--auth", "app", "search", "q429"],
        )),
        bearer(reason_case(
            "not-found",
            &["--output", "json", "--auth", "app", "search", "q404"],
        )),
        bearer(reason_case(
            "network-error",
            &["--output", "json", "--auth", "app", "search", "q500"],
        )),
        bearer(reason_case(
            "invalid-method",
            &[
                "--output",
                "json",
                "--auth",
                "app",
                "-X",
                "BAD METHOD",
                "/2/tweets/search/recent",
            ],
        )),
        reason_case("invalid-path-param", &["--output", "json", "read", "1/2"]),
        reason_case("validation", &["--output", "json", "relative/path"]),
        Case {
            store: Store::OAuth2,
            cwd: Some(upload_dir),
            ..reason_case(
                "serialization",
                &["--output", "json", "media", "upload", upload_name],
            )
        },
        reason_case(
            "io",
            &[
                "--output",
                "json",
                "validate",
                "/nonexistent/xr-golden.json",
            ],
        ),
        reason_case(
            "token-store",
            &[
                "--output",
                "json",
                "auth",
                "apps",
                "remove",
                "nosuchapp",
                "--force",
            ],
        ),
        reason_case(
            "confirmation-required",
            &["--output", "json", "--no-interactive", "delete", "123"],
        ),
        reason_case("no-tty", &["--output", "json", "auth", "default"]),
        reason_case(
            "unsupported-pagination",
            &["--output", "json", "--page", "2", "timeline"],
        ),
        Case {
            stdin: Some("nope"),
            ..reason_case("invalid-json", &["--output", "json", "validate"])
        },
        Case {
            stdin: Some("{}"),
            ..reason_case(
                "unknown-schema",
                &["--output", "json", "validate", "--schema", "bogus"],
            )
        },
        Case {
            stdin: Some(r#"{"foo":1}"#),
            ..reason_case(
                "validation-failed",
                &["--output", "json", "validate", "--schema", "user"],
            )
        },
        reason_case("missing-host", &["--output", "json", "skill", "install"]),
        Case {
            unset_home: true,
            ..reason_case(
                "home-not-set",
                &[
                    "--output",
                    "json",
                    "skill",
                    "install",
                    "claude_code",
                    "--dry-run",
                ],
            )
        },
        Case {
            home: Some(scratch.home_with_blocked_skill_dir()),
            ..reason_case(
                "remove-failed",
                &["--output", "json", "skill", "update", "claude_code"],
            )
        },
        Case {
            home: Some(scratch.home_with_populated_skill_dir()),
            ..reason_case(
                "destination-not-empty",
                &["--output", "json", "skill", "install", "claude_code"],
            )
        },
        Case {
            home: Some(scratch.home_with_blocked_skill_dir()),
            ..reason_case(
                "destination-is-file",
                &["--output", "json", "skill", "install", "claude_code"],
            )
        },
        Case {
            home: Some(scratch.empty_home()),
            env: vec![("PATH", scratch.path_without_git())],
            ..reason_case(
                "git-not-found",
                &["--output", "json", "skill", "install", "claude_code"],
            )
        },
        Case {
            home: Some(scratch.empty_home()),
            env: vec![("PATH", scratch.path_with_refusing_git())],
            ..reason_case(
                "git-clone-failed",
                &["--output", "json", "skill", "install", "claude_code"],
            )
        },
        Case {
            home: Some(scratch.empty_home()),
            ..reason_case(
                "not-installed",
                &["--output", "json", "skill", "update", "--all"],
            )
        },
    ]
}

/// The dry-run refusals a shortcut's validator raises before any request.
fn dry_run_cases() -> Vec<Case> {
    let long_body = "x".repeat(281);
    let mut too_many: Vec<String> = ["--output", "json", "--dry-run", "post", "hi"]
        .iter()
        .map(|a| (*a).to_string())
        .collect();
    too_many.extend((1..=5).map(|i| format!("--media-id={i}")));
    vec![
        case(
            "dry-run-empty-body",
            &["--output", "json", "--dry-run", "post", ""],
        ),
        case(
            "dry-run-body-too-long",
            &["--output", "json", "--dry-run", "post", &long_body],
        ),
        Case {
            name: "dry-run-too-many-attachments".to_string(),
            args: too_many,
            ..Case::default()
        },
        case(
            "dry-run-empty-post-id",
            &["--output", "json", "--dry-run", "like", ""],
        ),
        case(
            "dry-run-empty-username",
            &["--output", "json", "--dry-run", "follow", ""],
        ),
    ]
}

/// The text renderings of the errors that carry a recovery hint, plus the
/// parser's rejection of an unknown skill host.
fn text_cases() -> Vec<Case> {
    let bearer = |c: Case| Case {
        store: Store::BearerEnv,
        ..c
    };
    vec![
        case("text-auth-required", &["/2/users/me"]),
        bearer(case(
            "text-auth-method-mismatch",
            &["--auth", "app", "whoami"],
        )),
        case("text-client-credentials-missing", &["auth", "oauth2"]),
        bearer(case(
            "text-rate-limited",
            &["--auth", "app", "search", "q429"],
        )),
        case(
            "skill-install-unknown-host",
            &["skill", "install", "bogus_host"],
        ),
    ]
}

/// The first token of each entry in the `Commands:` block of a help page.
fn commands_in(help: &str) -> Vec<String> {
    let mut lines = help.lines().skip_while(|l| *l != "Commands:");
    lines.next();
    lines
        .take_while(|l| !l.trim().is_empty())
        .filter_map(|l| l.strip_prefix("  ").filter(|rest| !rest.starts_with(' ')))
        .filter_map(|entry| entry.split_whitespace().next())
        .filter(|name| *name != "help")
        .map(str::to_string)
        .collect()
}

/// Root help plus every subcommand's help, found by walking the tree.
fn help_cases(bin: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    let mut pending = vec![Vec::<String>::new()];
    while let Some(command_path) = pending.pop() {
        let mut args = command_path.clone();
        args.push("--help".to_string());
        let output = common::xr_std_at(bin)
            .args(&args)
            .stdin(Stdio::null())
            .output()
            .expect("spawn xr --help");
        let help = String::from_utf8(output.stdout).expect("help is utf-8");
        for sub in commands_in(&help) {
            let mut next = command_path.clone();
            next.push(sub);
            pending.push(next);
        }
        let name = if command_path.is_empty() {
            "help".to_string()
        } else {
            format!("help-{}", command_path.join("-"))
        };
        cases.push(Case {
            name,
            args,
            ..Case::default()
        });
    }
    cases.sort_by(|a, b| a.name.cmp(&b.name));
    cases
}

fn all_cases(bin: &str, scratch: &Scratch) -> Vec<Case> {
    let mut cases = help_cases(bin);
    cases.extend(static_cases());
    cases.extend(verbose_cases());
    cases.extend(reason_cases(scratch));
    cases.extend(dry_run_cases());
    cases.extend(text_cases());
    let mut seen = BTreeSet::new();
    for c in &cases {
        assert!(
            seen.insert(c.name.clone()),
            "duplicate case name {}",
            c.name
        );
    }
    cases
}

/// Spawns one case and returns its output with the run-varying values
/// replaced by placeholders.
fn capture(case: &Case, bin: &str, api: &MockApi, scratch: &Scratch) -> Captured {
    let mut cmd = match case.store {
        Store::OAuth2 => common::xr_std_with_store_at(bin, &scratch.oauth2_store()),
        Store::Empty | Store::BearerEnv => common::xr_std_at(bin),
    };
    if matches!(case.store, Store::BearerEnv) {
        cmd.env("XURL_BEARER_TOKEN", BEARER);
    }
    cmd.env("API_BASE_URL", &api.uri);
    for var in COLOR_FORCING_VARS {
        cmd.env_remove(var);
    }
    for (key, value) in &case.env {
        let scratch_dir = scratch.dir.path().to_str().expect("utf-8 path");
        cmd.env(key, value.replace(SCRATCH_PLACEHOLDER, scratch_dir));
    }
    if case.unset_home {
        cmd.env_remove("HOME");
    }
    if let Some(home) = &case.home {
        cmd.env("HOME", home);
    }
    if let Some(cwd) = &case.cwd {
        cmd.current_dir(cwd);
    }
    cmd.args(&case.args);
    cmd.stdin(if case.stdin.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("spawn xr");
    if let Some(input) = case.stdin {
        let mut stdin = child.stdin.take().expect("piped stdin");
        stdin.write_all(input.as_bytes()).expect("write stdin");
        drop(stdin);
    }
    let output = child.wait_with_output().expect("wait for xr");

    let substitute = |bytes: Vec<u8>| {
        let mut text = String::from_utf8(bytes).expect("xr output is utf-8");
        text = text.replace(&api.uri, BASE_URL_PLACEHOLDER);
        if let Some(home) = &case.home {
            text = text.replace(home.to_str().expect("utf-8 path"), HOME_PLACEHOLDER);
        }
        text.replace(env!("CARGO_PKG_VERSION"), VERSION_PLACEHOLDER)
    };
    Captured {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: substitute(output.stdout),
        stderr: substitute(output.stderr),
    }
}

/// The `reason` of the one JSON envelope a capture carries, on either stream.
///
/// A multi-host envelope carries its reasons per host under `results`; the
/// first one is the case's reason when every host reports the same.
fn envelope_reason(captured: &Captured) -> Option<String> {
    let value = [&captured.stderr, &captured.stdout]
        .into_iter()
        .filter(|stream| !stream.trim().is_empty())
        .find_map(|stream| serde_json::from_str::<serde_json::Value>(stream).ok())?;
    let reason_of = |v: &serde_json::Value| v.get("reason")?.as_str().map(str::to_string);
    reason_of(&value).or_else(|| {
        let results = value.get("installations")?.as_array()?;
        let mut reasons = results.iter().map(reason_of);
        let first = reasons.next()??;
        reasons
            .all(|r| r.as_deref() == Some(first.as_str()))
            .then_some(first)
    })
}

fn render_fixture(case: &Case, captured: &Captured) -> Vec<u8> {
    let mut out = Vec::new();
    let _ = writeln!(out, "args: {:?}", case.args);
    for (key, value) in &case.env {
        let _ = writeln!(out, "env: {key}={value}");
    }
    if case.unset_home {
        let _ = writeln!(out, "env: -HOME");
    }
    if case.home.is_some() {
        let _ = writeln!(out, "env: HOME={HOME_PLACEHOLDER}");
    }
    let _ = writeln!(out, "store: {}", store_label(case.store));
    if let Some(input) = case.stdin {
        let _ = writeln!(out, "stdin: {input:?}");
    }
    let _ = writeln!(out, "exit_code: {}", captured.exit_code);
    let _ = writeln!(out, "stdout_len: {}", captured.stdout.len());
    let _ = writeln!(out, "stderr_len: {}", captured.stderr.len());
    out.extend_from_slice(b"--- stdout ---\n");
    out.extend_from_slice(captured.stdout.as_bytes());
    out.extend_from_slice(b"\n--- stderr ---\n");
    out.extend_from_slice(captured.stderr.as_bytes());
    out.extend_from_slice(b"\n");
    out
}

fn store_label(store: Store) -> &'static str {
    match store {
        Store::Empty => "empty",
        Store::BearerEnv => "bearer-env",
        Store::OAuth2 => "oauth2",
    }
}

fn parse_fixture(bytes: &[u8]) -> Result<Captured, String> {
    const STDOUT_MARKER: &[u8] = b"--- stdout ---\n";
    const STDERR_MARKER: &[u8] = b"\n--- stderr ---\n";

    let header_end = find(bytes, STDOUT_MARKER).ok_or("no stdout marker")?;
    let header = std::str::from_utf8(&bytes[..header_end]).map_err(|e| e.to_string())?;
    let field = |key: &str| -> Result<String, String> {
        header
            .lines()
            .find_map(|line| line.strip_prefix(key))
            .map(|v| v.trim().to_string())
            .ok_or_else(|| format!("header lacks {key:?}"))
    };
    let exit_code: i32 = field("exit_code:")?
        .parse()
        .map_err(|e| format!("exit_code: {e}"))?;
    let stdout_len: usize = field("stdout_len:")?
        .parse()
        .map_err(|e| format!("stdout_len: {e}"))?;
    let stderr_len: usize = field("stderr_len:")?
        .parse()
        .map_err(|e| format!("stderr_len: {e}"))?;

    let mut pos = header_end + STDOUT_MARKER.len();
    let stdout = take(bytes, &mut pos, stdout_len).ok_or("stdout shorter than stdout_len")?;
    expect(bytes, &mut pos, STDERR_MARKER)?;
    let stderr = take(bytes, &mut pos, stderr_len).ok_or("stderr shorter than stderr_len")?;
    expect(bytes, &mut pos, b"\n")?;
    if pos != bytes.len() {
        return Err(format!("{} trailing bytes after stderr", bytes.len() - pos));
    }
    Ok(Captured {
        exit_code,
        stdout: String::from_utf8(stdout.to_vec()).map_err(|e| e.to_string())?,
        stderr: String::from_utf8(stderr.to_vec()).map_err(|e| e.to_string())?,
    })
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn take<'a>(bytes: &'a [u8], pos: &mut usize, len: usize) -> Option<&'a [u8]> {
    let slice = bytes.get(*pos..*pos + len)?;
    *pos += len;
    Some(slice)
}

fn expect(bytes: &[u8], pos: &mut usize, marker: &[u8]) -> Result<(), String> {
    match bytes.get(*pos..*pos + marker.len()) {
        Some(found) if found == marker => {
            *pos += marker.len();
            Ok(())
        }
        _ => Err(format!(
            "expected {:?} at byte {pos}",
            String::from_utf8_lossy(marker)
        )),
    }
}

/// Every difference between a capture and its fixture, as one report.
fn diff_report(name: &str, expected: &Captured, actual: &Captured) -> Option<String> {
    let mut report = String::new();
    if expected.exit_code != actual.exit_code {
        let _ = writeln!(
            report,
            "exit code: fixture {} but binary returned {}",
            expected.exit_code, actual.exit_code
        );
    }
    for (stream, want, got) in [
        ("stdout", &expected.stdout, &actual.stdout),
        ("stderr", &expected.stderr, &actual.stderr),
    ] {
        if want != got {
            let _ = writeln!(
                report,
                "{stream} differs (fixture on the left):\n{}",
                StrComparison::new(want, got)
            );
        }
    }
    (!report.is_empty()).then(|| format!("== {name} ==\n{report}"))
}

/// The kebab-case identifiers the schema's `reason` description names.
fn schema_reasons() -> BTreeSet<String> {
    let text = std::fs::read_to_string(schema_path()).expect("schema readable");
    let schema: serde_json::Value = serde_json::from_str(&text).expect("schema is JSON");
    let description = schema["oneOf"]
        .as_array()
        .expect("envelope is a oneOf")
        .iter()
        .find_map(|branch| branch["properties"]["reason"]["description"].as_str())
        .expect("one branch declares reason with a description");
    description
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|token| token.chars().all(|c| c.is_ascii_lowercase() || c == '-'))
        .map(str::to_string)
        .collect()
}

#[test]
fn xr_output_matches_the_golden_fixtures() {
    let bin = std::env::var("XURL_GOLDEN_BIN").unwrap_or_else(|_| common::xr_bin().to_string());
    let bless = std::env::var("XURL_GOLDEN_BLESS").is_ok_and(|v| v == "1");
    assert!(
        !bless || std::env::var_os("CI").is_none(),
        "XURL_GOLDEN_BLESS=1 rewrites the fixtures and never runs under CI"
    );
    let api = MockApi::start();
    let scratch = Scratch::new();
    let dir = golden_dir();
    std::fs::create_dir_all(&dir).expect("golden dir");

    let cases = all_cases(&bin, &scratch);
    let mut failures = Vec::new();
    let mut expected_files = BTreeSet::new();

    for case in &cases {
        let file = dir.join(format!("{}.{FIXTURE_EXT}", case.name));
        expected_files.insert(file.clone());
        let actual = capture(case, &bin, &api, &scratch);

        if let Some(reason) = case.reason {
            let found = envelope_reason(&actual);
            if found.as_deref() != Some(reason) {
                failures.push(format!(
                    "== {} ==\ntrigger no longer yields reason {reason:?}: found {found:?}\nstdout:\n{}\nstderr:\n{}",
                    case.name, actual.stdout, actual.stderr
                ));
            }
        }

        if bless {
            std::fs::write(&file, render_fixture(case, &actual)).expect("write fixture");
            continue;
        }
        let Ok(bytes) = std::fs::read(&file) else {
            failures.push(format!(
                "== {} ==\nfixture {} is missing",
                case.name,
                file.display()
            ));
            continue;
        };
        let expected = match parse_fixture(&bytes) {
            Ok(expected) => expected,
            Err(e) => {
                failures.push(format!("== {} ==\nfixture unreadable: {e}", case.name));
                continue;
            }
        };
        if let Some(report) = diff_report(&case.name, &expected, &actual) {
            failures.push(report);
        }
    }

    let on_disk: BTreeSet<PathBuf> = std::fs::read_dir(&dir)
        .expect("golden dir readable")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|e| e == FIXTURE_EXT))
        .collect();
    for orphan in on_disk.difference(&expected_files) {
        failures.push(format!(
            "== {} ==\nfixture has no case; a command or reason was removed, so trash the file deliberately",
            orphan.display()
        ));
    }

    assert!(
        failures.is_empty(),
        "{} golden case(s) diverged. Re-capture only when the change to what `xr` prints is intended: \
         XURL_GOLDEN_BLESS=1 cargo test --test golden_tests\n\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn every_schema_reason_is_captured_or_listed_untriggerable() {
    let scratch = Scratch::new();
    let captured: BTreeSet<String> = reason_cases(&scratch)
        .iter()
        .filter_map(|c| c.reason.map(str::to_string))
        .collect();
    let listed: BTreeMap<&str, &str> = UNTRIGGERABLE.iter().copied().collect();
    let schema = schema_reasons();

    assert!(
        schema.len() >= 20,
        "schema description names only {} reasons; the parse is broken",
        schema.len()
    );

    let uncovered: Vec<&String> = schema
        .iter()
        .filter(|r| !captured.contains(*r) && !listed.contains_key(r.as_str()))
        .collect();
    assert!(
        uncovered.is_empty(),
        "schema reasons with neither a golden fixture nor an untriggerable entry: {uncovered:?}"
    );

    let stale: Vec<&&str> = listed
        .keys()
        .filter(|r| !schema.contains(**r) || captured.contains(**r))
        .collect();
    assert!(
        stale.is_empty(),
        "untriggerable entries that the schema no longer names or a fixture now covers: {stale:?}"
    );

    let unknown: Vec<&String> = captured.iter().filter(|r| !schema.contains(*r)).collect();
    assert!(
        unknown.is_empty(),
        "golden reasons the schema does not name: {unknown:?}"
    );
}

#[test]
fn fixture_format_round_trips_byte_for_byte() {
    let case = Case {
        stdin: Some("in"),
        ..case("round-trip", &["--x", "a b"])
    };
    let captured = Captured {
        exit_code: 7,
        stdout: "no trailing newline".to_string(),
        stderr: "--- stderr ---\n--- stdout ---\n\n".to_string(),
    };
    let parsed = parse_fixture(&render_fixture(&case, &captured)).expect("parses");
    assert_eq!(parsed.exit_code, 7);
    assert_eq!(parsed.stdout, captured.stdout);
    assert_eq!(parsed.stderr, captured.stderr);
}

#[test]
fn commands_block_parser_reads_only_direct_entries() {
    let help = "Usage: xr auth <COMMAND>\n\nCommands:\n  oauth2   Configure\n  status   Show\n           continued description\n  help     Print this message\n\nOptions:\n  -h, --help\n";
    assert_eq!(commands_in(help), vec!["oauth2", "status"]);
    assert!(commands_in("no commands here").is_empty());
}

/// Every `reason` literal the binary can emit, read from the source: the
/// `kind()` arms, the installer's `reason()` arms, its `REASON_*` constants,
/// every `print_error_envelope` call, and every `reason:` field literal.
fn source_reasons() -> BTreeSet<String> {
    let files = common::shipped_sources();
    let arm = regex::Regex::new(r#"=> "([a-z]+(?:-[a-z]+)+)""#).unwrap();
    let envelope_call =
        regex::Regex::new(r#"print_error_envelope\(\s*[^,]+,\s*"([a-z-]+)""#).unwrap();
    let field = regex::Regex::new(r#"reason: (?:Some\()?"([a-z]+(?:-[a-z]+)+)""#).unwrap();
    let constant = regex::Regex::new(r#"const REASON_[A-Z_]+: &str = "([a-z-]+)""#).unwrap();
    let mut out = BTreeSet::new();
    for path in files {
        let source = std::fs::read_to_string(&path).expect("source readable");
        let arms_apply = path.ends_with("error.rs") || path.ends_with("skill_install/mod.rs");
        for line in source.lines() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            if arms_apply {
                for m in arm.captures_iter(line) {
                    out.insert(m[1].to_string());
                }
            }
            for m in field.captures_iter(line) {
                out.insert(m[1].to_string());
            }
            for m in constant.captures_iter(line) {
                out.insert(m[1].to_string());
            }
        }
        for m in envelope_call.captures_iter(&source) {
            out.insert(m[1].to_string());
        }
    }
    out
}

/// The refusals `validate_*` in the library's `api/shortcuts.rs` can raise.
fn validator_reasons() -> BTreeSet<String> {
    let source =
        std::fs::read_to_string(common::workspace_root().join("crates/xdk/src/api/shortcuts.rs"))
            .expect("shortcuts readable");
    let refusal = regex::Regex::new(r#"return Err\("([a-z]+(?:-[a-z]+)+)"\)"#).unwrap();
    refusal
        .captures_iter(&source)
        .map(|m| m[1].to_string())
        .collect()
}

/// A reason the source can emit is named by the schema and pinned by a
/// fixture (or listed untriggerable), so a reason added to the binary
/// without its documentation and its capture fails here.
#[test]
fn every_reason_the_source_emits_is_documented_and_pinned() {
    let scratch = Scratch::new();
    let captured: BTreeSet<String> = reason_cases(&scratch)
        .iter()
        .filter_map(|c| c.reason.map(str::to_string))
        .collect();
    let listed: BTreeSet<&str> = UNTRIGGERABLE.iter().map(|(r, _)| *r).collect();
    let schema = schema_reasons();
    let emitted = source_reasons();

    assert!(
        emitted.len() >= 20,
        "source scan found only {} reasons; a pattern is broken: {emitted:?}",
        emitted.len()
    );
    let undocumented: Vec<&String> = emitted.iter().filter(|r| !schema.contains(*r)).collect();
    assert!(
        undocumented.is_empty(),
        "reasons the binary emits that the schema description does not name: {undocumented:?}"
    );
    let unpinned: Vec<&String> = emitted
        .iter()
        .filter(|r| !captured.contains(*r) && !listed.contains(r.as_str()))
        .collect();
    assert!(
        unpinned.is_empty(),
        "reasons the binary emits with neither a golden fixture nor an untriggerable entry: {unpinned:?}"
    );

    let dry_run: BTreeSet<String> = dry_run_cases()
        .iter()
        .filter_map(|c| c.name.strip_prefix("dry-run-").map(str::to_string))
        .collect();
    let validators = validator_reasons();
    assert!(
        validators.len() >= 5,
        "validator scan is broken: {validators:?}"
    );
    let unpinned: Vec<&String> = validators
        .iter()
        .filter(|r| !dry_run.contains(*r))
        .collect();
    assert!(
        unpinned.is_empty(),
        "validator refusals without a dry-run fixture: {unpinned:?}"
    );
}
