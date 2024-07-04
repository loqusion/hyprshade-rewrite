use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub fn run(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Subcommand, Debug)]
enum Command {}
