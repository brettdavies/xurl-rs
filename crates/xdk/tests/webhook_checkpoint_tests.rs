//! A checkpoint records, at most once a minute, the arrival time a restarted
//! receiver resumes from, and a replay window turns it into the minutes X
//! can deliver again: from the checkpoint's minute to the current one,
//! within the 24 hours X replays.
//!
//! Every time is fixed; nothing here reads the clock or reaches X.

use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tempfile::TempDir;
use xdk::webhooks::{
    CHECKPOINT_SHAPE, CheckpointWriter, ReplayWindow, read_checkpoint, replay_window,
};

/// 2026-10-22T15:04:10Z and the seconds around it, computed with Python's
/// `datetime`.
const AT_15_04_10: u64 = 1_792_681_450;
const AT_15_04_50: u64 = 1_792_681_490;
const AT_15_04_59: u64 = 1_792_681_499;
const AT_15_05_02: u64 = 1_792_681_502;
const AT_15_31_10: u64 = 1_792_683_070;
/// 2026-10-21T09:31:10Z, thirty hours before 15:31:10 the next day.
const THIRTY_HOURS_BEFORE: u64 = 1_792_575_070;

fn at(secs: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(secs)
}

fn window(checkpoint: u64, now: u64) -> ReplayWindow {
    replay_window(at(checkpoint), at(now)).expect("a window")
}

#[test]
fn the_window_runs_from_the_checkpoints_minute_to_the_current_one() {
    let window = window(AT_15_04_59, AT_15_31_10);

    assert_eq!(window.from, "202610221504");
    assert_eq!(window.to, "202610221531");
    assert!(!window.clamped);
    assert_eq!(window.lost_from, None);
}

#[test]
fn a_checkpoint_older_than_a_day_is_clamped_and_names_what_is_lost() {
    let window = window(THIRTY_HOURS_BEFORE, AT_15_31_10);

    assert_eq!(
        window.from, "202610211532",
        "the first whole minute within 24 hours of now"
    );
    assert_eq!(window.to, "202610221531");
    assert!(window.clamped);
    assert_eq!(window.lost_from.as_deref(), Some("202610210931"));
}

#[test]
fn a_checkpoint_in_the_current_minute_gives_one_minute() {
    let window = window(AT_15_04_10, AT_15_04_59);

    assert_eq!(window.from, "202610221504");
    assert_eq!(window.to, window.from);
}

#[test]
fn a_checkpoint_ahead_of_the_clock_is_refused() {
    let err = replay_window(at(AT_15_31_10), at(AT_15_04_10)).expect_err("ahead of now");
    assert_eq!(err.kind(), "validation", "{err}");
}

fn files_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .expect("readable")
        .map(|entry| {
            entry
                .expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

#[test]
fn a_later_minute_replaces_the_record_and_leaves_no_temp_file() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("checkpoint.json");
    let mut writer = CheckpointWriter::new(&path);

    assert!(writer.record(at(AT_15_04_10)).expect("written"));
    assert!(writer.record(at(AT_15_05_02)).expect("written"));

    assert_eq!(
        read_checkpoint(&path, at(AT_15_31_10)).expect("readable"),
        at(AT_15_05_02)
    );
    assert_eq!(files_in(tmp.path()), ["checkpoint.json"]);
    let record: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("the file")).expect("JSON");
    assert_eq!(record, serde_json::json!({"last_event_at": AT_15_05_02}));
}

#[test]
fn a_second_arrival_in_the_same_minute_writes_nothing() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("checkpoint.json");
    let mut writer = CheckpointWriter::new(&path);

    assert!(writer.record(at(AT_15_04_10)).expect("written"));
    assert!(!writer.record(at(AT_15_04_50)).expect("skipped"));

    assert_eq!(
        read_checkpoint(&path, at(AT_15_31_10)).expect("readable"),
        at(AT_15_04_10)
    );
}

#[rstest::rstest]
#[case::missing(None)]
#[case::not_json(Some("not json"))]
#[case::not_a_number(Some(r#"{"last_event_at": "soon"}"#))]
#[case::negative(Some(r#"{"last_event_at": -5}"#))]
#[case::past_any_clock(Some(r#"{"last_event_at": 18446744073709551615}"#))]
fn a_checkpoint_that_cannot_be_read_is_refused_naming_its_path(#[case] contents: Option<&str>) {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("checkpoint.json");
    if let Some(contents) = contents {
        std::fs::write(&path, contents).expect("write the file");
    }

    let err = read_checkpoint(&path, at(AT_15_31_10)).expect_err("refused");

    assert_eq!(err.kind(), "io", "{err}");
    assert!(
        err.to_string().contains(&path.display().to_string()),
        "the refusal names the file: {err}"
    );
}

#[test]
fn a_checkpoint_file_ahead_of_the_clock_is_refused() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("checkpoint.json");
    std::fs::write(&path, format!(r#"{{"last_event_at": {AT_15_31_10}}}"#)).expect("write");

    let err = read_checkpoint(&path, at(AT_15_04_10)).expect_err("ahead of now");

    assert_eq!(err.kind(), "validation", "{err}");
}

/// The shape `xr` prints in its help, with a time where the placeholder is.
fn shaped(secs: u64) -> String {
    CHECKPOINT_SHAPE.replace("<Unix seconds>", &secs.to_string())
}

/// A file written by hand in the documented shape reads back as the time it
/// names, so a listener other than `xr` can write one.
#[test]
fn a_file_in_the_documented_shape_reads_back() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("checkpoint.json");
    std::fs::write(&path, shaped(AT_15_05_02)).expect("write the file");

    assert_eq!(
        read_checkpoint(&path, at(AT_15_31_10)).expect("readable"),
        at(AT_15_05_02)
    );
}

/// The writer records the keys the documented shape names, and no other.
#[test]
fn the_writer_writes_the_documented_shape() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("checkpoint.json");
    CheckpointWriter::new(&path)
        .record(at(AT_15_04_10))
        .expect("written");

    let written: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("the file")).expect("JSON");
    let documented: serde_json::Value =
        serde_json::from_str(&shaped(AT_15_04_10)).expect("the shape is JSON once filled in");
    assert_eq!(written, documented);
}

/// A file that is not a checkpoint is refused naming the shape it should hold.
#[test]
fn a_refusal_names_the_documented_shape() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("checkpoint.json");
    std::fs::write(&path, "{}").expect("write the file");

    let err = read_checkpoint(&path, at(AT_15_31_10)).expect_err("refused");

    assert!(err.to_string().contains(CHECKPOINT_SHAPE), "{err}");
}
