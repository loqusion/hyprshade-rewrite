use std::process::ExitCode;

use super::CommandExecute;
use anyhow::anyhow;
use clap::Parser;

/**
TODO: write help text
*/
#[derive(Debug, Parser)]
pub struct Auto;

impl CommandExecute for Auto {
    fn execute(self) -> anyhow::Result<ExitCode> {
        Err(anyhow!("Not implemented"))
    }
}
