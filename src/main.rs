mod cli;
mod hyprctl;

use std::process::ExitCode;

use clap::Parser;
use cli::{Cli, CommandExecute};

fn main() -> eyre::Result<ExitCode> {
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .install()?;

    let cli = Cli::parse();

    cli.execute()
}
