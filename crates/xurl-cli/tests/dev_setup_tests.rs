//! `.github/tool-versions` is the one place a tool's version is written. CI's
//! install steps read it through `scripts/tool-version.sh`, so `ci.yml` can
//! never install a tool at a version a contributor's machine was not given.
//!
//! The tests that run `scripts/dev-setup.sh` build a machine out of stubs. They
//! are serial because running a stub just written can fail with `ETXTBSY` while
//! a fork on another thread still holds its write descriptor.
#![cfg(unix)]

mod common;

use std::collections::BTreeMap;

use regex::Regex;
use rstest::rstest;
use serde::Deserialize;

const LIST: &str = ".github/tool-versions";
const CI: &str = ".github/workflows/ci.yml";

/// A command position in a shell line, so an installer named inside an
/// `::error` message or an `echo` is not read as an install.
const COMMAND_START: &str = r"(?m)(?:^|\|\||&&|;|\bthen\b|\bdo\b|\()\s*";

/// A `timeout [OPTION]... DURATION` in front of an install, which ends a hung
/// download and leaves the install a command of its own.
const DEADLINE: &str = r"(?:timeout\b[^|&;\n]*?\s)?";

#[derive(Deserialize)]
struct Workflow {
    jobs: BTreeMap<String, Job>,
}

#[derive(Deserialize)]
struct Job {
    #[serde(default)]
    outputs: BTreeMap<String, String>,
    #[serde(default)]
    steps: Vec<Step>,
}

#[derive(Deserialize)]
struct Step {
    name: Option<String>,
    uses: Option<String>,
    run: Option<String>,
    #[serde(rename = "if")]
    condition: Option<String>,
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(common::workspace_root().join(relative))
        .unwrap_or_else(|e| panic!("read {relative}: {e}"))
}

fn parse_list(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (name, version) = line
                .split_once('=')
                .unwrap_or_else(|| panic!("{LIST}: no `=` in {line:?}"));
            (name.to_string(), version.to_string())
        })
        .collect()
}

/// Each `(name, pin)` an install's arguments ask for. A `--path` install builds
/// this repository's own code, so it asks for nothing.
fn requested(arguments: &str) -> Vec<(String, Option<String>)> {
    let tokens: Vec<&str> = arguments
        .split_whitespace()
        .map(|token| token.trim_matches(|c| c == '"' || c == '\''))
        .collect();
    if tokens.contains(&"--path") {
        return Vec::new();
    }
    let mut flag_pin = None;
    let mut specs = Vec::new();
    let mut tokens = tokens.into_iter();
    while let Some(token) = tokens.next() {
        if token == "--version" || token == "--vers" {
            flag_pin = tokens.next().map(str::to_string);
        } else if let Some(pin) = token.strip_prefix("--version=") {
            flag_pin = Some(pin.to_string());
        } else if token == "--disable-strategies" {
            tokens.next();
        } else if !token.starts_with('-') {
            specs.push(token);
        }
    }
    specs
        .into_iter()
        .map(|spec| match spec.split_once('@') {
            Some((name, pin)) => (name.to_string(), Some(pin.to_string())),
            None => (spec.to_string(), flag_pin.clone()),
        })
        .collect()
}

