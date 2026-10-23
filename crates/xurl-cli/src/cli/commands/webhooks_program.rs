//! The program after `--` as a forward target: run once per event, with the
//! event's body on its stdin, no shell, and no terminal.
//!
//! Bytes from X reach the program on stdin alone, never in its arguments, its
//! environment, or a shell, so an event cannot become a command.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tokio::process::{ChildStdin, Command};
use xdk::error::Error;
use xdk::webhooks::{Attempt, AttemptTimedOut, Event};

use super::process_tree::ProcessTree;
use crate::cli::env::ProgramSearch;

/// The variable that carries the event's SHA-256, the key a URL target gets
/// as `Idempotency-Key`, as bare lowercase hex.
const EVENT_SHA256_VAR: &str = "XR_EVENT_SHA256";

/// A program found before `listen` binds, with the arguments it is given.
pub(super) struct ProgramTarget {
    program: PathBuf,
    args: Vec<String>,
}

/// Why one run of the program did not take the event.
#[derive(Debug)]
pub(super) enum ProgramFailure {
    /// It exited non-zero, or a signal ended it (`None`).
    Exit(Option<i32>),
    /// It could not be started.
    Start(String),
    /// It ran past the attempt timeout and was stopped with everything it
    /// started.
    TimedOut(Duration),
}

impl fmt::Display for ProgramFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exit(Some(code)) => write!(f, "the program exited with {code}"),
            Self::Exit(None) => write!(f, "a signal ended the program"),
            Self::Start(cause) => write!(f, "the program could not start: {cause}"),
            Self::TimedOut(after) => {
                write!(f, "the program ran past {after:?} and was stopped")
            }
        }
    }
}

impl From<AttemptTimedOut> for ProgramFailure {
    fn from(timed_out: AttemptTimedOut) -> Self {
        Self::TimedOut(timed_out.0)
    }
}

impl ProgramTarget {
    /// The program `argv` names, found the way the OS would find it.
    ///
    /// # Errors
    ///
    /// A `validation` error when no file matches.
    pub(super) fn resolve(argv: &[String], search: &ProgramSearch) -> Result<Self, Error> {
        let (name, args) = argv
            .split_first()
            .ok_or_else(|| Error::validation("-- names no program"))?;
        let program = find(name, search).ok_or_else(|| {
            Error::validation(format!(
                "the program after -- was not found: {name} is not a path to a file, and no \
                 directory on PATH holds it"
            ))
        })?;
        Ok(Self {
            program,
            args: args.to_vec(),
        })
    }

    /// One run of the program on `event`, with no timeout of its own: the
    /// budget's cut drops the run, and the drop ends the program's tree.
    pub(super) async fn attempt(&self, event: &Event) -> Attempt<i32, ProgramFailure> {
        let mut command = Command::new(&self.program);
        command
            .args(&self.args)
            .env(EVENT_SHA256_VAR, event.sha256())
            .stdin(Stdio::piped())
            .stdout(std::io::stderr())
            .stderr(Stdio::inherit());
        let mut tree = match ProcessTree::spawn(command) {
            Ok(tree) => tree,
            Err(error) => return Attempt::Final(ProgramFailure::Start(error.to_string())),
        };
        let write = write_stdin(tree.child.stdin.take(), event.body.clone());
        let ((), waited) = tokio::join!(write, tree.child.wait());
        match waited {
            Ok(status) => {
                tree.release();
                match status.code() {
                    Some(0) => Attempt::Delivered(0),
                    code => Attempt::Retryable(ProgramFailure::Exit(code)),
                }
            }
            Err(error) => Attempt::Retryable(ProgramFailure::Start(error.to_string())),
        }
    }
}

/// Writes `body` to the program's stdin, then closes it.
///
/// `xr` keeps SIGPIPE's default action, which ends the process on a write
/// to a pipe nobody reads, and a program may exit without reading. On Unix
/// the write therefore runs on a thread of its own with SIGPIPE blocked: the
/// signal is directed at that thread, the write returns `EPIPE` instead, and
/// the pending signal ends with the thread.
async fn write_stdin(stdin: Option<ChildStdin>, body: Vec<u8>) {
    let Some(stdin) = stdin else {
        return;
    };
    #[cfg(unix)]
    {
        use std::io::Write as _;
        let Ok(fd) = stdin.into_owned_fd() else {
            return;
        };
        let (done, finished) = tokio::sync::oneshot::channel();
        std::thread::spawn(move || {
            block_sigpipe();
            let _ = std::fs::File::from(fd).write_all(&body);
            let _ = done.send(());
        });
        let _ = finished.await;
    }
    #[cfg(not(unix))]
    {
        use tokio::io::AsyncWriteExt as _;
        let mut stdin = stdin;
        let _ = stdin.write_all(&body).await;
    }
}

/// Blocks SIGPIPE for the calling thread alone.
#[cfg(unix)]
fn block_sigpipe() {
    let broken_pipe = libc::SIGPIPE;
    // SAFETY: `set` is initialized by sigemptyset before it is read, and
    // pthread_sigmask changes only the calling thread's mask.
    unsafe {
        let mut set: libc::sigset_t = std::mem::zeroed();
        libc::sigemptyset(&mut set);
        libc::sigaddset(&mut set, broken_pipe);
        libc::pthread_sigmask(libc::SIG_BLOCK, &set, std::ptr::null_mut());
    }
}

/// The file `name` runs: itself when it holds a path separator, otherwise
/// the first match in a `PATH` directory, trying each `PATHEXT` extension on
/// Windows when `name` has none.
fn find(name: &str, search: &ProgramSearch) -> Option<PathBuf> {
    if name.chars().any(std::path::is_separator) {
        return candidates(Path::new(name), search)
            .into_iter()
            .find(|path| runnable(path));
    }
    let dirs = search.path.as_ref()?;
    std::env::split_paths(dirs)
        .flat_map(|dir| candidates(&dir.join(name), search))
        .find(|path| runnable(path))
}

/// `path` itself, then, on Windows and for a name with no extension, `path`
/// with each `PATHEXT` extension.
fn candidates(path: &Path, search: &ProgramSearch) -> Vec<PathBuf> {
    let mut found = vec![path.to_path_buf()];
    if cfg!(windows) && path.extension().is_none() {
        let extensions = search.pathext.as_deref().unwrap_or(".COM;.EXE;.BAT;.CMD");
        found.extend(
            extensions
                .split(';')
                .filter(|ext| !ext.is_empty())
                .map(|ext| {
                    let mut with = path.as_os_str().to_owned();
                    with.push(ext);
                    PathBuf::from(with)
                }),
        );
    }
    found
}

/// Whether `path` is a file the OS would run: on Unix, one with an execute
/// bit set.
fn runnable(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        metadata.is_file()
    }
}
