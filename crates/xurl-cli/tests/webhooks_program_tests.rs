//! A program after `--` on `xr webhooks listen` runs once per verified event
//! with the event's body on its stdin, beside or instead of a URL target, and
//! is stopped with everything it started when an attempt runs out of time.
//!
//! The programs are this test binary itself, run again with a marker word:
//! each `probe_*` test below does nothing in an ordinary run and acts as one
//! program when its marker is on the command line. Rust programs rather than
//! `sh` scripts, so every test here runs on Linux, macOS, and Windows alike.
//! Nothing here reaches X.

mod common;

use std::io::{Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use tempfile::TempDir;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::listen::{
    EVENT, EVENT_UNDER_CLIENT_SECRET, ListenProcess, OAUTH2_HEADER, crc_token_from, deliver,
    held_port, listen_while, post, stderr_records,
};
use common::{oauth1_store, run_in_process as run};

/// The word that turns a run of this binary into a probe program; what
/// follows it are the probe's own arguments.
const MARKER: &str = "xr-program-probe";

/// The argv that runs the probe `name` with `params`.
fn probe(name: &str, params: &[&str]) -> Vec<String> {
    let exe = std::env::current_exe().expect("the test binary");
    let mut argv = vec![
        exe.to_string_lossy().into_owned(),
        "--exact".to_string(),
        format!("probe_{name}"),
        "--nocapture".to_string(),
        "--test-threads".to_string(),
        "1".to_string(),
        MARKER.to_string(),
    ];
    argv.extend(params.iter().map(|param| (*param).to_string()));
    argv
}

/// The probe's arguments, when this run is a probe.
fn probe_args() -> Option<Vec<String>> {
    let args: Vec<String> = std::env::args().collect();
    let at = args.iter().position(|arg| arg == MARKER)?;
    Some(args[at + 1..].to_vec())
}

fn append_line(path: &str, line: &str) {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("open the probe's file");
    writeln!(file, "{line}").expect("write the probe's file");
}

fn read_stdin() -> Vec<u8> {
    let mut body = Vec::new();
    std::io::stdin().read_to_end(&mut body).expect("read stdin");
    body
}

fn hang() -> ! {
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

/// Appends what it read, the `XR_EVENT_SHA256` it saw, and its pid to the
/// file it names, and writes a line to its own stdout.
#[test]
fn probe_echo() {
    let Some(args) = probe_args() else {
        return;
    };
    let body = read_stdin();
    let record = serde_json::json!({
        "body": String::from_utf8_lossy(&body),
        "sha256": std::env::var("XR_EVENT_SHA256").ok(),
        "pid": std::process::id(),
    });
    append_line(&args[0], &record.to_string());
    println!("probe stdout: one event read");
}

/// Exits with the `exit` its event names.
#[test]
fn probe_exit() {
    let Some(_args) = probe_args() else {
        return;
    };
    let event: serde_json::Value = serde_json::from_slice(&read_stdin()).expect("a JSON event");
    let code = event["exit"].as_i64().expect("an exit code");
    std::process::exit(i32::try_from(code).expect("a small exit code"));
}

/// Appends its pid to the file it names, when it names one, and never ends.
#[test]
fn probe_hang() {
    let Some(args) = probe_args() else {
        return;
    };
    if let Some(path) = args.first() {
        append_line(path, &std::process::id().to_string());
    }
    hang();
}

/// Starts a hanging child of its own, appends the child's pid to the file
/// it names, and never ends.
#[test]
#[allow(clippy::zombie_processes)] // the probe hangs until it is killed; the test watches its child die with it
fn probe_spawn_and_hang() {
    let Some(args) = probe_args() else {
        return;
    };
    let argv = probe("hang", &[]);
    let child = std::process::Command::new(&argv[0])
        .args(&argv[1..])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn the grandchild");
    append_line(&args[0], &child.id().to_string());
    hang();
}

/// Exits 0 without reading its stdin, ending the line the test harness
/// started.
#[test]
fn probe_exit_unread() {
    if probe_args().is_some() {
        println!();
        std::process::exit(0);
    }
}

/// Writes output with no closing newline to its stdout and its stderr, after
/// ending the line the test harness started, and exits 0.
#[test]
fn probe_unfinished_output() {
    if probe_args().is_some() {
        let _ = read_stdin();
        println!();
        print!("out-unfinished");
        eprint!("err-unfinished");
        let _ = std::io::stdout().flush();
        std::process::exit(0);
    }
}

/// Appends its pid to the file it names and never reads its stdin.
#[test]
fn probe_never_read() {
    let Some(args) = probe_args() else {
        return;
    };
    append_line(&args[0], &std::process::id().to_string());
    hang();
}

// ── helpers ───────────────────────────────────────────────────────────

/// Whether process `pid` is still running. A zombie, dead and awaiting its
/// parent's reap, counts as gone.
fn alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        let stat = ps_stat(pid);
        !stat.is_empty() && !stat.starts_with('Z')
    }
    #[cfg(windows)]
    {
        listed(pid)
    }
}

