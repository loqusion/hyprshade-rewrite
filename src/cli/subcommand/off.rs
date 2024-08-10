use std::process::ExitCode;

use clap::Parser;

use crate::{cli::CommandExecute, config::Config, shader::Shader};

/**
Turn off shader
*/
#[derive(Debug, Parser)]
pub struct Off;

impl CommandExecute for Off {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self, _config: Option<&Config>) -> eyre::Result<ExitCode> {
        Shader::off()?;

        Ok(ExitCode::SUCCESS)
    }
}
