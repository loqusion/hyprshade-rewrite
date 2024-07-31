use std::process::ExitCode;

use crate::{cli::CommandExecute, config::Config, shader::Shader};
use clap::Parser;
use tracing::warn;

/**
Show the current shader
*/
#[derive(Debug, Parser)]
pub struct Current;

impl CommandExecute for Current {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self, _config: Option<&Config>) -> eyre::Result<ExitCode> {
        if let Some(shader) = Shader::current()? {
            dbg!(&shader);
            println!("{}", shader);
        }

        Ok(ExitCode::SUCCESS)
    }
}
