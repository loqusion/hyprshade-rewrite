use std::process::ExitCode;

use super::CommandExecute;
use crate::hyprctl;
use clap::Parser;

/**
Turn off shader
*/
#[derive(Debug, Parser)]
pub struct Off;

impl CommandExecute for Off {
    fn execute(self) -> anyhow::Result<ExitCode> {
        hyprctl::shader::clear()?;
        Ok(ExitCode::SUCCESS)
    }
}
