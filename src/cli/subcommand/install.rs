use std::process::ExitCode;

use crate::cli::CommandExecute;
use clap::Parser;
use color_eyre::eyre::eyre;

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
    fn execute(self) -> eyre::Result<ExitCode> {
        let Install { enable } = self;

        Err(eyre!("Not implemented"))
    }
}
