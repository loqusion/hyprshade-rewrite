use std::process::ExitCode;

use crate::cli::CommandExecute;
use crate::hyprctl;
use clap::Parser;
use tracing::warn;

/**
Turn on a shader
*/
#[derive(Debug, Parser)]
pub struct On {
    /// Which shader to turn on
    ///
    /// May be a name (e.g. `blue-light-filter`)
    /// or a path (e.g. `~/.config/hypr/shaders/blue-light-filter.glsl`)
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
