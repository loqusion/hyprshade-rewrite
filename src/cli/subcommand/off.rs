use std::process::ExitCode;

use crate::{cli::CommandExecute, shader::Shader};
use clap::Parser;

/**
Turn off shader
*/
#[derive(Debug, Parser)]
pub struct Off;

impl CommandExecute for Off {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self) -> eyre::Result<ExitCode> {
        Shader::off()?;

        Ok(ExitCode::SUCCESS)
    }
}
