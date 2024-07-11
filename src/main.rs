mod cli;
mod hyprctl;

use std::process::ExitCode;

use clap::Parser;
use cli::{Cli, CommandExecute};

fn main() -> eyre::Result<ExitCode> {
    color_eyre::install()?;

    let cli = Cli::parse();

    cli.execute()
}