/// What each step of `workflow` installs that the list does not own, one
/// message per finding.
fn violations(workflow: &str, list: &BTreeMap<String, String>) -> Vec<String> {
    let parsed: Workflow = serde_saphyr::from_str(workflow).expect("the workflow is YAML");
    let install = Regex::new(&format!(
        r"{COMMAND_START}{DEADLINE}cargo\s+(?:binstall|install)\b([^|&;\n]*)"
    ))
    .expect("valid pattern");
    let download = Regex::new(&format!(
        r"{COMMAND_START}(?:curl|wget|gh\s+release\s+download)\b"
    ))
    .expect("valid pattern");
    let reads =
        Regex::new(r#"(\w+)="?\$\((?:bash )?(?:\./)?scripts/tool-version\.sh ([\w.-]+)\)"?"#)
            .expect("valid pattern");

    let mut found = Vec::new();
    for (job_id, job) in &parsed.jobs {
        for step in &job.steps {
            let name = step.name.as_deref().or(step.uses.as_deref()).unwrap_or("?");
            let place = format!("step `{name}` of job `{job_id}` in {CI}");
            if step
                .uses
                .as_deref()
                .is_some_and(|uses| uses.contains("/install-action@"))
            {
                found.push(format!(
                    "tool-versions/unpinned-install: {place} installs through an action the list \
                     cannot pin. Install with `cargo binstall`, reading the version with \
                     `scripts/tool-version.sh`."
                ));
            }
            let Some(run) = &step.run else { continue };
            let script = run
                .replace("\\\n", " ")
                .lines()
                .filter(|line| !line.trim_start().starts_with('#'))
                .collect::<Vec<_>>()
                .join("\n");
            let read_for: BTreeMap<&str, &str> = reads
                .captures_iter(&script)
                .map(|read| {
                    (
                        read.get(1).expect("group").as_str(),
                        read.get(2).expect("group").as_str(),
                    )
                })
                .collect();
            if download.is_match(&script) && read_for.is_empty() {
                found.push(format!(
                    "tool-versions/unpinned-download: {place} downloads without reading a version \
                     from {LIST}. Add the tool to the list and read it with \
                     `scripts/tool-version.sh`."
                ));
            }
            for arguments in install.captures_iter(&script) {
                for (tool, pin) in requested(&arguments[1]) {
                    found.extend(judge_install(
                        &place,
                        &tool,
                        pin.as_deref(),
                        &read_for,
                        list,
                    ));
                }
            }
        }
    }
    // A step's `binstall || binstall || install` chain repeats one finding.
    found.dedup();
    found
}

/// The finding for one requested install, if it has one.
fn judge_install(
    place: &str,
    tool: &str,
    pin: Option<&str>,
    read_for: &BTreeMap<&str, &str>,
    list: &BTreeMap<String, String>,
) -> Option<String> {
    let fix = format!(
        "In that step set `version=\"$(bash scripts/tool-version.sh {tool})\"` and install \
         `{tool}@${{version}}`."
    );
    let Some(listed) = list.get(tool) else {
        return Some(format!(
            "tool-versions/unlisted-tool: {place} installs {tool}, which {LIST} does not name. \
             Add a `{tool}=<version>` line to {LIST}. {fix}"
        ));
    };
    let Some(pin) = pin else {
        return Some(format!(
            "tool-versions/unpinned-install: {place} installs {tool} with no version, and {LIST} \
             names {listed}. {fix}"
        ));
    };
    let Some(variable) = pin.strip_prefix('$') else {
        return Some(format!(
            "tool-versions/literal-version: {place} pins {tool} at {pin}, and {LIST} names \
             {listed}. The list owns the version. {fix}"
        ));
    };
    let variable = variable.trim_start_matches('{').trim_end_matches('}');
    if read_for.get(variable) == Some(&tool) {
        return None;
    }
    Some(format!(
        "tool-versions/unread-version: {place} pins {tool} through `${variable}`, which the step \
         does not read from `scripts/tool-version.sh {tool}`. {fix}"
    ))
}

fn seed_list() -> BTreeMap<String, String> {
    parse_list("cargo-hack=1.2.3\nagentnative=4.5.6\n")
}

/// A workflow holding one step that runs `script`.
fn seed_workflow(script: &str) -> String {
    let body: String = script
        .lines()
        .map(|line| format!("          {line}\n"))
        .collect();
    format!("jobs:\n  seeded:\n    steps:\n      - name: Seeded step\n        run: |\n{body}")
}

#[test]
fn ci_installs_only_what_the_list_names() {
    let found = violations(&read(CI), &parse_list(&read(LIST)));
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[rstest]
#[case::no_version("cargo binstall cargo-hack --force", "tool-versions/unpinned-install")]
#[case::no_version_in_fallback(
    "version=\"$(bash scripts/tool-version.sh cargo-hack)\"\n\
     cargo binstall \"cargo-hack@${version}\" --force || cargo install cargo-hack --locked",
    "tool-versions/unpinned-install"
)]
#[case::continued_line(
    "version=\"$(bash scripts/tool-version.sh cargo-hack)\"\n\
     cargo binstall \"cargo-hack@${version}\" \\\n  || cargo install cargo-hack --locked",
    "tool-versions/unpinned-install"
)]
#[case::wrong_literal(
    "cargo binstall cargo-hack@9.9.9 --force",
    "tool-versions/literal-version"
)]
#[case::matching_literal(
    "cargo binstall cargo-hack@1.2.3 --force",
    "tool-versions/literal-version"
)]
#[case::version_flag(
    "cargo install cargo-hack --version 9.9.9 --locked",
    "tool-versions/literal-version"
)]
#[case::version_flag_with_equals(
    "cargo install cargo-hack --version=9.9.9 --locked",
    "tool-versions/literal-version"
)]
#[case::unlisted_tool("cargo binstall cargo-udeps@1.0.0", "tool-versions/unlisted-tool")]
#[case::variable_never_read(
    "cargo binstall \"cargo-hack@${version}\"",
    "tool-versions/unread-version"
)]
#[case::variable_read_for_another_tool(
    "version=\"$(bash scripts/tool-version.sh agentnative)\"\n\
     cargo binstall \"cargo-hack@${version}\"",
    "tool-versions/unread-version"
)]
#[case::literal_under_a_deadline(
    "timeout -k 5 45 cargo binstall cargo-hack@1.2.3 --force --disable-strategies compile \
     || status=$?",
    "tool-versions/literal-version"
)]
#[case::variable_never_read_under_a_deadline(
    "timeout 45 cargo binstall \"cargo-hack@${version}\"",
    "tool-versions/unread-version"
)]
#[case::download(
    "curl -sSL https://example.com/tool.tar.gz | tar xz",
    "tool-versions/unpinned-download"
)]
fn a_seeded_install_fails_naming_the_step_and_the_file(#[case] script: &str, #[case] code: &str) {
    let found = violations(&seed_workflow(script), &seed_list());
    let [message] = found.as_slice() else {
        panic!("expected one finding for {script:?}, got {found:#?}");
    };
    assert!(
        message.starts_with(code),
        "expected {code} for {script:?}, got {message}"
    );
    assert!(
        message.contains("step `Seeded step` of job `seeded`") && message.contains(CI),
        "the message must name the step and the file to edit: {message}"
    );
}