/// Whether any process has `pid`, a zombie not yet reaped included.
fn listed(pid: u32) -> bool {
    #[cfg(unix)]
    {
        !ps_stat(pid).is_empty()
    }
    #[cfg(windows)]
    {
        let output = std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
            .output()
            .expect("tasklist runs");
        String::from_utf8_lossy(&output.stdout).contains(&format!("\"{pid}\""))
    }
}

/// The state `ps` gives process `pid`, empty when no process has it.
#[cfg(unix)]
fn ps_stat(pid: u32) -> String {
    let output = std::process::Command::new("ps")
        .args(["-o", "stat=", "-p", &pid.to_string()])
        .output()
        .expect("ps runs");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// Every pid listed in `path`, one per line.
fn pids_in(path: &Path) -> Vec<u32> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|line| line.trim().parse().expect("a pid"))
        .collect()
}

/// Asserts each pid in `path` stops running within five seconds.
fn assert_all_gone(path: &Path) {
    let pids = pids_in(path);
    assert!(
        !pids.is_empty(),
        "the probe recorded no pid in {}",
        path.display()
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while pids.iter().any(|pid| alive(*pid)) {
        assert!(
            Instant::now() < deadline,
            "still running after the program was stopped: {:?}",
            pids.iter().filter(|pid| alive(**pid)).collect::<Vec<_>>()
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// The JSON lines a probe appended to `path`.
fn echoed(path: &Path) -> Vec<serde_json::Value> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).expect("a probe record"))
        .collect()
}

fn listen_args<'a>(flags: &[&'a str], program: &'a [String]) -> Vec<&'a str> {
    let mut args = flags.to_vec();
    args.push("--");
    args.extend(program.iter().map(String::as_str));
    args
}

async fn answering(status: u16, delay: Duration) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(status).set_delay(delay))
        .mount(&server)
        .await;
    server
}

// ── delivery ──────────────────────────────────────────────────────────

#[tokio::test]
async fn a_program_and_a_url_each_receive_the_event_byte_for_byte() {
    let server = answering(200, Duration::ZERO).await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let out = tmp.path().join("events.jsonl");
    let program = probe("echo", &[out.to_str().expect("utf-8 path")]);
    let url = format!("{}/hook", server.uri());
    let args = listen_args(&["--max-events", "1", "--forward", &url], &program);

    let ((code, _stdout, stderr), ()) = listen_while(&store, &args, async |port| {
        crc_token_from(port).await;
        let status = deliver(port, Some((OAUTH2_HEADER, EVENT_UNDER_CLIENT_SECRET))).await;
        assert_eq!(status, 200);
    })
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let seen = server.received_requests().await.expect("recorded");
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].body, EVENT.as_bytes());
    let echoed = echoed(&out);
    assert_eq!(echoed.len(), 1);
    assert_eq!(echoed[0]["body"], EVENT, "the program's stdin is X's bytes");
    let key = seen[0].headers["idempotency-key"]
        .to_str()
        .expect("the key")
        .trim_matches('"');
    assert_eq!(
        echoed[0]["sha256"], key,
        "XR_EVENT_SHA256 is the key the URL target gets"
    );
    let records = stderr_records(&stderr);
    let program_record = records
        .iter()
        .find(|record| record["target"] == "program")
        .unwrap_or_else(|| panic!("no program record: {stderr}"));
    assert_eq!(program_record["status"], "forwarded");
    assert_eq!(program_record["exit_status"], 0);
    assert_eq!(program_record["attempts"], 1);
    assert!(
        records.iter().any(|record| record["target"] == "url"),
        "{stderr}"
    );
}

