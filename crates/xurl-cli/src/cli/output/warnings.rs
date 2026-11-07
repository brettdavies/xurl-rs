//! The warnings a structured run keeps for the one document it writes on
//! stderr: its error envelope, or the status-ok document that closes a run
//! that succeeded.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::cli::envelope::Warning;

/// The warnings kept so far. Every clone shares one list, because
/// `Diagnostics` records through its own clone of the run's `OutputConfig`
/// while a handler writes its envelope through another.
#[derive(Clone, Debug, Default)]
pub struct WarningLog(Arc<Mutex<Vec<Warning>>>);

impl WarningLog {
    /// Keeps a warning unless the same one is already kept. A warning can
    /// fire once per app or per request, and the repeat tells a caller
    /// nothing.
    pub(super) fn record(&self, message: &str, target: &str) {
        let mut kept = self.lock();
        let seen = kept
            .iter()
            .any(|warning| warning.message == message && warning.target == target);
        if !seen {
            kept.push(Warning {
                message: message.to_string(),
                target: target.to_string(),
            });
        }
    }

    /// Hands over what is kept, leaving the log empty, so one document
    /// carries each warning. `None` when nothing is kept.
    pub(super) fn take(&self) -> Option<Vec<Warning>> {
        let kept = std::mem::take(&mut *self.lock());
        (!kept.is_empty()).then_some(kept)
    }

    fn lock(&self) -> MutexGuard<'_, Vec<Warning>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
