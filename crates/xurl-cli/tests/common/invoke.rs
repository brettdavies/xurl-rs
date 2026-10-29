//! Running any `xr` command in-process with arguments read from clap: the
//! invocation the dry-run guard and the help-claims walks share.

use std::collections::{BTreeMap, BTreeSet};

use clap::CommandFactory;
use serde_json::Value;
use tempfile::TempDir;
use xdk::config::EnvOverrides;
use xdk::store::TokenStore;
use xurl::cli::skill_install::SkillEnv;

/// The value every synthesized argument takes. The seeded app and its
/// `OAuth2` user carry the same name, so a command that names an app or a
/// user reaches the point where it would act on one that exists.
pub const SAMPLE: &str = "1234567890";

/// Arguments for the commands whose useful invocation clap's declarations do
/// not spell out: a choice among optional flags, or a value with a meaning.
/// Every other command's arguments are synthesized by [`arguments_for`].
pub const OVERRIDES: &[(&str, &[&str])] = &[
    ("auth clear", &["--all", "--force"]),
    ("auth apps update", &[SAMPLE, "--client-secret", "rotated"]),
    (
        "auth apps redirect-uri set",
        &[SAMPLE, "http://localhost:8080/callback"],
    ),
    ("validate", &["{file}", "--schema", "envelope"]),
    ("schema", &["--list"]),
    ("skill install", &["claude_code"]),
    ("skill update", &["claude_code"]),
    ("webhooks add", &["https://example.com/webhook"]),
    (
        "webhooks replay",
        &[SAMPLE, "--from", "202601150000", "--to", "202601151200"],
    ),
    (
        "analytics",
        &[
            SAMPLE,
            "--start",
            "2026-10-01T00:00:00Z",
            "--end",
            "2026-10-02T00:00:00Z",
        ],
    ),
    ("calls scheduled add", &["--start", "2026-11-02T16:00:00Z"]),
    (
        "conversations start",
        &[SAMPLE, "9876543210", "--text", SAMPLE],
    ),
    ("conversations send", &["123-456", SAMPLE]),
    ("conversations history", &["123-456"]),
    ("lists update", &[SAMPLE, "--name", "Rust"]),
    ("media get", &["3_1234567890"]),
    (
        "media analytics",
        &[
            "3_1234567890",
            "--start",
            "2026-10-01T00:00:00Z",
            "--end",
            "2026-10-02T00:00:00Z",
        ],
    ),
    (
        "notes write",
        &[
            SAMPLE,
            "--text",
            "https://example.com/source",
            "--classification",
            "not_misleading",
        ],
    ),
    (
        "broadcasts scheduled add",
        &["--source", SAMPLE, "--start", "2026-11-02T16:00:00Z"],
    ),
    (
        "broadcasts scheduled update",
        &[
            SAMPLE,
            "--start",
            "2026-11-02T16:00:00Z",
            "--end",
            "2026-11-02T17:00:00Z",
        ],
    ),
    ("calls scheduled update", &[SAMPLE, "--title", "Retro"]),
];

/// Arguments that let the command at `path` parse and reach the point where
/// it would act: each required positional and option, one member of each
/// required group, and `--force` where the command takes it.
pub fn arguments_for(path: &str) -> Vec<String> {
    if let Some((_, args)) = OVERRIDES.iter().find(|(name, _)| *name == path) {
        return args.iter().map(|arg| (*arg).to_string()).collect();
    }
    let mut root = xurl::cli::Cli::command();
    root.build();
    let cmd = path.split(' ').fold(&root, |cmd, name| {
        cmd.find_subcommand(name)
            .unwrap_or_else(|| panic!("clap has no command {path:?}"))
    });
    let sample = |arg: &clap::Arg| {
        let id = arg.get_id().as_str();
        if let Some(value) = arg.get_possible_values().first() {
            value.get_name().to_string()
        } else if id.contains("file") {
            "{file}".to_string()
        } else if id.contains("language") {
            "en".to_string()
        } else {
            SAMPLE.to_string()
        }
    };
    let flag = |arg: &clap::Arg| {
        format!(
            "--{}",
            arg.get_long().expect("a required option has a long name")
        )
    };

    let mut args = Vec::new();
    let mut given: BTreeSet<String> = BTreeSet::new();
    for arg in cmd.get_arguments().filter(|arg| !arg.is_global_set()) {
        if arg.is_positional() && arg.is_required_set() {
            args.push(sample(arg));
        } else if arg.is_required_set() {
            args.extend([flag(arg), sample(arg)]);
        } else if arg.get_id() == "force" {
            args.push("--force".to_string());
        } else {
            continue;
        }
        given.insert(arg.get_id().to_string());
    }
    for group in cmd.get_groups().filter(|group| group.is_required_set()) {
        let members: Vec<&clap::Arg> = group
            .get_args()
            .filter_map(|id| cmd.get_arguments().find(|arg| arg.get_id() == id))
            .collect();
        if members
            .iter()
            .any(|arg| given.contains(arg.get_id().as_str()))
        {
            continue;
        }
        // The plain flag of a secret pair, not its file twin, so no file is read.
        let member = members
            .iter()
            .find(|arg| !arg.get_id().as_str().ends_with("_file"))
            .or(members.first())
            .unwrap_or_else(|| panic!("{path}: a required group has no member"));
        args.extend([flag(member), sample(member)]);
    }
    args
}

