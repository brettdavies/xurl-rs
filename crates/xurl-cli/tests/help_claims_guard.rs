//! What the global help says about which commands a flag reaches, held to
//! what each command sends.
//!
//! `--limit` is the page size of every command with `-n/--max-results`, and
//! `--cursor` reaches every command whose endpoint declares a page token in
//! the vendored spec. Every command that takes `--auth` runs for real, in
//! process, against a server that answers every endpoint, with both flags
//! set; each request it sends is matched to the spec's path template for its
//! endpoint. The command list and each command's arguments are read from
//! clap, so a command added later is checked without an edit here.

mod common;

use std::collections::BTreeSet;

use clap::CommandFactory;
use serde_json::{Value, json};
use xdk::testing::{MockX, Received};

use common::invoke::{
    arguments_for, bearer_only, live_oauth2_only, oauth1_only, run_xr, runnable_paths,
    takes_a_credential,
};

/// The page size the walk asks for: inside every `-n` range the CLI
/// declares (1-50, 5-100, 10-100, 1-100), so no clamp moves it.
const PAGE_SIZE: &str = "37";
const CURSOR: &str = "CURSOR-TOKEN";

/// The spec parameters that carry a page token.
const PAGE_TOKEN_PARAMS: &[&str] = &["pagination_token", "next_token"];

/// One spec operation: its method, path template, and the query parameters
/// that size a page and carry its token.
struct Operation {
    method: String,
    template: String,
    pattern: regex::Regex,
    size_params: Vec<String>,
    token_params: Vec<String>,
}

/// Every operation in the vendored spec.
fn operations() -> Vec<Operation> {
    let path = common::workspace_root().join("crates/xdk/vendor/x-api-openapi.json");
    let spec: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("the vendored spec"))
            .expect("the vendored spec is JSON");
    let shared = &spec["components"]["parameters"];
    let parameter = regex::Regex::new(r"\{[^}]+\}").expect("a pattern");
    let mut out = Vec::new();
    for (template, item) in spec["paths"].as_object().expect("paths") {
        for (method, operation) in item.as_object().expect("a path item") {
            let parameters = operation
                .get("parameters")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let names: Vec<String> = parameters
                .iter()
                .filter_map(|parameter| {
                    let parameter = match parameter["$ref"].as_str() {
                        Some(reference) => &shared[reference.rsplit('/').next()?],
                        None => parameter,
                    };
                    parameter["name"].as_str().map(str::to_string)
                })
                .collect();
            let widened = parameter.replace_all(template, "[^/]+");
            out.push(Operation {
                method: method.to_uppercase(),
                template: template.clone(),
                pattern: regex::Regex::new(&format!("^{widened}$")).expect("a template pattern"),
                size_params: names
                    .iter()
                    .filter(|name| name.starts_with("max_"))
                    .cloned()
                    .collect(),
                token_params: names
                    .iter()
                    .filter(|name| PAGE_TOKEN_PARAMS.contains(&name.as_str()))
                    .cloned()
                    .collect(),
            });
        }
    }
    out
}

/// The operation a request reached: the matching template with the fewest
/// parameters, so `/2/users/me/followers` wins over `/2/users/{id}/followers`.
fn operation_of<'a>(operations: &'a [Operation], request: &Received) -> Option<&'a Operation> {
    operations
        .iter()
        .filter(|op| op.method == request.method && op.pattern.is_match(&request.path))
        .min_by_key(|op| op.template.matches('{').count())
}

/// The value of `name` in a request's query string.
fn query_value(request: &Received, name: &str) -> Option<String> {
    url::form_urlencoded::parse(request.query.as_deref().unwrap_or_default().as_bytes())
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.into_owned())
}

/// Whether the command at `path` declares `-n/--max-results`.
fn takes_a_page_size(path: &str) -> bool {
    let mut root = xurl::cli::Cli::command();
    root.build();
    let cmd = path.split(' ').fold(&root, |cmd, name| {
        cmd.find_subcommand(name)
            .unwrap_or_else(|| panic!("clap has no command {path:?}"))
    });
    cmd.get_arguments().any(|arg| arg.get_id() == "max_results")
}