#[test]
fn an_install_action_fails_since_the_list_cannot_pin_it() {
    let workflow = "jobs:\n  seeded:\n    steps:\n      - name: Install tools\n        \
                    uses: taiki-e/install-action@0000000000000000000000000000000000000000\n";
    let found = violations(workflow, &seed_list());
    assert!(
        found
            .first()
            .is_some_and(|message| message.starts_with("tool-versions/unpinned-install")),
        "got {found:#?}"
    );
}

#[rstest]
#[case::read_from_the_list(
    "version=\"$(bash scripts/tool-version.sh cargo-hack)\"\n\
     cargo binstall \"cargo-hack@${version}\" --force --no-confirm \\\n  \
     || cargo install \"cargo-hack@${version}\" --locked"
)]
#[case::prebuilt_only_under_a_deadline(
    "version=\"$(bash scripts/tool-version.sh cargo-hack)\"\n\
     status=0\n\
     timeout -k 5 45 cargo binstall \"cargo-hack@${version}\" --force --no-confirm \
     --disable-strategies compile || status=$?"
)]
#[case::lines_after_the_install_are_not_its_arguments(
    "version=\"$(bash scripts/tool-version.sh cargo-hack)\"\n\
     cargo binstall \"cargo-hack@${version}\"\n\
     installed=\"$(cargo hack --version)\"\n\
     echo \"${installed}\""
)]
#[case::named_in_a_message("echo \"::error::run cargo install cargo-hack to reproduce\"")]
#[case::commented_out("# cargo binstall cargo-hack")]
#[case::own_code("cargo install --path crates/xurl-cli --locked")]
#[case::download_of_a_listed_version(
    "version=\"$(bash scripts/tool-version.sh cargo-hack)\"\n\
     curl -sSLO \"https://example.com/cargo-hack-${version}.tar.gz\""
)]
fn a_step_the_list_owns_passes(#[case] script: &str) {
    let found = violations(&seed_workflow(script), &seed_list());
    assert!(found.is_empty(), "{script:?} must pass, got {found:#?}");
}

#[test]
fn the_list_holds_one_exact_version_per_tool() {
    let exact = Regex::new(r"^[a-z0-9-]+=\d+\.\d+\.\d+$").expect("valid pattern");
    let mut seen = std::collections::BTreeSet::new();
    for line in read(LIST)
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        assert!(exact.is_match(line), "{LIST}: {line:?} is not `name=x.y.z`");
        let (name, _) = line.split_once('=').expect("checked above");
        assert!(seen.insert(name.to_string()), "{LIST} names {name} twice");
    }
}

/// A reusable workflow takes the version only as an input, and a warm run needs
/// it as much as a pull request does, so the read cannot depend on the event.
#[test]
fn the_scope_job_outputs_the_nextest_version_on_every_event() {
    let workflow: Workflow = serde_saphyr::from_str(&read(CI)).expect("the workflow is YAML");
    let scope = workflow.jobs.get("scope").expect("the scope job");
    assert_eq!(
        scope.outputs.get("nextest_version").map(String::as_str),
        Some("${{ steps.tools.outputs.nextest_version }}"),
        "the scope job in {CI} must output nextest_version from its `tools` step"
    );
    let read_step = scope
        .steps
        .iter()
        .find(|step| {
            step.run
                .as_deref()
                .is_some_and(|run| run.contains("nextest_version="))
        })
        .unwrap_or_else(|| panic!("no step of the scope job in {CI} writes nextest_version"));
    assert!(
        read_step
            .run
            .as_deref()
            .is_some_and(|run| run.contains("tool-version.sh cargo-nextest")),
        "the scope job in {CI} must read nextest_version through scripts/tool-version.sh"
    );
    for step in &scope.steps {
        let checks_out = step
            .uses
            .as_deref()
            .is_some_and(|uses| uses.contains("actions/checkout"));
        assert!(
            !checks_out || step.condition.is_none(),
            "the scope job's checkout in {CI} must run on every event, or a push run has no list \
             to read"
        );
    }
}

