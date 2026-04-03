mod api;
mod auth;
mod cli;
mod config;
mod error;
mod output;
mod store;

use clap::{CommandFactory, Parser};
use cli::{Cli, Commands};
use error::{EXIT_GENERAL_ERROR, EXIT_SUCCESS, exit_code_for_error};
use output::OutputConfig;

fn main() {
    // Restore default SIGPIPE handling (Rust masks it, causing panics on closed pipes)
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let cli = Cli::parse();

    // --- Tier 1: Meta-commands (need only parsed args) ---
    if let Some(ref cmd) = cli.command {
        match cmd {
            Commands::Completions { shell } => {
                let mut cmd = Cli::command();
                clap_complete::generate(*shell, &mut cmd, "xr", &mut std::io::stdout());
                return;
            }
            Commands::Version => {
                println!("xr {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            Commands::Schema { command, list, all } => {
                let out = OutputConfig::new(cli.output.clone(), cli.quiet);
                match cli::commands::schema::run_schema(command.as_deref(), *list, *all) {
                    Ok(()) => return,
                    Err(e) => {
                        out.print_error(&e, EXIT_GENERAL_ERROR);
                        std::process::exit(EXIT_GENERAL_ERROR);
                    }
                }
            }
            _ => {}
        }
    }

    // --- Tier 3: Everything else (needs config + auth) ---
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
