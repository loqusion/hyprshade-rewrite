use std::process::ExitCode;

use crate::cli::{common::SHADER_HELP, CommandExecute};
use clap::Parser;
use color_eyre::eyre::eyre;

/**
TODO: write help text
*/
#[derive(Debug, Parser)]
pub struct Toggle {
    #[arg(help = SHADER_HELP)]
    shader: Option<String>,
}

impl CommandExecute for Toggle {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self) -> eyre::Result<ExitCode> {
        let Toggle { shader: _ } = self;

        Err(eyre!("Not implemented"))
    }
}
