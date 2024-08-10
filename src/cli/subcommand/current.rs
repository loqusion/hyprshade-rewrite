use std::process::ExitCode;

use clap::Parser;
use tracing::warn;

use crate::{cli::CommandExecute, config::Config, shader::Shader};

/**
Show the current shader
*/
#[derive(Debug, Parser)]
pub struct Current {
    /// Show additional information
    #[arg(short, long)]
    long: bool,
}

impl CommandExecute for Current {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self, _config: Option<&Config>) -> eyre::Result<ExitCode> {
        let Self { long } = self;

        if let Some(shader) = Shader::current()? {
            if long {
                todo!("current --long")
            } else {
                println!("{}", shader.name());
            }
        }

        Ok(ExitCode::SUCCESS)
    }
}
