mod subcommand;

use std::process::ExitCode;

use self::subcommand::HyprshadeSubcommand;
use clap::Parser;

pub trait CommandExecute {
    fn execute(self) -> eyre::Result<ExitCode>;
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: HyprshadeSubcommand,
}

impl CommandExecute for Cli {
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
