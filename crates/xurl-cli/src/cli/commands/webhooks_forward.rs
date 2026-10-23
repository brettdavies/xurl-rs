//! Forward targets for `xr webhooks listen`: one worker per target over a
//! bounded queue, the stderr record of what each delivery came to, and the
//! drain at shutdown.
//!
//! Workers report every finished delivery to the consume loop, in queue
//! order, along with a program's output a line at a time, and the loop
//! writes both: stderr belongs to the run, and stdout keeps one line per
//! event.

use std::collections::VecDeque;
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::json;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use xdk::time::utc_rfc3339;
use xdk::webhooks::{Event, RetryBudget, UrlTarget};

use super::webhooks_checkpoint::Progress;
use super::webhooks_program::ProgramTarget;
use crate::cli::hints::NextStep;
use crate::cli::output::OutputConfig;

/// Events one target holds before `listen` takes no more. A full queue
/// holds X's answer back rather than drop an event.
const QUEUE_CAPACITY: usize = 256;

/// What the caller has done with an event that a forward failed to deliver.
const FAILED_SIDE_EFFECTS: &str = "printed, not delivered";

/// Where `listen` sends each verified event besides stdout.
pub(super) enum Target {
    /// The caller's HTTP endpoint.
    Url(UrlTarget),
    /// The program after `--`, run once per event.
    Program(ProgramTarget),
}

impl Target {
    /// The name records give the target.
    fn kind(&self) -> &'static str {
        match self {
            Self::Url(_) => "url",
            Self::Program(_) => "program",
        }
    }

    /// Delivers `event` under `budget`; a program's output goes to `report`.
    async fn deliver(
        &self,
        event: &Event,
        budget: &RetryBudget,
        report: &mpsc::UnboundedSender<Report>,
    ) -> Delivery {
        match self {
            Self::Url(target) => {
                let outcome = target.forward(event, budget).await;
                Delivery {
                    attempts: outcome.attempts,
                    elapsed: outcome.elapsed,
                    result: outcome
                        .result
                        .map(Answer::HttpStatus)
                        .map_err(|failure| failure.to_string()),
                }
            }
            Self::Program(target) => {
                let output = {
                    let report = report.clone();
                    move |lines| {
                        let _ = report.send(Report::Output(lines));
                    }
                };
                let outcome = budget.run(|| target.attempt(event, output.clone())).await;
                Delivery {
                    attempts: outcome.attempts,
                    elapsed: outcome.elapsed,
                    result: outcome
                        .result
                        .map(Answer::ExitStatus)
                        .map_err(|failure| failure.to_string()),
                }
            }
        }
    }
}

/// What a target answered with when it took an event.
enum Answer {
    HttpStatus(u16),
    ExitStatus(i32),
}

/// What delivering one event came to.
struct Delivery {
    attempts: u32,
    elapsed: Duration,
    result: Result<Answer, String>,
}

/// One delivery a worker finished, delivered or failed.
pub(super) struct Finished {
    target: usize,
    event: Arc<Event>,
    delivery: Delivery,
}

/// What a worker sends the consume loop.
pub(super) enum Report {
    /// A delivery it finished, delivered or failed.
    Finished(Finished),
    /// Whole lines a program wrote, newlines included.
    Output(Vec<u8>),
}

/// One target's name and queue.
struct Lane {
    kind: &'static str,
    queue: Option<mpsc::Sender<Arc<Event>>>,
}

/// The arrivals of the events each target has not finished, oldest first.
/// A worker finishes its events in queue order, so a finish always takes
/// the front.
pub(super) struct Pending {
    lanes: Vec<VecDeque<SystemTime>>,
}

impl Pending {
    pub(super) fn new(targets: usize) -> Self {
        Self {
            lanes: vec![VecDeque::new(); targets],
        }
    }

    pub(super) fn queued(&mut self, target: usize, arrival: SystemTime) {
        self.lanes[target].push_back(arrival);
    }

    pub(super) fn finished(&mut self, target: usize) {
        self.lanes[target].pop_front();
    }

    pub(super) fn undelivered(&self, target: usize) -> usize {
        self.lanes[target].len()
    }

    /// The arrival of the oldest event some target has not finished.
    pub(super) fn oldest(&self) -> Option<SystemTime> {
        self.lanes
            .iter()
            .filter_map(|lane| lane.front().copied())
            .min()
    }
}

/// The running forward targets of one `listen`.
pub(super) struct Forwarding {
    lanes: Vec<Lane>,
    pending: Pending,
    workers: Vec<JoinHandle<()>>,
    reports: mpsc::UnboundedReceiver<Report>,
    budget: RetryBudget,
}

