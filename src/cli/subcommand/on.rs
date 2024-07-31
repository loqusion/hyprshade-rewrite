use std::process::ExitCode;

use crate::{
    cli::{
        arg::{
            help::{SHADER_HELP, SHADER_HELP_LONG},
            var::{VarArg, VarArgParser},
        },
        CommandExecute,
    },
    config::Config,
    resolver::Resolver,
    template::MergeDeep,
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
    #[arg(long, value_name = "KEY=VALUE", value_parser = VarArgParser)]
    var: Vec<VarArg>,
}

impl CommandExecute for On {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self, config: Option<&Config>) -> eyre::Result<ExitCode> {
        let On { shader, var } = self;

        let data = VarArg::merge_into_data(var, "var")?;
        let shader = Resolver::with_cli_arg(&shader).resolve()?;

        let mut data = data;
        if let Some(config_data) = config.and_then(|c| c.data(shader.name())) {
            data.merge_deep_keep(config_data.clone());
        }
        let data = data;

        shader.on(&data)?;

        Ok(ExitCode::SUCCESS)
    }
}
