//! A forward program's stdout and stderr, passed on to `listen`'s stderr in
//! whole lines.
//!
//! `listen` writes one record per line to the same stderr, so a program's
//! output is passed on only a line at a time, and output still unfinished
//! when the program's stream closes is ended with a newline. A record never
//! lands in the middle of a program's line.

use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::task::JoinHandle;

/// The most output held back waiting for a newline. Longer output is passed
/// on as a line of its own, so a program that never ends a line cannot grow
/// `listen`'s memory without bound.
const MAX_LINE: usize = 64 * 1024;

/// How long a program that has exited gets for its output to close before
/// its record is written. A process the program started can hold the output
/// open longer; its lines follow as it writes them.
const SETTLE: Duration = Duration::from_millis(500);

/// Output read from a program, cut into complete lines.
#[derive(Default)]
struct Lines {
    held: Vec<u8>,
}

impl Lines {
    /// Adds `chunk` and returns every line it completes, newlines included.
    fn push(&mut self, chunk: &[u8]) -> Option<Vec<u8>> {
        self.held.extend_from_slice(chunk);
        let end = match self.held.iter().rposition(|&byte| byte == b'\n') {
            Some(last) => last + 1,
            None if self.held.len() >= MAX_LINE => {
                self.held.push(b'\n');
                self.held.len()
            }
            None => return None,
        };
        let rest = self.held.split_off(end);
        Some(std::mem::replace(&mut self.held, rest))
    }

    /// What is left when the output closes, ended with a newline.
    fn finish(mut self) -> Option<Vec<u8>> {
        if self.held.is_empty() {
            return None;
        }
        self.held.push(b'\n');
        Some(self.held)
    }
}

/// Passes `output` to `sink` in whole lines until it closes, on a task of
/// its own.
pub(super) fn relay<R, F>(output: Option<R>, sink: F) -> Option<JoinHandle<()>>
where
    R: AsyncRead + Unpin + Send + 'static,
    F: Fn(Vec<u8>) + Send + 'static,
{
    let mut output = output?;
    Some(tokio::spawn(async move {
        let mut lines = Lines::default();
        let mut chunk = [0u8; 8192];
        while let Ok(read @ 1..) = output.read(&mut chunk).await {
            if let Some(complete) = lines.push(&chunk[..read]) {
                sink(complete);
            }
        }
        if let Some(rest) = lines.finish() {
            sink(rest);
        }
    }))
}

/// Waits, up to a moment, for the relays of a program that has exited to
/// pass on the last of its output, so its lines come before its record.
pub(super) async fn settle(relays: impl IntoIterator<Item = Option<JoinHandle<()>>>) {
    let closed = async {
        for relay in relays.into_iter().flatten() {
            let _ = relay.await;
        }
    };
    let _ = tokio::time::timeout(SETTLE, closed).await;
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use tokio::io::AsyncWriteExt;

    use super::{Lines, MAX_LINE, relay};

    #[test]
    fn complete_lines_pass_and_an_unfinished_one_waits() {
        let mut lines = Lines::default();

        assert_eq!(lines.push(b"one\ntw"), Some(b"one\n".to_vec()));
        assert_eq!(lines.push(b"o"), None);
        assert_eq!(lines.push(b"\nthree\nfo"), Some(b"two\nthree\n".to_vec()));
        assert_eq!(lines.finish(), Some(b"fo\n".to_vec()));
    }

    #[test]
    fn output_that_ends_on_a_newline_leaves_nothing_to_finish() {
        let mut lines = Lines::default();

        assert_eq!(lines.push(b"done\n"), Some(b"done\n".to_vec()));
        assert_eq!(lines.finish(), None);
    }

    #[test]
    fn output_with_no_newline_passes_once_it_reaches_the_limit() {
        let mut lines = Lines::default();

        assert_eq!(lines.push(&vec![b'x'; MAX_LINE - 1]), None);
        let passed = lines.push(b"x").expect("the limit passes it on");
        assert_eq!(passed.len(), MAX_LINE + 1);
        assert_eq!(passed.last(), Some(&b'\n'));
        assert_eq!(lines.finish(), None);
    }

    #[tokio::test]
    async fn a_relay_ends_the_last_line_when_the_output_closes() {
        let (mut program, output) = tokio::io::duplex(64);
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = {
            let seen = Arc::clone(&seen);
            move |lines: Vec<u8>| seen.lock().expect("unpoisoned").extend(lines)
        };
        let relayed = relay(Some(output), sink).expect("a relay");

        program
            .write_all(b"first\nunfinished")
            .await
            .expect("write");
        drop(program);
        relayed.await.expect("the relay ends");

        assert_eq!(*seen.lock().expect("unpoisoned"), b"first\nunfinished\n");
    }
}
