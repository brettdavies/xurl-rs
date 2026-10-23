//! The `internal` error a panic anywhere in `xr` answers with.
//!
//! The release profile aborts on a panic (`panic = "abort"` in the workspace
//! manifest), so nothing after the panic runs: no join handle and no
//! `catch_unwind` sees it. The hook runs before the abort. It writes the
//! error in the active output format, flushes stdout, and exits 1 itself.

use std::backtrace::{Backtrace, BacktraceStatus};
use std::io::Write as _;
use std::panic::PanicHookInfo;
use std::sync::{Mutex, PoisonError};

use xdk::error::{EXIT_GENERAL_ERROR, Error};

use crate::cli::output::OutputConfig;

/// The output settings the runner resolved. Until they are known, a panic
/// is written as text.
static ACTIVE: Mutex<Option<OutputConfig>> = Mutex::new(None);

/// Records the output settings a panic is written in. The runner calls this
/// on every run; it matters only in a process that installed the hook.
pub fn publish(out: &OutputConfig) {
    *ACTIVE.lock().unwrap_or_else(PoisonError::into_inner) = Some(out.clone());
}

/// Installs the hook for the rest of the process. `run_argv`, the binary's
/// entry, is the one caller, so a caller that runs `xr` in process keeps
/// Rust's own panic behavior.
pub fn install() {
    std::panic::set_hook(Box::new(report));
}

fn report(info: &PanicHookInfo<'_>) {
    let location = info.location().map_or_else(
        || "an unknown location".to_string(),
        |at| format!("{}:{}:{}", at.file(), at.line(), at.column()),
    );
    let payload = info
        .payload()
        .downcast_ref::<&str>()
        .map(|message| (*message).to_string())
        .or_else(|| info.payload().downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "no message".to_string());
    let error = Error::Internal(format!("panicked at {location}: {payload}"));
    let backtrace = Backtrace::capture();
    let backtrace =
        (backtrace.status() == BacktraceStatus::Captured).then(|| backtrace.to_string());
    let out = ACTIVE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
        .unwrap_or_default();
    let _ = std::io::stdout().flush();
    out.print_panic(&mut std::io::stderr(), &error, backtrace);
    std::process::exit(EXIT_GENERAL_ERROR);
}
