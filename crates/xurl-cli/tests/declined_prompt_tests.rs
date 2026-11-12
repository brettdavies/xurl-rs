//! A destructive command whose prompt is answered no changes nothing, says
//! so, and exits 1.
//!
//! The prompt appears only when stdin and stderr are terminals, so each run
//! gives `xr` a pseudo-terminal and types the answer into it.

#![cfg(unix)]

mod common;

use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::{FromRawFd, OwnedFd};
use std::path::{Path, PathBuf};
use std::process::Stdio;

use tempfile::TempDir;
use xdk::store::TokenStore;

/// What `xr` ended with and everything it wrote to its terminal, prompt and
/// the answer's echo included.
struct Ran {
    code: Option<i32>,
    terminal: String,
}

impl Ran {
    /// The document `xr` wrote after the prompt: from the first `{` on, with
    /// the terminal's line endings undone.
    fn document(&self) -> serde_json::Value {
        let start = self.terminal.find('{').expect("a document on the terminal");
        let text = self.terminal[start..].replace("\r\n", "\n");
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}"))
    }
}

/// Runs `xr` with `args` on a terminal and types `answer` at the prompt.
fn on_a_terminal(store: &Path, args: &[&str], answer: &str) -> Ran {
    let (mut master, mut slave) = (0, 0);
    // SAFETY: both descriptors are out-parameters the call fills in; the name,
    // termios, and window size are optional and passed as null.
    let opened = unsafe {
        libc::openpty(
            &mut master,
            &mut slave,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert_eq!(opened, 0, "openpty failed");
    // SAFETY: `openpty` returned these descriptors and nothing else owns them.
    let (mut master, slave) = unsafe { (File::from_raw_fd(master), OwnedFd::from_raw_fd(slave)) };

    let mut command = common::xr_std_with_store_at(common::xr_bin(), store);
    command
        .args(args)
        .stdin(Stdio::from(slave.try_clone().expect("clone the terminal")))
        .stderr(Stdio::from(slave.try_clone().expect("clone the terminal")))
        .stdout(Stdio::null());
    let mut child = command.spawn().expect("xr runs");
    drop(command);
    drop(slave);

    master
        .write_all(format!("{answer}\n").as_bytes())
        .expect("type the answer");
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        match master.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => bytes.extend_from_slice(&chunk[..n]),
            Err(e) if e.raw_os_error() == Some(libc::EIO) => break,
            Err(e) => panic!("reading the terminal: {e}"),
        }
    }
    let status = child.wait().expect("xr ends");
    Ran {
        code: status.code(),
        terminal: String::from_utf8_lossy(&bytes).into_owned(),
    }
}

/// A store whose default app, `work`, holds an `OAuth2` login.
fn store_with_a_login(tmp: &TempDir) -> PathBuf {
    let store = tmp.path().join(".xurl");
    let mut ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    ts.add_app("work", "CLIENT-ID-VALUE", "SECRET-VALUE")
        .expect("add_app");
    ts.save_oauth2_token_for_app("work", "alice", "user-at", "user-rt", 9_999_999_999)
        .expect("save_oauth2");
    ts.set_default_app("work").expect("set_default_app");
    let _ = ts.remove_app("default");
    store
}

fn logins(store: &Path) -> Vec<String> {
    let ts = TokenStore::new_with_path(store.to_str().expect("utf-8 path"));
    let mut names: Vec<String> = ts
        .get_app("work")
        .map(|app| app.oauth2_tokens.keys().cloned().collect())
        .unwrap_or_default();
    names.sort();
    names
}

fn app_is_stored(store: &Path) -> bool {
    TokenStore::new_with_path(store.to_str().expect("utf-8 path"))
        .get_app("work")
        .is_some()
}

const CLEAR: &[&str] = &["auth", "clear", "--all"];
const REMOVE: &[&str] = &["auth", "apps", "remove", "work"];

/// An answer other than yes stops the command before it writes, says so in
/// text, and exits 1.
#[rstest::rstest]
#[case::no("n")]
#[case::nothing("")]
#[case::anything_else("maybe")]
fn declining_a_clear_changes_nothing_and_says_so(#[case] answer: &str) {
    let tmp = TempDir::new().expect("tempdir");
    let store = store_with_a_login(&tmp);

    let ran = on_a_terminal(&store, CLEAR, answer);

    assert_eq!(ran.code, Some(1), "{}", ran.terminal);
    assert!(
        ran.terminal.contains("Cancelled: nothing was changed."),
        "{}",
        ran.terminal
    );
    assert_eq!(logins(&store), ["alice"], "the login was cleared");
}

/// Under a structured format the stop is one `cancelled` document on stderr
/// that names what the prompt asked about.
#[test]
fn declining_a_clear_under_json_is_one_cancelled_document() {
    let tmp = TempDir::new().expect("tempdir");
    let store = store_with_a_login(&tmp);
    let args = ["--output", "json", "auth", "clear", "--all"];

    let ran = on_a_terminal(&store, &args, "n");

    assert_eq!(ran.code, Some(1), "{}", ran.terminal);
    let document = ran.document();
    assert_eq!(document["status"], "cancelled", "{document}");
    assert_eq!(document["message"], "cancelled at the prompt", "{document}");
    assert_eq!(document["exit_code"], 1, "{document}");
    assert_eq!(document["command"], "auth-clear", "{document}");
    assert_eq!(document["app"], "work", "{document}");
    assert_eq!(document["all"], true, "{document}");
    assert_eq!(logins(&store), ["alice"], "the login was cleared");
}

#[test]
fn declining_an_app_removal_keeps_the_app() {
    let tmp = TempDir::new().expect("tempdir");
    let store = store_with_a_login(&tmp);

    let text = on_a_terminal(&store, REMOVE, "n");
    assert_eq!(text.code, Some(1), "{}", text.terminal);
    assert!(
        text.terminal.contains("Cancelled: nothing was changed."),
        "{}",
        text.terminal
    );

    let mut json = vec!["--output", "json"];
    json.extend_from_slice(REMOVE);
    let structured = on_a_terminal(&store, &json, "n");
    assert_eq!(structured.code, Some(1), "{}", structured.terminal);
    assert_eq!(structured.document()["status"], "cancelled");
    assert!(app_is_stored(&store), "the app was removed");
}

/// The control: a yes at the same prompt runs the command, so the terminal
/// reaches the prompt and the checks above are about the answer.
#[test]
fn accepting_the_prompt_clears_and_exits_0() {
    let tmp = TempDir::new().expect("tempdir");
    let store = store_with_a_login(&tmp);

    let ran = on_a_terminal(&store, CLEAR, "y");

    assert_eq!(ran.code, Some(0), "{}", ran.terminal);
    assert!(!ran.terminal.contains("Cancelled"), "{}", ran.terminal);
    assert_eq!(logins(&store), Vec::<String>::new());
}
