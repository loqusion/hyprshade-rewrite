use std::process::ExitCode;

use super::CommandExecute;
use crate::hyprctl;
use clap::Parser;

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
    fn execute(self) -> anyhow::Result<ExitCode> {
        let On { shader } = self;

        eprintln!("Implementation is incomlete");

        hyprctl::shader::set(&shader)?;

        Ok(ExitCode::SUCCESS)
    }
}
