//! How a command handler reports that it stopped.

use xdk::error::Error;

use crate::cli::envelope::Reason;

/// Why a handler returned early, as the runner sees it.
///
/// A handler that has already written the canonical envelope hands back only
/// the exit code, so the runner never prints a second error on top of it.
#[derive(Debug)]
pub(crate) enum Failure {
    /// A library error the runner still has to render.
    Error(Error),
    /// An argument the handler refused. The runner renders `error` with the
    /// help of the command the invocation names as its step; `reason` is
    /// the refusal's own name when it has one apart from the error's kind.
    Refused {
        /// What was refused, as the message says it.
        error: Error,
        /// The reason the envelope carries in place of the error's kind.
        reason: Option<Reason>,
    },
    /// A destructive command that ran without its confirmation. The value
    /// names what it would have destroyed; the runner renders it with the
    /// invocation that confirms.
    Unconfirmed(serde_json::Value),
    /// The handler wrote the envelope itself; only the exit code remains.
    Emitted {
        /// Exit code the process surfaces.
        exit_code: i32,
    },
}

impl Failure {
    /// An argument refused under the error's own reason.
    pub(crate) fn refused(error: Error) -> Self {
        Self::Refused {
            error,
            reason: None,
        }
    }
}

impl From<Error> for Failure {
    fn from(error: Error) -> Self {
        Self::Error(error)
    }
}

/// The result every command handler returns.
pub(crate) type CommandResult<T = ()> = std::result::Result<T, Failure>;
