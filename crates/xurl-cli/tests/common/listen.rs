//! Driving `xr webhooks listen` from a test: a free port, the run beside a
//! client, the CRC answer, a delivery, and stdout read line by line.
//!
//! `oauth1_store` gives the app the client secret `SECRET-VALUE` and the
//! consumer secret `CONSUMER-SECRET`. Each expected signature below was
//! computed outside this crate (Python's `hmac.new(key, message, sha256)`,
//! base64-encoded) for those keys.

use std::path::Path;

use super::run_in_process as run;

pub const CRC_TOKEN: &str = "challenge_string";
pub const CRC_UNDER_CLIENT_SECRET: &str = "sha256=xT/YyTRZde5T1HbCK7K2bpcELYOIMkZVbgYjsESw+AM=";
pub const CRC_UNDER_CONSUMER_SECRET: &str = "sha256=9CnCWcBcOApA1LHdqkjWbF2DBNdBrnAJPmL5qTUUANQ=";
pub const EVENT: &str = r#"{"for_user_id":"2244994945","follow_events":[]}"#;
pub const EVENT_UNDER_CLIENT_SECRET: &str = "sha256=jlSMkj72sqpIWJdl8rdKiEf1zPMRJkvY1VWxVhuPD4c=";
pub const EVENT_UNDER_CONSUMER_SECRET: &str = "sha256=IdtPMVhr8Hql9OtN7YTX/+gknTMNOui0D4ixytI8bD0=";

pub fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("a loopback port")
        .local_addr()
        .expect("its address")
        .port()
}

/// Runs `xr webhooks listen` on `port` with `extra` flags while `drive` talks
/// to it, and returns both results. The run ends once `--max-events` events
/// have printed, so `drive` has to deliver that many. Both futures share
/// this task: the run holds a `&mut dyn Write` and cannot be spawned.
pub async fn listen_while<T>(
    store: &Path,
    port: u16,
    extra: &[&str],
    drive: impl std::future::Future<Output = T>,
) -> ((i32, String, String), T) {
    listen_as(store, "json", port, extra, drive).await
}

/// [`listen_while`] under `--output <format>`.
pub async fn listen_as<T>(
    store: &Path,
    format: &str,
    port: u16,
    extra: &[&str],
    drive: impl std::future::Future<Output = T>,
) -> ((i32, String, String), T) {
    let port = port.to_string();
    let mut args = vec!["--output", format, "webhooks", "listen", "--port", &port];
    args.extend_from_slice(extra);
    tokio::join!(run(store, "http://127.0.0.1:9", &args), drive)
}

