use std::process::ExitCode;

use crate::cli::CommandExecute;
use clap::Parser;
use color_eyre::eyre::eyre;

/**
TODO: write help text
*/
#[derive(Debug, Parser)]
pub struct Auto;

impl CommandExecute for Auto {
    fn execute(self) -> eyre::Result<ExitCode> {
        Err(eyre!("Not implemented"))
    }
}