/// Stands in for cargo, Homebrew, and every listed tool. `$SANDBOX/installed/<tool>`
/// holds a tool's version, `available/<tool>` what Homebrew would give, a file at
/// `refuse/<tool>` fails its install, and `calls` records each install.
const STUB: &str = r#"#!/bin/sh
name=$(basename "$0")
say_version() {
  [ -f "$SANDBOX/installed/$1" ] || exit 101
  printf '%s%s%s\n' "$2" "$(cat "$SANDBOX/installed/$1")" "$3"
}
put_in_place() {
  if [ -e "$SANDBOX/refuse/$1" ]; then
    echo "stub: refusing $1" >&2
    exit 1
  fi
  printf '%s' "$2" >"$SANDBOX/installed/$1"
}
case "$name" in
  cargo)
    case "$1" in
      binstall | install)
        if [ "$2" = -V ]; then
          say_version cargo-binstall "" ""
          exit
        fi
        echo "cargo $*" >>"$SANDBOX/calls"
        put_in_place "${2%@*}" "${2#*@}"
        ;;
      hack | deny | nextest) say_version "cargo-$1" "cargo-$1 " "" ;;
      *) exit 101 ;;
    esac
    ;;
  brew)
    echo "brew $*" >>"$SANDBOX/calls"
    put_in_place "$2" "$(cat "$SANDBOX/available/$2")"
    ;;
  anc) say_version agentnative "anc " "" ;;
  rg) say_version ripgrep "ripgrep " " (rev 0123456789)" ;;
  bash) say_version bash "GNU bash, version " "(1)-release" ;;
  bats) say_version bats-core "Bats " "" ;;
  shellcheck) printf 'ShellCheck\n'; say_version shellcheck "version: " "" ;;
  markdownlint-cli2)
    say_version markdownlint-cli2 "markdownlint-cli2 v" " (markdownlint v0.42.0)"
    exit 2
    ;;
  *) say_version "$name" "" "" ;;
esac
"#;

const STUBBED_COMMANDS: &[&str] = &[
    "cargo",
    "brew",
    "anc",
    "rg",
    "bash",
    "bats",
    "shellcheck",
    "markdownlint-cli2",
    "actionlint",
];

/// What the script and the stub run besides the stubbed commands. PATH holds these
/// and nothing else, so no real tool on the host can answer for a stubbed one.
const BASE_UTILITIES: &[&str] = &["basename", "cat", "dirname", "mktemp", "rm", "sort", "tail"];

/// A machine made of stubs, with nothing installed but cargo-binstall.
struct Machine {
    dir: tempfile::TempDir,
}

