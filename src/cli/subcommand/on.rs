use std::process::ExitCode;

use crate::{
    cli::{
        common::{SHADER_HELP, SHADER_HELP_LONG},
        CommandExecute,
    },
    resolver::Resolver,
};
use clap::Parser;
use tracing::warn;

/**
Turn on a shader
*/
#[derive(Debug, Parser)]
pub struct On {
    #[arg(help = SHADER_HELP, long_help = SHADER_HELP_LONG)]
    shader: String,
}

impl CommandExecute for On {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self) -> eyre::Result<ExitCode> {
        let On { shader } = self;

        let shader = Resolver::with_cli_arg(&shader).resolve()?;
        shader.on()?;

        Ok(ExitCode::SUCCESS)
    }
}
