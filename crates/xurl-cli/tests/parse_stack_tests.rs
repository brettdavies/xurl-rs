//! The argument parser builds clap's whole command tree, and a debug build
//! keeps every builder temporary in its own stack slot: parsing took about
//! 1.8 MiB of stack in a debug build once the command families landed, and
//! about 295 KiB in a release build. The in-process tests parse on libtest's
//! threads, so `.cargo/config.toml` sets `RUST_MIN_STACK` for them; this file
//! fails, alone, when the parser outgrows the budget below, before an
//! unrelated test overflows its thread.

use clap::Parser;

/// The stack the parser may use in a debug build: half of the 8 MiB that
/// `.cargo/config.toml` gives every test thread.
const PARSE_STACK_BUDGET: usize = 4 * 1024 * 1024;

/// Clap builds the whole tree whichever command is named, so one
/// invocation measures them all. A stack overflow aborts the process rather
/// than failing the test, so the thread's name is what the abort reports.
#[test]
fn the_parser_fits_its_stack_budget() {
    let parsed = std::thread::Builder::new()
        .name("parse-within-4-MiB-of-stack".to_string())
        .stack_size(PARSE_STACK_BUDGET)
        .spawn(|| xurl::cli::Cli::try_parse_from(["xr", "whoami"]).is_ok())
        .expect("spawn the parse thread")
        .join()
        .expect("the parse thread finished");
    assert!(parsed, "the sample invocation no longer parses");
}

/// libtest spawns each test thread at `RUST_MIN_STACK`, which
/// `.cargo/config.toml` sets; a run that bypasses that file gets 2 MiB,
/// close to what the parser takes, and in-process tests overflow at random.
#[test]
fn test_threads_get_the_configured_stack() {
    assert_eq!(
        std::env::var("RUST_MIN_STACK").as_deref(),
        Ok("8388608"),
        "the test run did not read .cargo/config.toml's [env]; run the tests through cargo \
         from the workspace, or export RUST_MIN_STACK=8388608"
    );
}
