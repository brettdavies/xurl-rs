//! How a command handler reports that it stopped.

use xdk::error::Error;

/// Why a handler returned early, as the runner sees it.
///
/// A handler that has already written the canonical envelope hands back only
/// the exit code, so the runner never prints a second error on top of it.
#[derive(Debug)]
pub(crate) enum Failure {
    /// A library error the runner still has to render.
    Error(Error),
    /// The handler wrote the envelope itself; only the exit code remains.
    Emitted {
        /// Exit code the process surfaces.
        exit_code: i32,
    },
}

impl From<Error> for Failure {
    fn from(error: Error) -> Self {
        Self::Error(error)
    }
}

/// The result every command handler returns.
pub(crate) type CommandResult<T = ()> = std::result::Result<T, Failure>;
