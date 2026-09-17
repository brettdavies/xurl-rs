//! Shared by the examples: credentials come from the environment, and a
//! failure exits with the code the crate maps the error to.

// `xdk::Error` is wide (its mismatch variant carries several strings), which
// the crate allows for itself; an example returning `xdk::Result` inherits it.
#![allow(clippy::result_large_err)]

use xdk::Error;

/// Reads `name`, or fails as a missing credential so the process exits with
/// the auth-required code.
pub fn required_env(name: &str) -> xdk::Result<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            Error::auth(format!(
                "{name} is not set; export it before running this example"
            ))
        })
}

/// Prints the error the way a CLI built on the crate would, then exits with
/// its code.
pub fn exit_with(err: Error) -> ! {
    eprintln!("{}: {err}", err.kind());
    if let Some(action) = err.next_action() {
        eprintln!("next: {action:?}");
    }
    if let Some(url) = err.docs_url() {
        eprintln!("see {url}");
    }
    std::process::exit(err.exit_code())
}
