//! Seeded failures for the nextest rollout's evidence. Dropped before the PR is ready.

use std::path::Path;

/// Never passes, so a retry cannot turn it green.
#[test]
fn probe_always_fails() {
    panic!("probe: this test always fails");
}

/// Fails its first attempt and passes its second, through a marker file in the
/// target's scratch directory, which survives the retry's new process.
#[test]
fn probe_fails_once_then_passes() {
    let marker = Path::new(env!("CARGO_TARGET_TMPDIR")).join("probe-fails-once");
    if !marker.exists() {
        std::fs::write(&marker, "first attempt").expect("write the marker");
        panic!("probe: failing the first attempt on purpose");
    }
    std::fs::remove_file(&marker).expect("remove the marker");
}

/// Never returns, so only nextest's slow-timeout ends it.
#[test]
fn probe_hangs() {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}