impl Machine {
    fn bare() -> Self {
        use std::os::unix::fs::{PermissionsExt, symlink};

        let dir = tempfile::tempdir().expect("tempdir");
        for sub in ["bin", "installed", "available", "refuse"] {
            std::fs::create_dir(dir.path().join(sub)).expect("create a machine directory");
        }
        let stub = dir.path().join("stub");
        std::fs::write(&stub, STUB).expect("write the stub");
        std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).expect("chmod");
        for command in STUBBED_COMMANDS {
            symlink(&stub, dir.path().join("bin").join(command)).expect("link a stub");
        }
        for utility in BASE_UTILITIES {
            let real = ["/usr/bin", "/bin"]
                .iter()
                .map(|parent| std::path::Path::new(parent).join(utility))
                .find(|path| path.exists())
                .unwrap_or_else(|| panic!("no {utility} in /usr/bin or /bin"));
            symlink(real, dir.path().join("bin").join(utility)).expect("link a utility");
        }
        let machine = Self { dir };
        machine.set_installed("cargo-binstall", "1.26.2");
        machine.set_available("cargo-binstall", "1.26.2");
        for (tool, version) in listed() {
            machine.set_available(&tool, &version);
        }
        machine
    }

    fn with_every_tool_current() -> Self {
        let machine = Self::bare();
        for (tool, version) in listed() {
            machine.set_installed(&tool, &version);
        }
        machine
    }

    fn state(&self, sub: &str, tool: &str) -> std::path::PathBuf {
        self.dir.path().join(sub).join(tool)
    }

    fn set_installed(&self, tool: &str, version: &str) {
        std::fs::write(self.state("installed", tool), version).expect("set a version");
    }

    fn set_available(&self, tool: &str, version: &str) {
        std::fs::write(self.state("available", tool), version).expect("set a version");
    }

    fn refuse(&self, tool: &str) {
        std::fs::write(self.state("refuse", tool), "").expect("refuse a tool");
    }

    fn remove_command(&self, command: &str) {
        std::fs::remove_file(self.state("bin", command)).expect("remove a command");
    }

    fn remove_installed(&self, tool: &str) {
        std::fs::remove_file(self.state("installed", tool)).expect("remove a tool");
    }

    fn installed(&self, tool: &str) -> Option<String> {
        std::fs::read_to_string(self.state("installed", tool)).ok()
    }

    fn calls(&self) -> Vec<String> {
        std::fs::read_to_string(self.dir.path().join("calls"))
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }

    fn run(&self, script: &std::path::Path, args: &[&str]) -> Run {
        let output = std::process::Command::new("/bin/bash")
            .arg(script)
            .args(args)
            .current_dir(self.dir.path())
            .env("PATH", self.dir.path().join("bin"))
            .env("SANDBOX", self.dir.path())
            .output()
            .expect("spawn the script");
        Run {
            code: output.status.code(),
            stdout: String::from_utf8(output.stdout).expect("stdout is UTF-8"),
            stderr: String::from_utf8(output.stderr).expect("stderr is UTF-8"),
        }
    }

    fn setup(&self, args: &[&str]) -> Run {
        self.run(&common::workspace_root().join("scripts/dev-setup.sh"), args)
    }
}

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Run {
    /// The line the script printed for `tool`.
    fn line(&self, tool: &str) -> &str {
        self.stdout
            .lines()
            .find(|line| line.split_whitespace().next() == Some(tool))
            .unwrap_or_else(|| panic!("no line for {tool} in:\n{}", self.stdout))
    }
}

fn listed() -> BTreeMap<String, String> {
    parse_list(&read(LIST))
}

#[test]
#[serial_test::serial]
fn a_bare_machine_ends_with_every_listed_tool_and_a_rerun_changes_nothing() {
    let machine = Machine::bare();
    let list = listed();

    let first = machine.setup(&[]);
    assert_eq!(first.code, Some(0), "stderr: {}", first.stderr);
    assert_eq!(first.stdout.lines().count(), list.len(), "{}", first.stdout);
    for (tool, version) in &list {
        assert_eq!(machine.installed(tool).as_deref(), Some(version.as_str()));
        assert!(
            first
                .line(tool)
                .contains(&format!("installed {version} (was missing)")),
            "{}",
            first.line(tool)
        );
    }
    let calls = machine.calls();
    let hack = format!(
        "cargo binstall cargo-hack@{} --force --no-confirm",
        list["cargo-hack"]
    );
    assert!(calls.contains(&hack), "{calls:#?}");
    assert!(
        calls.contains(&"brew install actionlint".to_string()),
        "{calls:#?}"
    );

    let second = machine.setup(&[]);
    assert_eq!(second.code, Some(0), "stderr: {}", second.stderr);
    assert_eq!(
        second.stdout.lines().count(),
        list.len(),
        "{}",
        second.stdout
    );
    for (tool, version) in &list {
        assert!(
            second.line(tool).contains(&format!("current {version}")),
            "{}",
            second.line(tool)
        );
    }
    assert_eq!(machine.calls(), calls, "a rerun must install nothing");
}

#[test]
#[serial_test::serial]
fn an_older_tool_is_replaced_and_a_newer_one_stays() {
    let machine = Machine::with_every_tool_current();
    let list = listed();
    // Each differs from its listed version only where a string compare gets it wrong.
    machine.set_installed("shellcheck", "0.9.0");
    machine.set_installed("ripgrep", "15.10.0");

    let run = machine.setup(&[]);
    assert_eq!(run.code, Some(0), "stderr: {}", run.stderr);
    assert_eq!(machine.calls(), ["brew install shellcheck"]);
    assert!(
        run.line("shellcheck")
            .contains(&format!("installed {} (was 0.9.0)", list["shellcheck"])),
        "{}",
        run.line("shellcheck")
    );
    assert!(
        run.line("ripgrep").contains(&format!(
            "current 15.10.0 (the list names {})",
            list["ripgrep"]
        )),
        "{}",
        run.line("ripgrep")
    );
    assert_eq!(machine.installed("ripgrep").as_deref(), Some("15.10.0"));
}

