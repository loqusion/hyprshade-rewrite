use std::process::ExitCode;

use super::CommandExecute;
use anyhow::anyhow;
use clap::Parser;

/**
TODO: write help text
*/
#[derive(Debug, Parser)]
pub struct Install {
    /// TODO: write help text
    #[arg(long)]
    enable: bool,
}

impl CommandExecute for Install {
    fn execute(self) -> anyhow::Result<ExitCode> {
        let Install { enable } = self;

        Err(anyhow!("Not implemented"))
    }
}
