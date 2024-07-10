mod current;
use current::Current;
mod off;
use off::Off;
mod on;
use on::On;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

pub trait CommandExecute {
    fn execute(self) -> anyhow::Result<ExitCode>;
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

impl CommandExecute for Cli {
    fn execute(self) -> anyhow::Result<ExitCode> {
        match self.command {
            Command::Current(current) => current.execute(),
            Command::Off(off) => off.execute(),
            Command::On(on) => on.execute(),
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    Current(Current),
    Off(Off),
    On(On),
}
