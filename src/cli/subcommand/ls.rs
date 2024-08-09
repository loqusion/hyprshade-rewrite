use std::process::ExitCode;

use crate::{cli::CommandExecute, config::Config};
use clap::Parser;
use color_eyre::eyre::eyre;

/**
TODO: write help text
*/
#[derive(Debug, Parser)]
pub struct Ls {
    /// TODO: write help text
    #[arg(short, long)]
    long: bool,
}

impl CommandExecute for Ls {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self, _config: Option<&Config>) -> eyre::Result<ExitCode> {
        let Ls { long: _ } = self;

        Err(eyre!("Not implemented"))
    }
}