#[rstest]
#[case::rust_tool("cargo-hack", "cargo install cargo-hack@")]
#[case::linter("shellcheck", "brew install shellcheck")]
#[serial_test::serial]
fn a_failed_install_exits_1_naming_the_tool_and_the_manual_command(
    #[case] tool: &str,
    #[case] manual: &str,
) {
    let machine = Machine::bare();
    machine.refuse(tool);

    let run = machine.setup(&[]);
    assert_eq!(run.code, Some(1), "stdout: {}", run.stdout);
    assert!(
        run.stderr.contains(&format!("dev-setup/{tool}: ")),
        "{}",
        run.stderr
    );
    assert!(
        run.stderr.contains(&format!("install via `{manual}")),
        "{}",
        run.stderr
    );
    assert!(
        run.stderr.contains("stub: refusing"),
        "the installer's own output: {}",
        run.stderr
    );
    assert_eq!(
        machine.installed("actionlint").as_deref(),
        listed().get("actionlint").map(String::as_str),
        "a failure must not stop the other tools"
    );
}

#[test]
#[serial_test::serial]
fn an_install_that_leaves_an_older_version_on_path_fails() {
    let machine = Machine::bare();
    machine.set_available("shellcheck", "0.0.1");

    let run = machine.setup(&[]);
    assert_eq!(run.code, Some(1), "stdout: {}", run.stdout);
    let line = run
        .stderr
        .lines()
        .find(|line| line.starts_with("dev-setup/shellcheck: "))
        .unwrap_or_else(|| panic!("{}", run.stderr));
    assert!(line.contains("reports 0.0.1"), "{line}");
    assert!(
        line.contains("install via `brew install shellcheck`"),
        "{line}"
    );
}

#[test]
#[serial_test::serial]
fn without_homebrew_a_missing_linter_stops_and_rust_tools_still_install() {
    let machine = Machine::bare();
    machine.remove_command("brew");
    let list = listed();

    let run = machine.setup(&[]);
    assert_eq!(run.code, Some(1), "stdout: {}", run.stdout);
    assert!(
        run.stderr.contains("dev-setup/actionlint: "),
        "{}",
        run.stderr
    );
    assert!(run.stderr.contains("https://brew.sh"), "{}", run.stderr);
    assert_eq!(
        machine.installed("cargo-hack").as_ref(),
        list.get("cargo-hack")
    );
}

#[test]
#[serial_test::serial]
fn without_cargo_it_stops_before_touching_anything() {
    let machine = Machine::bare();
    machine.remove_command("cargo");

    let run = machine.setup(&[]);
    assert_eq!(run.code, Some(1), "stdout: {}", run.stdout);
    assert!(run.stderr.contains("dev-setup/cargo: "), "{}", run.stderr);
    assert!(
        run.stderr.contains("https://sh.rustup.rs"),
        "{}",
        run.stderr
    );
    assert!(machine.calls().is_empty(), "{:#?}", machine.calls());
}

#[test]
#[serial_test::serial]
fn without_binstall_homebrew_supplies_it() {
    let machine = Machine::bare();
    machine.remove_installed("cargo-binstall");

    let run = machine.setup(&[]);
    assert_eq!(run.code, Some(0), "stderr: {}", run.stderr);
    let calls = machine.calls();
    assert_eq!(
        calls.first().map(String::as_str),
        Some("brew install cargo-binstall")
    );
    assert!(
        calls
            .iter()
            .any(|call| call.starts_with("cargo binstall cargo-hack@")),
        "{calls:#?}"
    );
    assert!(
        run.line("cargo-binstall").contains("installed"),
        "{}",
        run.stdout
    );
}

#[test]
#[serial_test::serial]
fn without_binstall_or_homebrew_rust_tools_build_from_source() {
    let machine = Machine::bare();
    machine.remove_installed("cargo-binstall");
    machine.remove_command("brew");
    let list = listed();

    let run = machine.setup(&[]);
    let source = format!(
        "cargo install cargo-hack@{} --locked --force",
        list["cargo-hack"]
    );
    assert!(machine.calls().contains(&source), "{:#?}", machine.calls());
    assert_eq!(
        machine.installed("cargo-hack").as_ref(),
        list.get("cargo-hack")
    );
    assert!(
        run.line("cargo-binstall").contains("build from source"),
        "{}",
        run.stdout
    );
}

