//! A restart checkpoint, and the replay window it turns into.
//!
//! X documents no redelivery: an event that arrives while a receiver is down
//! is lost unless someone asks X to replay it. The checkpoint records how far
//! a receiver got, at most once a minute, and [`replay_window`] turns it into
//! the minutes to ask X for, within the 24 hours X replays.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::store::atomic::write_atomically;
use crate::time::utc_minute;

/// How far back X replays deliveries to a webhook.
const REPLAY_HORIZON: Duration = Duration::from_secs(24 * 60 * 60);

const SECS_PER_MINUTE: u64 = 60;

/// The record on disk: one field, Unix seconds.
#[derive(Serialize, Deserialize)]
struct Record {
    last_event_at: u64,
}

/// Writes a checkpoint file, at most once a minute.
///
/// A replay window starts at a minute and includes it, so a record from
/// earlier in the same minute recovers what a record per event would, and a
/// burst of events costs one write a minute. Each write replaces the file
/// atomically: a crash mid-write leaves the previous record readable.
///
/// # Example
///
/// ```rust,no_run
/// use std::time::SystemTime;
/// use xdk::webhooks::CheckpointWriter;
///
/// let mut checkpoint = CheckpointWriter::new("listen-checkpoint.json");
/// // The arrival of the oldest event not yet handled, or of the newest
/// // when nothing is pending.
/// checkpoint.record(SystemTime::now())?;
/// # Ok::<(), xdk::Error>(())
/// ```
#[derive(Debug)]
pub struct CheckpointWriter {
    path: PathBuf,
    last_minute: Option<u64>,
}

impl CheckpointWriter {
    /// A writer for the checkpoint at `path`. Nothing is written until the
    /// first [`CheckpointWriter::record`]; a missing file is created then.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            last_minute: None,
        }
    }

    /// The file the writer replaces.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Records `at` when its minute differs from the minute last written,
    /// and returns whether it wrote.
    ///
    /// # Errors
    ///
    /// Returns an `io` error naming the file when it cannot be written; the
    /// next call tries again.
    pub fn record(&mut self, at: SystemTime) -> Result<bool> {
        let secs = unix_secs(at);
        let minute = secs / SECS_PER_MINUTE;
        if self.last_minute == Some(minute) {
            return Ok(false);
        }
        let mut bytes = serde_json::to_vec(&Record {
            last_event_at: secs,
        })?;
        bytes.push(b'\n');
        write_atomically(&self.path, &bytes).map_err(|e| {
            Error::io(format!(
                "could not write the checkpoint {}: {e}",
                self.path.display()
            ))
            .with_source(e)
        })?;
        self.last_minute = Some(minute);
        Ok(true)
    }
}

/// The time the checkpoint at `path` records.
///
/// # Errors
///
/// Returns an `io` error naming the file when it is missing or does not hold
/// a checkpoint, and a `validation` error when its time is later than `now`.
pub fn read_checkpoint(path: &Path, now: SystemTime) -> Result<SystemTime> {
    let bytes = std::fs::read(path).map_err(|e| {
        Error::io(format!(
            "could not read the checkpoint {}: {e}",
            path.display()
        ))
        .with_source(e)
    })?;
    let not_a_checkpoint = |cause: &dyn std::fmt::Display| {
        Error::io(format!(
            "{} is not a checkpoint, a JSON object whose last_event_at is a number of seconds: \
             {cause}",
            path.display()
        ))
    };
    let record: Record =
        serde_json::from_slice(&bytes).map_err(|e| not_a_checkpoint(&e).with_source(e))?;
    let at = UNIX_EPOCH
        .checked_add(Duration::from_secs(record.last_event_at))
        .ok_or_else(|| not_a_checkpoint(&"its time is past any clock"))?;
    not_ahead(at, now)?;
    Ok(at)
}

/// The minutes X can deliver again, from a checkpoint to now.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ReplayWindow {
    /// The first minute, `yyyymmddhhmm` in UTC, included.
    pub from: String,
    /// The last minute, `yyyymmddhhmm` in UTC, included.
    pub to: String,
    /// Whether the checkpoint was older than X replays, so `from` was moved
    /// up to the earliest minute X can deliver.
    pub clamped: bool,
    /// When clamped, the checkpoint's own minute: events from it up to
    /// `from` cannot be recovered.
    pub lost_from: Option<String>,
}

/// The window from the minute `checkpoint` falls in to the minute `now`
/// falls in, both included. A checkpoint older than the 24 hours X replays
/// moves `from` up to the first whole minute within them.
///
/// # Example
///
/// ```
/// use std::time::{Duration, UNIX_EPOCH};
/// use xdk::webhooks::replay_window;
///
/// // 2026-10-22T15:04:59Z, read back from a checkpoint, and 15:31:10Z.
/// let checkpoint = UNIX_EPOCH + Duration::from_secs(1_792_681_499);
/// let now = UNIX_EPOCH + Duration::from_secs(1_792_683_070);
/// let window = replay_window(checkpoint, now)?;
/// assert_eq!((window.from.as_str(), window.to.as_str()), ("202610221504", "202610221531"));
/// assert!(!window.clamped);
/// # Ok::<(), xdk::Error>(())
/// ```
///
/// # Errors
///
/// Returns a `validation` error when `checkpoint` is later than `now`.
pub fn replay_window(checkpoint: SystemTime, now: SystemTime) -> Result<ReplayWindow> {
    not_ahead(checkpoint, now)?;
    let checkpoint = unix_secs(checkpoint);
    let now = unix_secs(now);
    let horizon = now.saturating_sub(REPLAY_HORIZON.as_secs());
    let earliest = horizon.div_ceil(SECS_PER_MINUTE) * SECS_PER_MINUTE;
    let clamped = checkpoint < earliest;
    Ok(ReplayWindow {
        from: utc_minute(if clamped { earliest } else { checkpoint }),
        to: utc_minute(now),
        clamped,
        lost_from: clamped.then(|| utc_minute(checkpoint)),
    })
}

fn not_ahead(at: SystemTime, now: SystemTime) -> Result<()> {
    if at > now {
        return Err(Error::validation(format!(
            "the checkpoint's time is later than the clock's, by {} seconds",
            at.duration_since(now).map_or(0, |ahead| ahead.as_secs())
        )));
    }
    Ok(())
}

fn unix_secs(at: SystemTime) -> u64 {
    at.duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}
