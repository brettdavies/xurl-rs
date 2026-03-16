mod api;
mod auth;
mod cli;
mod config;
mod error;
mod output;
mod store;

use clap::{CommandFactory, Parser};
use cli::Cli;
use cli::exit_codes::{
    EXIT_AUTH_REQUIRED, EXIT_GENERAL_ERROR, EXIT_NETWORK_ERROR, EXIT_NOT_FOUND, EXIT_RATE_LIMITED,
    EXIT_SUCCESS,
};
use error::XurlError;
use output::OutputConfig;

fn main() {
    let cli = Cli::parse();

    // Handle --generate-completion before anything else
    if let Some(shell) = cli.generate_completion {
        let mut cmd = Cli::command();
        clap_complete::generate(shell, &mut cmd, "xurl", &mut std::io::stdout());
        return;
    }

    let out = OutputConfig::new(cli.output.clone(), cli.quiet);

    match cli::commands::run(cli, &out) {
        Ok(()) => std::process::exit(EXIT_SUCCESS),
        Err(e) => {
            let code = exit_code_for_error(&e);
            out.print_error(&e, code);
            std::process::exit(code);
        }
    }
}

/// Maps error variants to structured exit codes.
fn exit_code_for_error(e: &XurlError) -> i32 {
    match e {
        XurlError::Auth(_) | XurlError::TokenStore(_) => EXIT_AUTH_REQUIRED,
        XurlError::Http(msg) if msg.contains("429") => EXIT_RATE_LIMITED,
        XurlError::Api(msg) if msg.contains("429") => EXIT_RATE_LIMITED,
        XurlError::Http(msg) if msg.contains("404") => EXIT_NOT_FOUND,
        XurlError::Api(msg) if msg.contains("404") => EXIT_NOT_FOUND,
        XurlError::Io(_) => EXIT_NETWORK_ERROR,
        _ => EXIT_GENERAL_ERROR,
    }
}