#[test]
#[serial_test::serial]
fn a_dry_run_reports_and_changes_nothing() {
    let machine = Machine::bare();

    let run = machine.setup(&["--dry-run"]);
    assert_eq!(run.code, Some(0), "stderr: {}", run.stderr);
    for (tool, version) in listed() {
        assert!(
            run.line(&tool)
                .contains(&format!("would install {version} (now missing)")),
            "{}",
            run.line(&tool)
        );
        assert_eq!(machine.installed(&tool), None);
    }
    assert!(machine.calls().is_empty(), "{:#?}", machine.calls());
}

#[test]
#[serial_test::serial]
fn a_root_takes_the_rust_tools_and_homebrew_stays_untouched() {
    let machine = Machine::bare();
    let list = listed();

    let run = machine.setup(&["--root", "scratch"]);
    assert_eq!(run.code, Some(0), "stderr: {}", run.stderr);
    let root = machine.dir.path().join("scratch");
    let expected = format!(
        "cargo binstall cargo-hack@{} --force --no-confirm --root {}",
        list["cargo-hack"],
        root.display()
    );
    assert!(
        machine.calls().contains(&expected),
        "{:#?}",
        machine.calls()
    );
    assert!(
        !machine.calls().iter().any(|call| call.starts_with("brew ")),
        "{:#?}",
        machine.calls()
    );
    assert!(
        run.line("actionlint").contains("would install"),
        "{}",
        run.line("actionlint")
    );
    assert!(
        run.line("actionlint").contains("no install root"),
        "{}",
        run.line("actionlint")
    );
}

#[test]
#[serial_test::serial]
fn a_listed_tool_with_no_recipe_stops_naming_the_script() {
    let machine = Machine::with_every_tool_current();
    let repo = machine.dir.path().join("repo");
    std::fs::create_dir_all(repo.join("scripts")).expect("mkdir");
    std::fs::create_dir_all(repo.join(".github")).expect("mkdir");
    let script = repo.join("scripts/dev-setup.sh");
    std::fs::copy(
        common::workspace_root().join("scripts/dev-setup.sh"),
        &script,
    )
    .expect("copy");
    std::fs::write(repo.join(".github/tool-versions"), "new-linter=1.0.0\n").expect("seed a list");

    let run = machine.run(&script, &[]);
    assert_eq!(run.code, Some(1), "stdout: {}", run.stdout);
    assert!(
        run.stderr.contains("dev-setup/new-linter: "),
        "{}",
        run.stderr
    );
    assert!(
        run.stderr.contains("recipe() in scripts/dev-setup.sh"),
        "{}",
        run.stderr
    );
}

#[rstest]
#[case::unknown_flag(&["--nope"], 2, "dev-setup/usage")]
#[case::root_without_a_directory(&["--root"], 2, "dev-setup/usage")]
#[serial_test::serial]
fn a_bad_invocation_exits_2(#[case] args: &[&str], #[case] code: i32, #[case] message: &str) {
    let run = Machine::bare().setup(args);
    assert_eq!(run.code, Some(code), "stdout: {}", run.stdout);
    assert!(run.stderr.contains(message), "{}", run.stderr);
}

#[test]
#[serial_test::serial]
fn help_exits_0_and_names_both_flags() {
    let run = Machine::bare().setup(&["--help"]);
    assert_eq!(run.code, Some(0), "stderr: {}", run.stderr);
    assert!(
        run.stdout.contains("--root DIR") && run.stdout.contains("--dry-run"),
        "{}",
        run.stdout
    );
}

#[test]
#[serial_test::serial]
fn the_reader_prints_a_listed_version_and_stops_on_any_other_name() {
    let reader = common::workspace_root().join("scripts/tool-version.sh");
    let run = |args: &[&str]| {
        std::process::Command::new("/bin/bash")
            .arg(&reader)
            .args(args)
            .output()
            .expect("spawn the reader")
    };

    let listed_tool = run(&["cargo-nextest"]);
    assert_eq!(listed_tool.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&listed_tool.stdout).trim(),
        listed()["cargo-nextest"]
    );

    let other = run(&["not-a-listed-tool"]);
    assert_eq!(other.status.code(), Some(1));
    assert!(
        other.stdout.is_empty(),
        "nothing an installer could read as a version"
    );
    let message = String::from_utf8_lossy(&other.stderr);
    assert!(
        message.contains("tool-versions/unlisted-tool") && message.contains(LIST),
        "{message}"
    );

    assert_eq!(run(&[]).status.code(), Some(2));
}

