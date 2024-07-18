use std::process::ExitCode;

use crate::{
    cli::{common::SHADER_HELP, CommandExecute},
    hyprctl,
    resolver::Resolver,
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

        warn!("Implementation is incomlete");

        let shader_path = Resolver::new(&shader).resolve()?;
        hyprctl::shader::set(&shader_path)?;

        Ok(ExitCode::SUCCESS)
    }
}