/// The CRC response token the listener on `port` gives, once it is up.
pub async fn crc_token_from(port: u16) -> String {
    let url = format!("http://127.0.0.1:{port}/webhook?crc_token={CRC_TOKEN}");
    for _ in 0..100 {
        if let Ok(response) = reqwest::get(&url).await {
            let body: serde_json::Value = response.json().await.expect("a JSON body");
            return body["response_token"]
                .as_str()
                .expect("a response token")
                .to_string();
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("the listener never came up on port {port}");
}

/// Posts [`EVENT`] to the listener on `port` under `header`, when one is
/// given, and returns the status it answers with.
pub async fn deliver(port: u16, header: Option<(&str, &str)>) -> u16 {
    let mut request = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{port}/webhook"))
        .body(EVENT);
    if let Some((name, value)) = header {
        request = request.header(name, value);
    }
    request
        .send()
        .await
        .expect("the listener answers")
        .status()
        .as_u16()
}

pub const OAUTH2_HEADER: &str = "X-Twitter-Webhooks-Signature-OAuth2";
pub const LEGACY_HEADER: &str = "X-Twitter-Webhooks-Signature";

/// A stdout that hands each line to the test as `listen` writes it, so a
/// test can act on the `listening` record while the run goes on.
pub struct LineTap {
    partial: Vec<u8>,
    lines: tokio::sync::mpsc::UnboundedSender<String>,
}

impl std::io::Write for LineTap {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.partial.extend_from_slice(buf);
        while let Some(end) = self.partial.iter().position(|byte| *byte == b'\n') {
            let line: Vec<u8> = self.partial.drain(..=end).collect();
            let _ = self
                .lines
                .send(String::from_utf8_lossy(&line[..end]).into_owned());
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Runs `xr --output json webhooks listen` on `port` with `extra` flags while
/// `drive` reads its stdout line by line, and returns the exit code, stderr,
/// and what `drive` returned.
pub async fn listen_tapped<T, F>(
    store: &Path,
    port: u16,
    extra: &[&str],
    drive: impl FnOnce(tokio::sync::mpsc::UnboundedReceiver<String>) -> F,
) -> (i32, String, T)
where
    F: std::future::Future<Output = T>,
{
    let port = port.to_string();
    let mut argv = vec![
        "xr", "--output", "json", "webhooks", "listen", "--port", &port,
    ];
    argv.extend_from_slice(extra);
    let (sender, lines) = tokio::sync::mpsc::unbounded_channel();
    let mut stdout = LineTap {
        partial: Vec::new(),
        lines: sender,
    };
    let mut stderr: Vec<u8> = Vec::new();
    let overrides = xdk::config::EnvOverrides {
        api_base_url: Some("http://127.0.0.1:9".to_string()),
        ..xdk::config::EnvOverrides::default()
    };
    let run =
        xurl::cli::runner::run_with_overrides(argv, &mut stdout, &mut stderr, store, &overrides);
    let (code, driven) = tokio::join!(run, drive(lines));
    (code, String::from_utf8_lossy(&stderr).into_owned(), driven)
}

/// The next line `listen` printed, as JSON.
pub async fn next_record(
    lines: &mut tokio::sync::mpsc::UnboundedReceiver<String>,
) -> serde_json::Value {
    let line = tokio::time::timeout(std::time::Duration::from_secs(10), lines.recv())
        .await
        .expect("listen prints within ten seconds")
        .expect("listen printed a line");
    serde_json::from_str(&line).unwrap_or_else(|e| panic!("not JSON ({e}): {line}"))
}

/// Posts `body` to the listener on `port` with `headers`, and returns the
/// status it answers with.
pub async fn post(port: u16, body: &str, headers: &[(&str, &str)]) -> u16 {
    let mut request = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{port}/webhook"))
        .body(body.to_string());
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    request
        .send()
        .await
        .expect("the listener answers")
        .status()
        .as_u16()
}

/// The JSON records `listen` wrote on stderr, one per line.
pub fn stderr_records(stderr: &str) -> Vec<serde_json::Value> {
    stderr
        .lines()
        .filter(|line| line.starts_with('{'))
        .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("not JSON ({e}): {line}")))
        .collect()
}

/// `xr --output json webhooks listen` as a child process, for what only a
/// real process shows: its exit, its own stderr, and a signal.
pub struct ListenProcess {
    child: std::process::Child,
    lines: std::sync::mpsc::Receiver<String>,
    stderr: std::thread::JoinHandle<String>,
}

impl ListenProcess {
    /// Starts `listen` on `port` with `extra` flags against `store`, and
    /// waits for its `listening` line.
    pub fn start(store: &Path, port: u16, extra: &[&str]) -> Self {
        use std::io::{BufRead, Read};
        use std::process::Stdio;

        let port = port.to_string();
        let mut child = super::xr_std_with_store_at(super::xr_bin(), store)
            .args(["--output", "json", "webhooks", "listen", "--port", &port])
            .args(extra)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn xr webhooks listen");
        let stdout = child.stdout.take().expect("piped stdout");
        let (sender, lines) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in std::io::BufReader::new(stdout)
                .lines()
                .map_while(Result::ok)
            {
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        let mut stderr_pipe = child.stderr.take().expect("piped stderr");
        let stderr = std::thread::spawn(move || {
            let mut text = String::new();
            let _ = stderr_pipe.read_to_string(&mut text);
            text
        });
        let process = Self {
            child,
            lines,
            stderr,
        };
        match process
            .lines
            .recv_timeout(std::time::Duration::from_secs(10))
        {
            Ok(listening) if listening.contains("\"listening\"") => process,
            Ok(other) => process.fail(&format!("its first stdout line was {other}")),
            Err(error) => process.fail(&format!("it printed no listening line ({error})")),
        }
    }

    /// Ends the process and fails the test with `why`, its exit, and its
    /// stderr, the only place a refusal before binding shows.
    fn fail(mut self, why: &str) -> ! {
        let _ = self.child.kill();
        let status = self.child.wait();
        let stderr = self.stderr.join().unwrap_or_default();
        panic!("listen did not start: {why}; exit {status:?}; stderr: {stderr}");
    }

    /// The next stdout line, within ten seconds.
    pub fn next_line(&self) -> String {
        self.lines
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("listen prints a line within ten seconds")
    }

    /// Sends SIGINT, as Ctrl+C does.
    #[cfg(unix)]
    pub fn interrupt(&self) {
        let status = std::process::Command::new("kill")
            .args(["-INT", &self.child.id().to_string()])
            .status()
            .expect("kill runs");
        assert!(status.success(), "kill -INT failed");
    }

    /// Waits for the exit, at most `within`, and returns the status and all
    /// of stderr. Past `within` the process is killed and the test fails.
    pub fn wait(self, within: std::time::Duration) -> (std::process::ExitStatus, String) {
        let (status, _rest, stderr) = self.wait_for_all(within);
        (status, stderr)
    }

    /// [`Self::wait`], also returning the stdout lines not yet read.
    pub fn wait_for_all(
        mut self,
        within: std::time::Duration,
    ) -> (std::process::ExitStatus, Vec<String>, String) {
        let deadline = std::time::Instant::now() + within;
        let status = loop {
            if let Some(status) = self.child.try_wait().expect("try_wait") {
                break status;
            }
            if std::time::Instant::now() >= deadline {
                let _ = self.child.kill();
                let _ = self.child.wait();
                panic!("listen did not exit within {within:?}");
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        let stderr = self.stderr.join().expect("the stderr reader");
        let rest = self.lines.iter().collect();
        (status, rest, stderr)
    }
}
