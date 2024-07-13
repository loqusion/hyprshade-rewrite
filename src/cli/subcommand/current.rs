use std::process::ExitCode;

use crate::cli::CommandExecute;
use crate::hyprctl;
use clap::Parser;
use tracing::warn;

/**
Show the current shader
*/
#[derive(Debug, Parser)]
pub struct Current;

impl CommandExecute for Current {
    #[tracing::instrument(level = "debug")]
    fn execute(self) -> eyre::Result<ExitCode> {
        warn!("Implementation is incomlete");

        if let Some(shader_path) = hyprctl::shader::get()? {
            println!("{shader_path}")
        }

        Ok(ExitCode::SUCCESS)
    }
}