impl Forwarding {
    /// Starts one worker per target, each draining its own queue in order.
    pub(super) fn start(targets: Vec<Target>, budget: RetryBudget) -> Self {
        let (report, reports) = mpsc::unbounded_channel();
        let mut lanes = Vec::with_capacity(targets.len());
        let mut workers = Vec::with_capacity(targets.len());
        for (index, target) in targets.into_iter().enumerate() {
            let (queue, mut events) = mpsc::channel::<Arc<Event>>(QUEUE_CAPACITY);
            lanes.push(Lane {
                kind: target.kind(),
                queue: Some(queue),
            });
            let report = report.clone();
            workers.push(tokio::spawn(async move {
                while let Some(event) = events.recv().await {
                    let delivery = target.deliver(&event, &budget, &report).await;
                    let finished = Finished {
                        target: index,
                        event,
                        delivery,
                    };
                    if report.send(Report::Finished(finished)).is_err() {
                        return;
                    }
                }
            }));
        }
        Self {
            pending: Pending::new(lanes.len()),
            lanes,
            workers,
            reports,
            budget,
        }
    }

    /// Puts `event` on every target's queue, waiting while a queue is full.
    pub(super) async fn queue(&mut self, event: &Arc<Event>) {
        for (index, lane) in self.lanes.iter().enumerate() {
            let Some(queue) = &lane.queue else {
                continue;
            };
            if queue.send(Arc::clone(event)).await.is_ok() {
                self.pending.queued(index, event.received_at);
            }
        }
    }

    /// The arrival of the oldest event some target has not finished.
    pub(super) fn oldest_pending(&self) -> Option<SystemTime> {
        self.pending.oldest()
    }

    /// The next delivery a worker finished. A program's output that comes
    /// first is written to `stderr` as it arrives.
    pub(super) async fn next_finished(&mut self, stderr: &mut dyn Write) -> Option<Finished> {
        loop {
            match self.reports.recv().await? {
                Report::Finished(finished) => return Some(finished),
                Report::Output(lines) => write_output(stderr, &lines),
            }
        }
    }

    /// Writes the record of `finished` and counts it done.
    pub(super) fn record(
        &mut self,
        out: &OutputConfig,
        stderr: &mut dyn Write,
        finished: &Finished,
    ) {
        self.pending.finished(finished.target);
        write_record(out, stderr, self.lanes[finished.target].kind, finished);
    }

    /// Closes the queues and gives the workers until the deadline, one
    /// attempt timeout plus the budget's waits, to deliver what is queued;
    /// another signal ends the wait at once. Each finish moves `progress`
    /// on; whatever is still undelivered at the end is counted in one record
    /// per target, and holds the checkpoint at its minute.
    pub(super) async fn drain(
        mut self,
        out: &OutputConfig,
        stderr: &mut dyn Write,
        progress: &mut Progress,
    ) {
        for lane in &mut self.lanes {
            lane.queue = None;
        }
        let deadline = tokio::time::sleep(self.budget.attempt_timeout() + self.budget.total_wait());
        tokio::pin!(deadline);
        let another_signal = crate::cli::shutdown::shutdown_signal();
        tokio::pin!(another_signal);
        loop {
            tokio::select! {
                biased;
                finished = self.next_finished(stderr) => match finished {
                    Some(finished) => {
                        self.record(out, stderr, &finished);
                        progress.advance(self.oldest_pending(), out, stderr);
                    }
                    None => break,
                },
                () = &mut deadline => break,
                () = &mut another_signal => break,
            }
        }
        // Awaiting each aborted worker drops the attempt it was in, which
        // stops a program and everything it started before `listen` exits.
        for worker in &self.workers {
            worker.abort();
        }
        for worker in self.workers.drain(..) {
            let _ = worker.await;
        }
        while let Ok(report) = self.reports.try_recv() {
            if let Report::Output(lines) = report {
                write_output(stderr, &lines);
            }
        }
        for (index, lane) in self.lanes.iter().enumerate() {
            let undelivered = self.pending.undelivered(index);
            if undelivered > 0 {
                write_incomplete(out, stderr, lane.kind, undelivered as u64);
            }
        }
        let _ = stderr.flush();
    }
}

/// Whole lines a program wrote, passed on as written.
fn write_output(stderr: &mut dyn Write, lines: &[u8]) {
    let _ = stderr.write_all(lines);
    let _ = stderr.flush();
}

