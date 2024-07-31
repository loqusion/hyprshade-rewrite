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
    shader::{OnOrOff, Shader},
    template::MergeDeep,
};
use clap::Parser;
use color_eyre::eyre::eyre;

/**
TODO: write help text
*/
#[derive(Debug, Parser)]
pub struct Toggle {
    #[arg(help = SHADER_HELP, long_help = SHADER_HELP_LONG)]
    shader: Option<String>,

    /// Configuration variable used in rendering <SHADER> (may be specified multiple times)
    #[arg(long, value_name = "KEY=VALUE", value_parser = VarArgParser)]
    var: Vec<VarArg>,

    #[arg(long, group = "fallback_args")]
    fallback: Option<String>,

    #[arg(long, group = "fallback_args")]
    fallback_default: bool,

    #[arg(long, group = "fallback_args")]
    fallback_auto: bool,

    /// Configuration variable used in rendering fallback shader (may be specified multiple times)
    ///
    /// Applies to `--fallback`, `--fallback-default`, and `--fallback-auto`
    #[arg(long, value_name = "KEY=VALUE", value_parser = VarArgParser, requires = "fallback_args")]
    var_fallback: Vec<VarArg>,
}

impl CommandExecute for Toggle {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self, config: Option<&Config>) -> eyre::Result<ExitCode> {
        let Toggle {
            shader,
            var,
            fallback,
            fallback_default,
            fallback_auto,
            var_fallback,
        } = self;

        // Eagerly evaluate --var and --var-fallback so that feedback is presented unconditionally
        let fallback_data = VarArg::merge_into_data(var_fallback, "var-fallback")?;
        let data = VarArg::merge_into_data(var, "var")?;

        let fallback = match (&fallback, fallback_default, fallback_auto) {
            (None, false, false) => None,
            (Some(fallback), false, false) => Some(Resolver::with_cli_arg(fallback).resolve()?),
            (None, true, false) => todo!("getting default shader from config"),
            (None, false, true) => todo!("getting scheduled shader from config"),
            _ => {
                return Err(eyre!(
                    "only one of --fallback, --fallback-default, or --fallback-auto can be used"
                ))
            }
        };

        let shader = match &shader {
            Some(shader) => Some(Resolver::with_cli_arg(shader).resolve()?),
            None => todo!("shader inference from config"),
        };

        let current_shader = Shader::current()?;

        if shader == current_shader {
            let fallback_data = if let Some(config_data) =
                config.and_then(|c| fallback.as_ref().and_then(|s| c.data(s.name())))
            {
                let mut fallback_data = fallback_data;
                fallback_data.merge_deep_keep(config_data.clone());
                fallback_data
            } else {
                fallback_data
            };
            fallback.on_or_off(&fallback_data)?;
        } else {
            let data = if let Some(config_data) =
                config.and_then(|c| shader.as_ref().and_then(|s| c.data(s.name())))
            {
                let mut data = data;
                data.merge_deep_keep(config_data.clone());
                data
            } else {
                data
            };
            shader.on_or_off(&data)?;
        }

        Ok(ExitCode::SUCCESS)
    }
}
