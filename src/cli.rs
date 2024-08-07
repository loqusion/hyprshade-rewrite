mod arg {
    pub(crate) mod help;
    pub(crate) mod var;
}
mod instrumentation;
mod subcommand;

use std::{path::PathBuf, process::ExitCode};

use self::{instrumentation::Instrumentation, subcommand::HyprshadeSubcommand};
use crate::{
    config::{Config, ConfigReadError},
    constants::{HYPRLAND_CONFIG_DIR, HYPRSHADE_CONFIG_DIR, HYPRSHADE_CONFIG_FILE_ENV},
};
use clap::Parser;

pub trait CommandExecute {
    fn execute(self, config: Option<&Config>) -> eyre::Result<ExitCode>;
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[group(flatten)]
    pub instrumentation: Instrumentation,

    /// Path to configuration file
    #[arg(long, env = HYPRSHADE_CONFIG_FILE_ENV, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: HyprshadeSubcommand,
}

impl Cli {
    pub fn config(&self) -> Result<Option<Config>, ConfigReadError> {
        if let Some(path) = &self.config {
            return Some(Config::read(path)).transpose();
        }

        for path in &[
            HYPRLAND_CONFIG_DIR.to_owned().join("hyprshade.toml"),
            HYPRSHADE_CONFIG_DIR.to_owned().join("config.toml"),
        ] {
            match Config::read(path) {
                Ok(config) => return Ok(Some(config)),
                Err(ConfigReadError::Io { .. }) => continue,
                Err(err @ ConfigReadError::Parse { .. }) => return Err(err),
            }
        }

        Ok(None)
    }
}

impl CommandExecute for Cli {
    #[tracing::instrument(level = "trace", skip_all)]
    fn execute(self, config: Option<&Config>) -> eyre::Result<ExitCode> {
        match self.command {
            HyprshadeSubcommand::Auto(auto) => auto.execute(config),
            HyprshadeSubcommand::Current(current) => current.execute(config),
            HyprshadeSubcommand::Install(install) => install.execute(config),
            HyprshadeSubcommand::Ls(ls) => ls.execute(config),
            HyprshadeSubcommand::Off(off) => off.execute(config),
            HyprshadeSubcommand::On(on) => on.execute(config),
            HyprshadeSubcommand::Toggle(toggle) => toggle.execute(config),
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn debug_assert() {
        Cli::command().debug_assert();
    }
}
