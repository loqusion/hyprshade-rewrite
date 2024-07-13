use std::process::ExitCode;

use crate::cli::CommandExecute;
use crate::hyprctl;
use clap::Parser;

/**
Turn off shader
*/
#[derive(Debug, Parser)]
pub struct Off;

impl CommandExecute for Off {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self) -> eyre::Result<ExitCode> {
        hyprctl::shader::clear()?;
        Ok(ExitCode::SUCCESS)
    }
}
