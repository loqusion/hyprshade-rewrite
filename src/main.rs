mod builtin;
mod cli;
mod config;
mod dirs;
mod hyprctl;
mod resolver;
mod shader;
mod util;

use std::process::ExitCode;

use clap::Parser;
use cli::{Cli, CommandExecute};

fn main() -> eyre::Result<ExitCode> {
    color_eyre::config::HookBuilder::default()
        .display_env_section(cfg!(debug_assertions))
        .install()?;

    let cli = Cli::parse();

    cli.instrumentation.setup()?;

    cli.execute()
}

#[cfg(not(target_os = "linux"))]
compile_error!("target os must be linux");
