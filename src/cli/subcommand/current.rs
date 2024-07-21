use std::process::ExitCode;

use crate::{cli::CommandExecute, shader::Shader};
use clap::Parser;
use tracing::warn;

/**
Show the current shader
*/
#[derive(Debug, Parser)]
pub struct Current;

impl CommandExecute for Current {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self) -> eyre::Result<ExitCode> {
        if let Some(shader) = Shader::current()? {
            println!("{}", shader);
        }

        Ok(ExitCode::SUCCESS)
    }
}
