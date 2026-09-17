//! The process-level stop: SIGINT (Ctrl+C) and, on Unix, SIGTERM.
//!
//! Registering a signal handler changes the disposition for the whole
//! process, so only the binary does it; the library takes a
//! [`CancellationToken`] and never touches signals.

use tokio_util::sync::CancellationToken;

/// Resolves when the process receives SIGINT (Ctrl+C) or, on Unix, SIGTERM.
///
/// On Windows only `ctrl_c()` is available; the `cfg(unix)` arm folds
/// SIGTERM into the same future.
pub(crate) async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = match signal(SignalKind::terminate()) {
            Ok(s) => s,
            Err(_) => {
                // SignalKind::terminate is documented to always succeed on
                // Unix; if it does fail, fall back to ctrl_c only.
                let _ = tokio::signal::ctrl_c().await;
                return;
            }
        };
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = term.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

/// A token that is cancelled when the process receives its shutdown signal.
///
/// The watcher task lives for the rest of the process; a second call
/// registers a second watcher, which is harmless.
pub(crate) fn cancel_on_shutdown() -> CancellationToken {
    let token = CancellationToken::new();
    let cancelled = token.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        cancelled.cancel();
    });
    token
}
