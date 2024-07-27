use std::process::ExitCode;

use crate::{
    cli::{
        common::{arg_vars_to_data, ArgVar, SHADER_HELP, SHADER_HELP_LONG},
        CommandExecute,
    },
    resolver::Resolver,
};
use clap::Parser;
use tracing::warn;

/**
Turn on a shader
*/
#[derive(Debug, Parser)]
pub struct On {
    #[arg(help = SHADER_HELP, long_help = SHADER_HELP_LONG)]
    shader: String,

    /// Configuration variable used in rendering <SHADER> (may be specified multiple times)
    #[arg(long, value_name = "KEY=VALUE")]
    var: Vec<ArgVar>,
}

impl CommandExecute for On {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self) -> eyre::Result<ExitCode> {
        let On { shader, var } = self;

        let data = arg_vars_to_data(var, "var")?;
        let shader = Resolver::with_cli_arg(&shader).resolve()?;
        shader.on(&data)?;

        Ok(ExitCode::SUCCESS)
    }
}
