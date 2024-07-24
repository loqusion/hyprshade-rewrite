mod common;
mod instrumentation;
mod subcommand;

use std::process::ExitCode;

use self::{instrumentation::Instrumentation, subcommand::HyprshadeSubcommand};
use clap::Parser;

pub trait CommandExecute {
    fn execute(self) -> eyre::Result<ExitCode>;
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[group(flatten)]
    pub instrumentation: Instrumentation,

    #[command(subcommand)]
    command: HyprshadeSubcommand,
}

impl CommandExecute for Cli {
    #[tracing::instrument(level = "trace", skip_all)]
    fn execute(self) -> eyre::Result<ExitCode> {
        match self.command {
            HyprshadeSubcommand::Auto(auto) => auto.execute(),
            HyprshadeSubcommand::Current(current) => current.execute(),
            HyprshadeSubcommand::Install(install) => install.execute(),
            HyprshadeSubcommand::Ls(ls) => ls.execute(),
            HyprshadeSubcommand::Off(off) => off.execute(),
            HyprshadeSubcommand::On(on) => on.execute(),
            HyprshadeSubcommand::Toggle(toggle) => toggle.execute(),
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