#[tokio::test]
async fn events_reach_the_program_in_order_one_process_each() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let out = tmp.path().join("events.jsonl");
    let program = probe("echo", &[out.to_str().expect("utf-8 path")]);
    let args = listen_args(&["--allow-unsigned", "--max-events", "3"], &program);

    let ((code, _stdout, stderr), ()) = listen_while(&store, &args, async |port| {
        crc_token_from(port).await;
        for n in 1..=3 {
            assert_eq!(post(port, &format!(r#"{{"n":{n}}}"#), &[]).await, 200);
        }
    })
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let echoed = echoed(&out);
    let bodies: Vec<&str> = echoed.iter().filter_map(|r| r["body"].as_str()).collect();
    assert_eq!(bodies, [r#"{"n":1}"#, r#"{"n":2}"#, r#"{"n":3}"#]);
    let mut pids: Vec<u64> = echoed.iter().filter_map(|r| r["pid"].as_u64()).collect();
    pids.dedup();
    assert_eq!(pids.len(), 3, "one process per event: {echoed:?}");
}

#[tokio::test]
async fn the_programs_output_goes_to_stderr_and_stdout_keeps_one_line_per_event() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let out = tmp.path().join("events.jsonl");
    let program = probe("echo", &[out.to_str().expect("utf-8 path")]);
    let args = listen_args(&["--allow-unsigned", "--max-events", "1"], &program);
    let listen = ListenProcess::start(&store, &args);
    let port = listen.port;

    assert_eq!(post(port, r#"{"n":1}"#, &[]).await, 200);
    let (status, rest, stderr) = listen.wait_for_all(Duration::from_secs(20));

    assert_eq!(status.code(), Some(0), "stderr: {stderr}");
    assert_eq!(rest, [r#"{"n":1}"#], "stdout holds the event alone");
    assert!(
        stderr.contains("probe stdout: one event read"),
        "the program's stdout reaches stderr: {stderr}"
    );
}

/// A program's output and `listen`'s records share stderr, one line each:
/// output a program leaves without a newline is ended by `listen`, so the
/// record after it still reads as a line of its own.
#[tokio::test]
async fn a_programs_unfinished_output_never_shares_a_line_with_a_record() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let program = probe("unfinished_output", &[]);
    let args = listen_args(&["--allow-unsigned", "--max-events", "1"], &program);
    let listen = ListenProcess::start(&store, &args);
    let port = listen.port;

    assert_eq!(post(port, r#"{"n":1}"#, &[]).await, 200);
    let (status, _rest, stderr) = listen.wait_for_all(Duration::from_secs(20));

    assert_eq!(status.code(), Some(0), "stderr: {stderr}");
    let lines: Vec<&str> = stderr.lines().collect();
    assert!(
        lines.contains(&"out-unfinished") && lines.contains(&"err-unfinished"),
        "each unfinished output ends its own line: {stderr}"
    );
    let record = stderr_records(&stderr)
        .into_iter()
        .find(|record| record["target"] == "program")
        .unwrap_or_else(|| panic!("no program record on a line of its own: {stderr}"));
    assert_eq!(record["status"], "forwarded", "{stderr}");
}

#[tokio::test]
async fn a_program_that_fails_is_reported_after_four_attempts_and_the_next_event_runs() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let program = probe("exit", &[]);
    let args = listen_args(&["--allow-unsigned", "--max-events", "2"], &program);

    let ((code, stdout, stderr), ()) = listen_while(&store, &args, async |port| {
        crc_token_from(port).await;
        assert_eq!(post(port, r#"{"exit":3}"#, &[]).await, 200);
        assert_eq!(post(port, r#"{"exit":0}"#, &[]).await, 200);
    })
    .await;

    assert_eq!(code, 0, "a failed forward never changes the exit: {stderr}");
    assert_eq!(stdout.lines().count(), 3, "{stdout}");
    let records = stderr_records(&stderr);
    assert_eq!(records.len(), 2, "{stderr}");
    assert_eq!(records[0]["status"], "forward-failed");
    assert_eq!(records[0]["target"], "program");
    assert_eq!(records[0]["attempts"], 4);
    assert!(
        records[0]["cause"]
            .as_str()
            .is_some_and(|c| c.contains("exited with 3")),
        "{}",
        records[0]
    );
    assert_eq!(records[1]["status"], "forwarded");
    assert_eq!(records[1]["exit_status"], 0);
}

// ── stopping ──────────────────────────────────────────────────────────

#[tokio::test]
async fn a_program_past_the_timeout_is_stopped_and_leaves_nothing_running() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let pids = tmp.path().join("pids");
    let program = probe("hang", &[pids.to_str().expect("utf-8 path")]);
    let flags = [
        "--allow-unsigned",
        "--max-events",
        "1",
        "--forward-timeout",
        "1",
    ];
    let args = listen_args(&flags, &program);

    let started = Instant::now();
    let ((code, _stdout, stderr), ()) = listen_while(&store, &args, async |port| {
        crc_token_from(port).await;
        assert_eq!(post(port, r#"{"n":1}"#, &[]).await, 200);
    })
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        started.elapsed() < Duration::from_secs(15),
        "the drain deadline holds: {:?}",
        started.elapsed()
    );
    assert_all_gone(&pids);
}

#[tokio::test]
async fn a_program_is_stopped_with_the_children_it_started() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let children = tmp.path().join("children");
    let program = probe("spawn_and_hang", &[children.to_str().expect("utf-8 path")]);
    let flags = [
        "--allow-unsigned",
        "--max-events",
        "1",
        "--forward-timeout",
        "1",
    ];
    let args = listen_args(&flags, &program);

    let ((code, _stdout, stderr), ()) = listen_while(&store, &args, async |port| {
        crc_token_from(port).await;
        assert_eq!(post(port, r#"{"n":1}"#, &[]).await, 200);
    })
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        !stderr.contains("could not start"),
        "the program ran in a tree of its own: {stderr}"
    );
    assert_all_gone(&children);
}

#[tokio::test]
async fn a_program_that_never_reads_a_large_body_is_stopped_at_the_timeout() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let pids = tmp.path().join("pids");
    let program = probe("never_read", &[pids.to_str().expect("utf-8 path")]);
    let flags = [
        "--allow-unsigned",
        "--max-events",
        "2",
        "--forward-timeout",
        "1",
    ];
    let args = listen_args(&flags, &program);
    let large = format!(r#"{{"pad":"{}"}}"#, "x".repeat(200 * 1024));

    let started = Instant::now();
    let ((code, stdout, stderr), ()) = listen_while(&store, &args, async |port| {
        crc_token_from(port).await;
        assert_eq!(post(port, &large, &[]).await, 200);
        assert_eq!(post(port, r#"{"n":2}"#, &[]).await, 200);
    })
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        started.elapsed() < Duration::from_secs(15),
        "a blocked write is cut with its attempt: {:?}",
        started.elapsed()
    );
    assert_eq!(stdout.lines().count(), 3, "the next event is still handled");
    assert_all_gone(&pids);
}

/// `xr` keeps SIGPIPE's default action, so `xr ... | head` ends quietly. A
/// program that exits without reading its stdin closes the pipe `listen`
/// is writing; that must end the write, not `listen`.
#[cfg(unix)]
#[tokio::test]
async fn a_program_that_exits_without_reading_its_stdin_leaves_listen_running() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let program = probe("exit_unread", &[]);
    let args = listen_args(&["--allow-unsigned", "--max-events", "1"], &program);
    let listen = ListenProcess::start(&store, &args);
    let port = listen.port;
    let large = format!(r#"{{"pad":"{}"}}"#, "x".repeat(200 * 1024));

    assert_eq!(post(port, &large, &[]).await, 200);
    let (status, _rest, stderr) = listen.wait_for_all(Duration::from_secs(20));

    assert_eq!(
        status.code(),
        Some(0),
        "listen outlived the closed pipe: {status:?}; stderr: {stderr}"
    );
    let records = stderr_records(&stderr);
    let record = records
        .iter()
        .find(|record| record["target"] == "program")
        .unwrap_or_else(|| panic!("no program record: {stderr}"));
    assert_eq!(record["status"], "forwarded", "{stderr}");
    assert_eq!(record["exit_status"], 0);
}

/// A test that panics before it stops its `ListenProcess` drops it instead,
/// and a listener with no `--max-events` runs until something stops it.
#[test]
fn a_listen_process_dropped_unstopped_leaves_nothing_running() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let listen = ListenProcess::start(&store, &[]);
    let pid = listen.id();

    drop(listen);

    assert!(
        !listed(pid),
        "xr webhooks listen (pid {pid}) outlived its ListenProcess"
    );
}

// ── isolation and refusals ────────────────────────────────────────────

#[tokio::test]
async fn an_event_reaches_the_program_as_bytes_and_runs_nothing() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let out = tmp.path().join("events.jsonl");
    let program = probe("echo", &[out.to_str().expect("utf-8 path")]);
    let args = listen_args(&["--allow-unsigned", "--max-events", "1"], &program);
    let hostile = r#"{"a":"$(touch pwned)","b":"; rm -rf pwned-too"}"#;

    let ((code, _stdout, stderr), ()) = listen_while(&store, &args, async |port| {
        crc_token_from(port).await;
        assert_eq!(post(port, hostile, &[]).await, 200);
    })
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(echoed(&out)[0]["body"], hostile);
    assert!(!Path::new("pwned").exists(), "nothing ran the event");
    assert!(!tmp.path().join("pwned").exists());
}

#[tokio::test]
async fn a_stalled_url_target_never_delays_the_program() {
    let server = answering(200, Duration::from_secs(30)).await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let out = tmp.path().join("events.jsonl");
    let program = probe("echo", &[out.to_str().expect("utf-8 path")]);
    let url = format!("{}/hook", server.uri());
    let flags = [
        "--allow-unsigned",
        "--max-events",
        "2",
        "--forward",
        &url,
        "--forward-timeout",
        "1",
    ];
    let args = listen_args(&flags, &program);

    let ((code, _stdout, stderr), both_within) = listen_while(&store, &args, async |port| {
        crc_token_from(port).await;
        assert_eq!(post(port, r#"{"n":1}"#, &[]).await, 200);
        assert_eq!(post(port, r#"{"n":2}"#, &[]).await, 200);
        let deadline = Instant::now() + Duration::from_secs(5);
        while echoed(&out).len() < 2 {
            if Instant::now() >= deadline {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        true
    })
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        both_within,
        "the program received both events while the URL target stalled"
    );
}

#[tokio::test]
async fn a_program_that_does_not_resolve_is_refused_before_anything_binds() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let (_held, port) = held_port();

    let (code, stdout, stderr) = run(
        &store,
        "http://127.0.0.1:9",
        &[
            "--output",
            "json",
            "webhooks",
            "listen",
            "--port",
            &port.to_string(),
            "--",
            "no-such-program-for-xr-tests",
        ],
    )
    .await;

    assert_eq!(code, 1, "stderr: {stderr}");
    assert_eq!(stdout, "");
    let envelope: serde_json::Value = serde_json::from_str(stderr.trim()).expect("an envelope");
    assert_eq!(envelope["reason"], "validation", "{envelope}");
    assert_eq!(envelope["next_step"]["action"], "show-help");
}

#[tokio::test]
async fn a_dry_run_names_the_program_and_runs_nothing() {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let (_held, port) = held_port();

    let (code, stdout, stderr) = run(
        &store,
        "http://127.0.0.1:9",
        &[
            "--dry-run",
            "--output",
            "json",
            "webhooks",
            "listen",
            "--port",
            &port.to_string(),
            "--",
            "./handle.sh",
            "--verbose",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON");
    assert_eq!(body["status"], "dry_run");
    assert_eq!(
        body["program"],
        serde_json::json!(["./handle.sh", "--verbose"])
    );
}