fn every_kind(ts: &mut xdk::store::TokenStore) {
    live_oauth2_only(ts);
    oauth1_only(ts);
    bearer_only(ts);
}

async fn answering_mock() -> MockX {
    let mock = MockX::start().await;
    // The media fixtures report processing under way, which a real upload
    // waits on. A finished one has nothing to wait for.
    let done = json!({"data": {"id": "1234567890", "processing_info": {"state": "succeeded"}}});
    mock.stub("POST", "/finalize$", 200, done.clone()).await;
    mock.stub("GET", "^/2/media/upload$", 200, done).await;
    mock
}

#[tokio::test]
async fn the_paging_flags_reach_exactly_the_commands_their_help_names() {
    let mock = answering_mock().await;
    let origin = mock.base_url();
    let operations = operations();
    let mut found = Vec::new();

    let paths: BTreeSet<String> = runnable_paths()
        .into_iter()
        .filter(|path| takes_a_credential(path))
        .collect();
    assert!(!paths.is_empty(), "no command takes --auth");
    for path in &paths {
        let mut args = arguments_for(path);
        args.extend(["--limit", PAGE_SIZE, "--cursor", CURSOR].map(str::to_string));
        let seen = mock.requests().await.len();
        let ran = run_xr(&origin, every_kind, path, &args, false).await;
        let sent = mock.requests().await.split_off(seen);
        if sent.is_empty() {
            found.push(format!(
                "{path}: sent no request (exit {:?}): {}\n  \
                 Cause: the synthesized arguments stop the command before it sends.\n  \
                 Fix: give it an OVERRIDES row in crates/xurl-cli/tests/common/invoke.rs.",
                ran.code,
                String::from_utf8_lossy(&ran.stderr).trim()
            ));
            continue;
        }

        let mut sized = false;
        let mut paged = false;
        let mut carried_cursor = false;
        for request in &sent {
            let Some(operation) = operation_of(&operations, request) else {
                found.push(format!(
                    "{path}: {} {} matches no operation in the vendored spec",
                    request.method, request.path
                ));
                continue;
            };
            sized |= operation
                .size_params
                .iter()
                .any(|name| query_value(request, name).as_deref() == Some(PAGE_SIZE));
            paged |= !operation.token_params.is_empty();
            carried_cursor |= operation
                .token_params
                .iter()
                .any(|name| query_value(request, name).as_deref() == Some(CURSOR));
        }
        if takes_a_page_size(path) && !sized {
            found.push(format!(
                "{path}: takes -n/--max-results, but --limit {PAGE_SIZE} did not reach its page size.\n  \
                 Cause: its arm reads -n alone, so the global --limit never applies.\n  \
                 Fix: resolve the size from -n, then --limit, in its arm under \
                 crates/xurl-cli/src/cli/commands/, as list_paged does with effective_limit."
            ));
        }
        if paged && !carried_cursor {
            found.push(format!(
                "{path}: its endpoint declares a page token, but --cursor did not reach it.\n  \
                 Cause: its arm builds the request without the cursor (send_validated drops it).\n  \
                 Fix: route it through list_paged or list_owned_by in \
                 crates/xurl-cli/src/cli/commands/mod.rs."
            ));
        }
        if !paged && carried_cursor {
            found.push(format!(
                "{path}: sent --cursor to an endpoint whose spec declares no page token.\n  \
                 Cause: its arm threads the cursor into a request that does not page.\n  \
                 Fix: send that request without the cursor."
            ));
        }
    }

    let seen = mock.requests().await.len();
    let raw: Vec<String> = [
        "--limit",
        PAGE_SIZE,
        "--cursor",
        CURSOR,
        "/2/users/me/followers",
    ]
    .map(str::to_string)
    .to_vec();
    run_xr(&origin, every_kind, "raw GET", &raw, false).await;
    for request in mock.requests().await.split_off(seen) {
        let query = request.query.unwrap_or_default();
        if query.contains(PAGE_SIZE) || query.contains(CURSOR) {
            found.push(format!(
                "raw GET: a raw request carried a paging flag: {} {}?{query}",
                request.method, request.path
            ));
        }
    }

    assert!(
        found.is_empty(),
        "the --limit and --cursor help no longer describes what these commands send:\n{}",
        found.join("\n")
    );
}

