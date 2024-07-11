use std::process::ExitCode;

use super::CommandExecute;
use anyhow::anyhow;
use clap::Parser;

/**
TODO: write help text
*/
#[derive(Debug, Parser)]
pub struct Ls {
    /// TODO: write help text
    #[arg(short, long)]
    long: bool,
}

impl CommandExecute for Ls {
    fn execute(self) -> anyhow::Result<ExitCode> {
        let Ls { long } = self;

        Err(anyhow!("Not implemented"))
    }
}
