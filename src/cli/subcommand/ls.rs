use std::process::ExitCode;

use crate::cli::CommandExecute;
use clap::Parser;
use color_eyre::eyre::eyre;

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
    fn execute(self) -> eyre::Result<ExitCode> {
        let Ls { long } = self;

        Err(eyre!("Not implemented"))
    }
}