/// The Unix second an event arrived.
fn received_secs(event: &Event) -> u64 {
    event
        .received_at
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

fn plural(count: u64, one: &str, many: &str) -> String {
    if count == 1 {
        format!("{count} {one}")
    } else {
        format!("{count} {many}")
    }
}

/// The `forwarded` or `forward-failed` record for one delivery.
fn write_record(
    out: &OutputConfig,
    stderr: &mut dyn Write,
    kind: &'static str,
    finished: &Finished,
) {
    let event = &finished.event;
    let delivery = &finished.delivery;
    let received_secs = received_secs(event);
    let received_at = utc_rfc3339(received_secs);
    let attempts = plural(u64::from(delivery.attempts), "attempt", "attempts");
    let elapsed_ms = u64::try_from(delivery.elapsed.as_millis()).unwrap_or(u64::MAX);
    match &delivery.result {
        Ok(answer) => {
            let (key, value, said) = match answer {
                Answer::HttpStatus(status) => (
                    "http_status",
                    i64::from(*status),
                    format!("it answered {status}"),
                ),
                Answer::ExitStatus(code) => {
                    ("exit_status", i64::from(*code), format!("it exited {code}"))
                }
            };
            if out.format.is_structured() {
                let mut record = json!({
                    "status": "forwarded",
                    "target": kind,
                    "attempts": delivery.attempts,
                    "elapsed_ms": elapsed_ms,
                    "received_at": received_at,
                    "event_sha256": event.sha256(),
                });
                record[key] = json!(value);
                let _ = writeln!(stderr, "{record}");
            } else {
                out.info(
                    stderr,
                    &format!(
                        "Forwarded the event received at {received_at} to the {kind} target: \
                         {said} after {attempts}, in {elapsed_ms} ms."
                    ),
                );
            }
        }
        Err(cause) => {
            let next_step = NextStep::replay_window(received_secs);
            if out.format.is_structured() {
                let record = json!({
                    "status": "forward-failed",
                    "target": kind,
                    "attempts": delivery.attempts,
                    "cause": cause,
                    "received_at": received_at,
                    "event_sha256": event.sha256(),
                    "side_effects": FAILED_SIDE_EFFECTS,
                    "next_step": next_step,
                });
                let _ = writeln!(stderr, "{record}");
            } else {
                let replay = next_step.display_invocation().unwrap_or_default();
                out.warning(
                    stderr,
                    &format!(
                        "Could not forward the event received at {received_at} to the {kind} \
                         target after {attempts}: {cause}. It was printed, not delivered. To \
                         deliver it again, run: {replay}"
                    ),
                );
            }
        }
    }
}

/// The `forward-incomplete` record: events a target still held at shutdown.
fn write_incomplete(
    out: &OutputConfig,
    stderr: &mut dyn Write,
    kind: &'static str,
    undelivered: u64,
) {
    if out.format.is_structured() {
        let record = json!({
            "status": "forward-incomplete",
            "target": kind,
            "undelivered": undelivered,
        });
        let _ = writeln!(stderr, "{record}");
    } else {
        let events = plural(undelivered, "event", "events");
        out.warning(
            stderr,
            &format!(
                "Stopped with {events} printed and not delivered to the {kind} target. A replay \
                 of the minutes they arrived in delivers them again."
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use super::Pending;

    /// 15:04, 15:05, and 15:06 on 2026-10-22, UTC.
    fn minute(n: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(1_792_681_440 + 60 * n)
    }

    const URL: usize = 0;
    const PROGRAM: usize = 1;

    fn three_events_on_two_targets() -> Pending {
        let mut pending = Pending::new(2);
        for n in 0..3 {
            pending.queued(URL, minute(n));
            pending.queued(PROGRAM, minute(n));
        }
        pending
    }

    #[test]
    fn events_waiting_on_a_stalled_target_hold_the_checkpoint_at_the_oldest() {
        let mut pending = three_events_on_two_targets();
        for _ in 0..3 {
            pending.finished(URL);
        }

        assert_eq!(
            pending.oldest(),
            Some(minute(0)),
            "the program target, stalled, still holds the first event"
        );
        assert_eq!(pending.undelivered(PROGRAM), 3);
        assert_eq!(pending.undelivered(URL), 0);
    }

    #[test]
    fn a_finished_event_failed_or_delivered_stops_holding_the_checkpoint() {
        let mut pending = three_events_on_two_targets();
        for _ in 0..3 {
            pending.finished(URL);
        }
        pending.finished(PROGRAM);

        assert_eq!(pending.oldest(), Some(minute(1)));
    }

    #[test]
    fn with_nothing_pending_there_is_no_oldest() {
        let mut pending = three_events_on_two_targets();
        for _ in 0..3 {
            pending.finished(URL);
            pending.finished(PROGRAM);
        }

        assert_eq!(pending.oldest(), None);
    }
}