/// Runs `scripts/hooks/pre-push` by hand with a stub `cargo` first on PATH, and
/// returns its output with every cargo call the stub saw, one per line. `answers`
/// is the body of a shell `case "$*" in ... esac`; any call it does not name exits 0.
fn run_pre_push_with_cargo(answers: &str) -> (std::process::Output, String) {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().expect("tempdir");
    let calls = dir.path().join("calls");
    let stub = dir.path().join("cargo");
    std::fs::write(
        &stub,
        format!(
            "#!/bin/sh\necho \"$*\" >> '{}'\ncase \"$*\" in\n{answers}\nesac\nexit 0\n",
            calls.display()
        ),
    )
    .expect("write the stub");
    std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    let inherited = std::env::var_os("PATH").expect("a test has a PATH");
    let path = std::env::join_paths(
        std::iter::once(dir.path().to_path_buf()).chain(std::env::split_paths(&inherited)),
    )
    .expect("a joinable PATH");

    let output = std::process::Command::new("bash")
        .arg("scripts/hooks/pre-push")
        .current_dir(common::workspace_root())
        .env("PATH", path)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("spawn the hook");
    let seen = std::fs::read_to_string(&calls).unwrap_or_default();
    (output, seen)
}

/// The exit `cargo nextest show-config version` ends with when cargo finds no
/// nextest, or nextest finds itself older than `.config/nextest.toml` requires.
#[rstest]
#[case::missing(101)]
#[case::too_old(92)]
#[serial_test::serial]
fn pre_push_stops_with_the_setup_command_when_nextest_cannot_run(#[case] exit_code: i32) {
    let (output, calls) = run_pre_push_with_cargo(&format!(
        "\"nextest show-config version\") echo 'what cargo said' >&2; exit {exit_code} ;;"
    ));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert!(
        stderr.contains("what cargo said")
            && stderr.contains("pre-push/missing-tool: cargo-nextest")
            && stderr.contains("scripts/dev-setup.sh")
            && stderr.contains("(AGENTS.md § pre-push/missing-tool)"),
        "{stderr}"
    );
    assert_eq!(
        calls, "nextest show-config version\n",
        "the hook must stop before any other cargo step"
    );
}

/// nextest exits 96 on a config it cannot parse. Installing nextest again leaves
/// the file as it is, so the stop names the file instead.
#[test]
#[serial_test::serial]
fn pre_push_stops_naming_the_file_when_nextest_cannot_parse_its_config() {
    let (output, calls) = run_pre_push_with_cargo(
        "\"nextest show-config version\") echo 'TOML parse error at line 2' >&2; exit 96 ;;",
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert!(
        stderr.contains("TOML parse error at line 2")
            && stderr.contains("pre-push/nextest-config: .config/nextest.toml")
            && stderr.contains("(AGENTS.md § pre-push/nextest-config)"),
        "{stderr}"
    );
    assert!(
        !stderr.contains("pre-push/missing-tool") && !stderr.contains("dev-setup.sh"),
        "a config nextest cannot parse is not a missing tool: {stderr}"
    );
    assert_eq!(
        calls, "nextest show-config version\n",
        "the hook must stop before any other cargo step"
    );
}

/// nextest exits 10 when it meets the config's required version but not its
/// recommended one, and prints its own warning. The push goes on to the next step,
/// which the stub fails so the hook ends there.
#[test]
#[serial_test::serial]
fn pre_push_goes_on_when_nextest_only_recommends_a_newer_release() {
    let (output, calls) = run_pre_push_with_cargo(
        "\"nextest show-config version\") echo 'warning: a newer nextest is recommended' >&2; \
         exit 10 ;;\n\"fmt -- --check\") exit 1 ;;",
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("warning: a newer nextest is recommended"),
        "the hook must show nextest's warning: {stderr}"
    );
    assert!(!stderr.contains("pre-push/"), "{stderr}");
    assert_eq!(
        calls, "nextest show-config version\nfmt -- --check\n",
        "a recommendation must not stop the push"
    );
}

/// Every code the hook can stop with has its own heading in AGENTS.md, where the
/// stop's message points.
#[test]
fn every_pre_push_code_has_a_heading_in_agents_md() {
    let hook = read("scripts/hooks/pre-push");
    let agents = read("AGENTS.md");
    let codes: std::collections::BTreeSet<&str> = Regex::new(r"pre-push/[a-z][a-z-]*[a-z]")
        .expect("a regex")
        .find_iter(&hook)
        .map(|found| found.as_str())
        .collect();
    assert!(
        !codes.is_empty(),
        "scripts/hooks/pre-push names no pre-push/ code"
    );
    for code in codes {
        assert!(
            agents.contains(&format!("\n### `{code}`\n")),
            "scripts/hooks/pre-push stops with {code}, and AGENTS.md has no `### `{code}`` heading \
             for its message to point at"
        );
    }
}
