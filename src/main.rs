mod api;
mod auth;
mod cli;
mod config;
mod error;
mod store;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();
    cli::commands::run(cli);
}
