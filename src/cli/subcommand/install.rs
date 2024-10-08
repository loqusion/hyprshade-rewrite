use std::process::ExitCode;

use clap::Parser;
use color_eyre::eyre::eyre;

use crate::{cli::CommandExecute, config::Config};

/**
TODO: write help text
*/
#[derive(Debug, Parser)]
pub struct Install {
    /// TODO: write help text
    #[arg(long)]
    enable: bool,
}

impl CommandExecute for Install {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self, _config: Option<&Config>) -> eyre::Result<ExitCode> {
        let Install { enable: _ } = self;

        Err(eyre!("Not implemented"))
    }
}