/// The largest page size a command's `-n/--max-results` help states, read
/// from its `(min-max` range; `None` for a command without `-n`.
fn stated_page_size_max(path: &str) -> Option<u32> {
    let mut root = xurl::cli::Cli::command();
    root.build();
    let cmd = path.split(' ').fold(&root, |cmd, name| {
        cmd.find_subcommand(name)
            .unwrap_or_else(|| panic!("clap has no command {path:?}"))
    });
    let help = cmd
        .get_arguments()
        .find(|arg| arg.get_id() == "max_results")?
        .get_help()?
        .to_string();
    let range = regex::Regex::new(r"\((\d+)-(\d+)").expect("a pattern");
    let found = range.captures(&help)?;
    found[2].parse().ok()
}

#[tokio::test]
async fn each_page_size_help_states_the_largest_size_the_command_sends() {
    let mock = answering_mock().await;
    let origin = mock.base_url();
    let operations = operations();
    let mut found = Vec::new();

    for path in runnable_paths()
        .into_iter()
        .filter(|path| takes_a_credential(path))
    {
        let Some(max) = stated_page_size_max(&path) else {
            continue;
        };
        let mut args = arguments_for(&path);
        args.extend(["-n".to_string(), max.to_string()]);
        let seen = mock.requests().await.len();
        let ran = run_xr(&origin, every_kind, &path, &args, false).await;
        let sent = mock.requests().await.split_off(seen);
        let sizes: Vec<String> = sent
            .iter()
            .filter_map(|request| {
                let operation = operation_of(&operations, request)?;
                operation
                    .size_params
                    .iter()
                    .find_map(|name| query_value(request, name))
            })
            .collect();
        if sizes != [max.to_string()] {
            found.push(format!(
                "{path}: its -n help states sizes up to {max}, but -n {max} sent {sizes:?} \
                 (exit {:?}).\n  \
                 Cause: the command clamps the size below the range its help states.\n  \
                 Fix: state the range the command sends in its -n help in \
                 crates/xurl-cli/src/cli/mod.rs, or lift the clamp.",
                ran.code
            ));
        }
    }

    assert!(
        found.is_empty(),
        "the -n/--max-results help states a range the command does not send:\n{}",
        found.join("\n")
    );
}

/// Each paging flag's first line, the only line `-h` prints, states its rule;
/// neither help names a command, since a list of names is what went stale.
#[test]
fn the_paging_flag_help_states_its_rule_on_its_first_line() {
    let root = xurl::cli::Cli::command();
    let subcommands: BTreeSet<&str> = root
        .get_subcommands()
        .map(clap::Command::get_name)
        .collect();
    for (flag, rule) in [("limit", "-n/--max-results"), ("cursor", "meta.next_token")] {
        let arg = root
            .get_arguments()
            .find(|arg| arg.get_id() == flag)
            .unwrap_or_else(|| panic!("--{flag} is declared"));
        let short = arg.get_help().map(ToString::to_string).unwrap_or_default();
        assert!(
            short.contains(rule),
            "--{flag} -h does not state its rule: {short}"
        );
        assert!(
            short.contains("raw"),
            "--{flag} -h does not mention raw requests: {short}"
        );
        let long = arg
            .get_long_help()
            .map(ToString::to_string)
            .unwrap_or_default();
        let named: Vec<&str> = long
            .split('`')
            .skip(1)
            .step_by(2)
            .filter(|word| subcommands.contains(word))
            .collect();
        assert!(
            named.is_empty(),
            "--{flag} help names commands {named:?}: {long}"
        );
    }
}