/// Every path that runs on its own: each command with no subcommand, and
/// `usage`, which runs with or without one.
pub fn runnable_paths() -> BTreeSet<String> {
    let paths = super::command_paths();
    let mut runnable: BTreeSet<String> = paths
        .iter()
        .filter(|path| {
            !paths
                .iter()
                .any(|other| other.len() > path.len() && other.starts_with(path))
        })
        .map(|path| path.join(" "))
        .filter(|path| !path.ends_with("help"))
        .collect();
    runnable.insert("usage".to_string());
    runnable
}

/// What a test stores under the one app.
pub type Fill = fn(&mut TokenStore);

/// Every credential kind, with the `OAuth2` login expired so that building
/// its header would refresh it.
pub fn every_kind_with_the_login_expired(ts: &mut TokenStore) {
    ts.save_oauth2_token_for_app(SAMPLE, SAMPLE, "EXPIRED-ACCESS", "REFRESH", 1)
        .expect("save oauth2");
    oauth1_only(ts);
    bearer_only(ts);
}

pub fn nothing(_: &mut TokenStore) {}

pub fn bearer_only(ts: &mut TokenStore) {
    ts.save_bearer_token_for_app(SAMPLE, "BEARER")
        .expect("save bearer");
}

pub fn oauth1_only(ts: &mut TokenStore) {
    ts.save_oauth1_tokens_for_app(
        SAMPLE,
        "OA1-ACCESS",
        "OA1-SECRET",
        "OA1-KEY",
        "OA1-CONSUMER",
    )
    .expect("save oauth1");
}

pub fn live_oauth2_only(ts: &mut TokenStore) {
    let in_an_hour = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock is past the epoch")
        .as_secs()
        + 3600;
    ts.save_oauth2_token_for_app(SAMPLE, SAMPLE, "LIVE-ACCESS", "REFRESH", in_an_hour)
        .expect("save oauth2");
}

/// One in-process run of `xr`.
pub struct Ran {
    /// The exit code, or `None` for a run that did not return.
    pub code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub store_changed: bool,
    /// Entries the run left in the skill home.
    pub installed: usize,
}

impl Ran {
    /// The document on `stream`, or `Null` when it holds none.
    pub fn document(stream: &[u8]) -> Value {
        serde_json::from_slice(stream).unwrap_or_default()
    }
}

/// Runs `name` with `args` over a store `fill` seeds, every URL the run
/// could reach pointed at `origin`.
pub async fn run_xr(origin: &str, fill: Fill, name: &str, args: &[String], dry_run: bool) -> Ran {
    let tmp = TempDir::new().expect("tempdir");
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(&store.to_string_lossy());
    ts.add_app(SAMPLE, "CLIENT-ID", "CLIENT-SECRET")
        .expect("add app");
    fill(&mut ts);
    let before = std::fs::read(&store).expect("read the store");
    let file = tmp.path().join("input.json");
    std::fs::write(&file, b"{\"status\":\"ok\"}").expect("write the input file");
    let skill_home = tmp.path().join("skill-home");
    std::fs::create_dir(&skill_home).expect("create the skill home");

    let overrides = EnvOverrides {
        api_base_url: Some(origin.to_string()),
        auth_url: Some(format!("{origin}/authorize")),
        token_url: Some(format!("{origin}/token")),
        info_url: Some(format!("{origin}/me")),
        ..EnvOverrides::default()
    };
    let skill_env = SkillEnv {
        home: None,
        skill_home: Some(skill_home.to_string_lossy().into_owned()),
        config_dirs: BTreeMap::new(),
    };
    let file_arg = file.to_string_lossy().into_owned();
    let mut argv = vec!["xr".to_string()];
    if dry_run {
        argv.push("--dry-run".to_string());
    }
    argv.extend(["--output".to_string(), "json".to_string()]);
    if !name.starts_with("raw ") {
        argv.extend(name.split(' ').map(str::to_string));
    }
    argv.extend(args.iter().map(|arg| arg.replace("{file}", &file_arg)));

    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let run = xurl::cli::runner::run_with_env(
        argv.iter(),
        &mut stdout,
        &mut stderr,
        &store,
        &overrides,
        &skill_env,
    );
    // A run against a local server has nothing to wait for. One that does
    // not return is waiting on a listener, a prompt, or a stream.
    let code = tokio::time::timeout(std::time::Duration::from_secs(10), run)
        .await
        .ok();
    Ran {
        code,
        store_changed: std::fs::read(&store).expect("read the store") != before,
        installed: std::fs::read_dir(&skill_home)
            .expect("read the skill home")
            .count(),
        stdout,
        stderr,
    }
}

/// Whether the command at `path` sends a request under a credential: the
/// ones that do take `--auth`.
pub fn takes_a_credential(path: &str) -> bool {
    let mut root = xurl::cli::Cli::command();
    root.build();
    let cmd = path.split(' ').fold(&root, |cmd, name| {
        cmd.find_subcommand(name)
            .unwrap_or_else(|| panic!("clap has no command {path:?}"))
    });
    cmd.get_arguments().any(|arg| arg.get_id() == "auth_type")
}
