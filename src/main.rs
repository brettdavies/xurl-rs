mod api;
mod auth;
mod cli;
mod config;
mod error;
mod store;

use clap::{CommandFactory, Parser};
use cli::Cli;

fn main() {
    let cli = Cli::parse();

    // Handle --generate-completion before anything else
    if let Some(shell) = cli.generate_completion {
        let mut cmd = Cli::command();
        clap_complete::generate(shell, &mut cmd, "xurl", &mut std::io::stdout());
        return;
    }

    cli::commands::run(cli);
}
