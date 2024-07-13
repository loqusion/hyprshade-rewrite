use std::process::ExitCode;

use crate::cli::CommandExecute;
use clap::Parser;
use color_eyre::eyre::eyre;

/**
TODO: write help text
*/
#[derive(Debug, Parser)]
pub struct Toggle {
    /// Which shader to turn on
    ///
    /// May be a name (e.g. `blue-light-filter`)
    /// or a path (e.g. `~/.config/hypr/shaders/blue-light-filter.glsl`)
    shader: Option<String>,
}

impl CommandExecute for Toggle {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self) -> eyre::Result<ExitCode> {
        let Toggle { shader: _ } = self;

        Err(eyre!("Not implemented"))
    }
}
