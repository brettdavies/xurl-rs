//! The retry rule a delivery to any kind of target runs under.

use std::future::Future;
use std::time::Duration;

use super::WEBHOOKS_TARGET;

/// Attempts a [`RetryBudget`] makes, the first included.
const MAX_ATTEMPTS: u32 = 4;

/// The wait before the second attempt; each later wait doubles it.
const FIRST_WAIT: Duration = Duration::from_secs(1);

/// What one attempt at a delivery came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attempt<T, E> {
    /// The target took the event.
    Delivered(T),
    /// The attempt failed in a way another attempt can fix.
    Retryable(E),
    /// The attempt failed in a way another attempt would repeat.
    Final(E),
}

/// An attempt outlasted its [`RetryBudget::attempt_timeout`]. A failure type
/// a budget runs under converts from it, so the budget can report the cut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttemptTimedOut(pub Duration);

/// What a delivery under a [`RetryBudget`] came to.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Outcome<T, E> {
    /// Attempts made, the first included.
    pub attempts: u32,
    /// Time from the start of the first attempt to the end of the last,
    /// waits included.
    pub elapsed: Duration,
    /// What the delivered attempt returned, or the last attempt's failure.
    pub result: std::result::Result<T, E>,
}

/// The rule a forward retries by: up to four attempts, waiting 1, 2, then 4
/// seconds between them, each cut off at the attempt timeout.
///
/// The waits add up to 7 seconds, long enough for a local server to
/// restart. A target down for longer is reported and recovered by a replay,
/// since a longer budget would hold up every event queued behind it.
///
/// # Example
///
/// ```
/// use std::time::Duration;
/// use xdk::webhooks::{Attempt, ForwardFailure, RetryBudget};
///
/// #[tokio::main]
/// async fn main() {
///     let budget = RetryBudget::new(Duration::from_secs(5))
///         .with_first_wait(Duration::from_millis(10));
///     let mut answers = vec![503, 503, 200].into_iter();
///     let outcome = budget
///         .run(|| {
///             let status = answers.next().unwrap_or(200);
///             async move {
///                 if status == 200 {
///                     Attempt::Delivered(status)
///                 } else {
///                     Attempt::Retryable(ForwardFailure::Status(status))
///                 }
///             }
///         })
///         .await;
///     assert_eq!(outcome.result, Ok(200));
///     assert_eq!(outcome.attempts, 3);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryBudget {
    first_wait: Duration,
    attempt_timeout: Duration,
}

impl RetryBudget {
    /// The attempt timeout `xr` uses unless told otherwise. A program target
    /// may run an agent, which takes its time.
    pub const DEFAULT_ATTEMPT_TIMEOUT: Duration = Duration::from_secs(30);

    /// The budget's rule with each attempt cut off at `attempt_timeout`.
    #[must_use]
    pub fn new(attempt_timeout: Duration) -> Self {
        Self {
            first_wait: FIRST_WAIT,
            attempt_timeout,
        }
    }

    /// The same rule with `first_wait` before the second attempt instead of
    /// one second; each later wait still doubles the one before. A target on
    /// the same host that restarts quickly, or a test, can wait less.
    #[must_use]
    pub fn with_first_wait(self, first_wait: Duration) -> Self {
        Self { first_wait, ..self }
    }

    /// How long one attempt may take before it is cut off and counted
    /// failed.
    #[must_use]
    pub fn attempt_timeout(&self) -> Duration {
        self.attempt_timeout
    }

    /// The waits between attempts, summed: 7 seconds under the default
    /// first wait.
    #[must_use]
    pub fn total_wait(&self) -> Duration {
        self.first_wait
            .saturating_mul((1 << (MAX_ATTEMPTS - 1)) - 1)
    }

    /// Runs `attempt` until it delivers, fails finally, or the attempts run
    /// out. An attempt that outlasts the attempt timeout is dropped and
    /// counted as a retryable failure.
    pub async fn run<T, E, F, Fut>(&self, mut attempt: F) -> Outcome<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Attempt<T, E>>,
        E: From<AttemptTimedOut>,
    {
        let started = tokio::time::Instant::now();
        let mut wait = self.first_wait;
        let mut attempts = 0;
        loop {
            attempts += 1;
            let verdict = tokio::time::timeout(self.attempt_timeout, attempt())
                .await
                .unwrap_or_else(|_| {
                    Attempt::Retryable(E::from(AttemptTimedOut(self.attempt_timeout)))
                });
            let result = match verdict {
                Attempt::Delivered(value) => Ok(value),
                Attempt::Retryable(_) if attempts < MAX_ATTEMPTS => {
                    tracing::debug!(target: WEBHOOKS_TARGET, kind = "forward-retry", attempts);
                    tokio::time::sleep(wait).await;
                    wait = wait.saturating_mul(2);
                    continue;
                }
                Attempt::Retryable(cause) | Attempt::Final(cause) => Err(cause),
            };
            return Outcome {
                attempts,
                elapsed: started.elapsed(),
                result,
            };
        }
    }
}

impl Default for RetryBudget {
    fn default() -> Self {
        Self::new(Self::DEFAULT_ATTEMPT_TIMEOUT)
    }
}
