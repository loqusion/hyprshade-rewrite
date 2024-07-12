use std::process::ExitCode;

use crate::cli::CommandExecute;
use crate::hyprctl;
use clap::Parser;

/**
Show the current shader
*/
#[derive(Debug, Parser)]
pub struct Current;

impl CommandExecute for Current {
    fn execute(self) -> eyre::Result<ExitCode> {
        eprintln!("Implementation is incomlete");

        if let Some(shader_path) = hyprctl::shader::get()? {
            println!("{shader_path}")
        }

        Ok(ExitCode::SUCCESS)
    }
}
