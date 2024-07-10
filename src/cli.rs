mod current;
mod off;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            Command::Current => current::run(),
            Command::Off => off::run(),
        }
    }
}

#[derive(Subcommand, Debug)]
enum Command {
    Current,
    Off,
}
