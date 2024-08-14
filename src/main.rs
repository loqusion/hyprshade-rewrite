mod builtin;
mod cli;
mod config;
mod constants;
mod dirs;
mod hyprctl;
mod resolver;
mod schedule;
mod shader;
mod template;
mod time;
mod util;

use std::process::ExitCode;

use clap::Parser;

use crate::cli::{Cli, CommandExecute};

fn main() -> eyre::Result<ExitCode> {
    color_eyre::config::HookBuilder::default()
        .display_env_section(cfg!(debug_assertions))
        .install()?;

    let cli = Cli::parse();

    cli.instrumentation.setup()?;

    let config = cli.config()?;

    match cli.execute(config.as_ref()) {
        Ok(exit_code) => Ok(exit_code),
        Err(err) => match err.downcast_ref::<clap::Error>() {
            None => Err(err),
            Some(clap_err) => {
                clap_err.exit();
            }
        },
    }
}

#[cfg(not(target_os = "linux"))]
compile_error!("target os must be linux");
