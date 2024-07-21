use std::process::ExitCode;

use crate::{
    cli::{common::SHADER_HELP, CommandExecute},
    shader::Shader,
};
use clap::Parser;
use tracing::warn;

/**
Turn on a shader
*/
#[derive(Debug, Parser)]
pub struct On {
    #[arg(help = SHADER_HELP)]
    shader: String,
}

impl CommandExecute for On {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self) -> eyre::Result<ExitCode> {
        let On { shader } = self;

        Shader::from_cli_arg(&shader).on()?;

        Ok(ExitCode::SUCCESS)
    }
}
