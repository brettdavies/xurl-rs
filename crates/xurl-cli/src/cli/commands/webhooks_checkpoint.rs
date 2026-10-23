//! `xr webhooks listen --checkpoint`: how far `listen` got, written so a
//! later `xr webhooks replay --from-checkpoint` can request what a stop
//! missed.
//!
//! The time recorded is the arrival of the oldest event some forward target
//! has not finished, where finished means delivered or reported
//! `forward-failed`, and the arrival of the newest event when nothing is
//! pending. A hard stop with a backlog then leaves the oldest undelivered
//! minute behind, so the replay covers the backlog.

use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

use serde_json::json;
use xdk::error::Error;
use xdk::webhooks::CheckpointWriter;

use crate::cli::hints::NextStep;
use crate::cli::output::OutputConfig;

/// What `listen` has seen, and the checkpoint it keeps from it.
pub(super) struct Progress {
    writer: Option<CheckpointWriter>,
    newest: Option<SystemTime>,
}

impl Progress {
    /// Progress that writes to `checkpoint`, when one is named.
    pub(super) fn new(checkpoint: Option<&Path>) -> Self {
        Self {
            writer: checkpoint.map(CheckpointWriter::new),
            newest: None,
        }
    }

    /// An event arrived at `at`.
    pub(super) fn arrived(&mut self, at: SystemTime) {
        self.newest = Some(at);
    }

    /// Records the oldest pending arrival, or the newest arrival with none
    /// pending, when its minute moved. A failed write is reported on stderr
    /// and `listen` goes on.
    pub(super) fn advance(
        &mut self,
        oldest_pending: Option<SystemTime>,
        out: &OutputConfig,
        stderr: &mut dyn Write,
    ) {
        let (Some(writer), Some(newest)) = (self.writer.as_mut(), self.newest) else {
            return;
        };
        if let Err(error) = writer.record(oldest_pending.unwrap_or(newest)) {
            write_failed(out, stderr, &error);
        }
    }
}

/// The `checkpoint-failed` record.
fn write_failed(out: &OutputConfig, stderr: &mut dyn Write, error: &Error) {
    if out.format.is_structured() {
        let record = json!({
            "status": "checkpoint-failed",
            "cause": error.to_string(),
            "next_step": NextStep::fix_input(None),
        });
        let _ = writeln!(stderr, "{record}");
    } else {
        out.warning(
            stderr,
            &format!(
                "{error}. listen goes on, and the checkpoint keeps its last record until a write \
                 succeeds."
            ),
        );
    }
}
