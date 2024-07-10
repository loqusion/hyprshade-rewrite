mod cli;
mod hyprctl;

use std::process::ExitCode;

use clap::Parser;
use cli::{Cli, CommandExecute};

fn main() -> anyhow::Result<ExitCode> {
    let cli = Cli::parse();

    cli.execute()
}
