mod auto;
use auto::Auto;
mod current;
use current::Current;
mod install;
use install::Install;
mod ls;
use ls::Ls;
mod off;
use off::Off;
mod on;
use on::On;
mod toggle;
use toggle::Toggle;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

pub trait CommandExecute {
    fn execute(self) -> eyre::Result<ExitCode>;
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

impl CommandExecute for Cli {
    fn execute(self) -> eyre::Result<ExitCode> {
        match self.command {
            Command::Auto(auto) => auto.execute(),
            Command::Current(current) => current.execute(),
            Command::Install(install) => install.execute(),
            Command::Ls(ls) => ls.execute(),
            Command::Off(off) => off.execute(),
            Command::On(on) => on.execute(),
            Command::Toggle(toggle) => toggle.execute(),
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    Auto(Auto),
    Current(Current),
    Install(Install),
    Ls(Ls),
    Off(Off),
    On(On),
    Toggle(Toggle),
}
