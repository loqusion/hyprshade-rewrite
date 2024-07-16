use std::process::ExitCode;

use crate::cli::{common::SHADER_HELP, CommandExecute};
use crate::hyprctl;
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

        warn!("Implementation is incomlete");

        hyprctl::shader::set(&shader)?;

        Ok(ExitCode::SUCCESS)
    }
}
