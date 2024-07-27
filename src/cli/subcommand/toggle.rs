use std::process::ExitCode;

use crate::{
    cli::{
        common::{arg_vars_to_data, ArgVar, SHADER_HELP, SHADER_HELP_LONG},
        CommandExecute,
    },
    resolver::Resolver,
    shader::{OnOrOff, Shader},
};
use clap::Parser;
use color_eyre::{eyre::eyre, Section};

/**
TODO: write help text
*/
#[derive(Debug, Parser)]
pub struct Toggle {
    #[arg(help = SHADER_HELP, long_help = SHADER_HELP_LONG)]
    shader: Option<String>,

    /// Configuration variable used in rendering <SHADER> (may be specified multiple times)
    #[arg(long)]
    var: Vec<ArgVar>,

    #[arg(long, group = "fallback_args")]
    fallback: Option<String>,

    #[arg(long, group = "fallback_args")]
    fallback_default: bool,

    #[arg(long, group = "fallback_args")]
    fallback_auto: bool,

    /// Configuration variable used in rendering fallback shader (may be specified multiple times)
    ///
    /// Applies to `--fallback`, `--fallback-default`, and `--fallback-auto`
    #[arg(long, requires = "fallback_args")]
    var_fallback: Vec<ArgVar>,
}

impl CommandExecute for Toggle {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self) -> eyre::Result<ExitCode> {
        let Toggle {
            shader,
            var,
            fallback,
            fallback_default,
            fallback_auto,
            var_fallback,
        } = self;

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
            let data = arg_vars_to_data(var_fallback, "var-fallback")?;
            fallback.on_or_off(&data)?;
        } else {
            let data = arg_vars_to_data(var, "var")?;
            shader.on_or_off(&data)?;
        }

        Ok(ExitCode::SUCCESS)
    }
}
