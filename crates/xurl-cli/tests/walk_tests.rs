//! `walk`, the concurrent visit the guards share.

mod common;

use std::sync::mpsc;
use std::time::Duration;

use common::invoke::walk;
use wiremock::MockServer;

/// Runs that send nothing never yield, so a walk over them can end with the
/// task's coop budget spent, and a `MockServer`'s drop blocks on a tokio lock.
/// The walk runs on a thread of its own so that a drop that never returns fails
/// this test rather than hanging the binary.
#[test]
fn a_walk_ends_after_a_visit_spends_the_tasks_budget() {
    let (ended, on_end) = mpsc::channel();
    std::thread::spawn(move || {
        walk(&[()], MockServer::start, async |_: &MockServer, (): &()| {
            while tokio::task::coop::has_budget_remaining() {
                tokio::task::coop::consume_budget().await;
            }
        });
        ended.send(()).expect("the test is waiting on this");
    });
    on_end
        .recv_timeout(Duration::from_secs(30))
        .expect("the walk did not end: dropping its mock server never returned");
}
