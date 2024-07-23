use std::process::ExitCode;

use crate::{
    cli::{common::SHADER_HELP, CommandExecute},
    resolver::Resolver,
    shader::{OnOrOff, Shader},
};
use clap::Parser;
use color_eyre::eyre::eyre;

/**
TODO: write help text
*/
#[derive(Debug, Parser)]
pub struct Toggle {
    #[arg(help = SHADER_HELP)]
    shader: Option<String>,
    #[arg(long)]
    fallback: Option<String>,
    #[arg(long)]
    fallback_default: bool,
    #[arg(long)]
    fallback_auto: bool,
}

impl CommandExecute for Toggle {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self) -> eyre::Result<ExitCode> {
        let Toggle {
            shader,
            fallback,
            fallback_default,
            fallback_auto,
        } = self;

        let fallback = match (&fallback, fallback_default, fallback_auto) {
            (None, false, false) => None,
            (Some(fallback), false, false) => Some(Resolver::from_cli_arg(fallback).resolve()?),
            (None, true, false) => todo!("getting default shader from config"),
            (None, false, true) => todo!("getting scheduled shader from config"),
            _ => {
                return Err(eyre!(
                    "only one of --fallback, --fallback-default, or --fallback-auto can be used"
                ))
            }
        };

        let shader = match &shader {
            Some(shader) => Some(Resolver::from_cli_arg(shader).resolve()?),
            None => todo!("shader inference from config"),
        };

        let current_shader = Shader::current()?;

        if shader == current_shader {
            fallback.on_or_off()?;
        } else {
            shader.on_or_off()?;
        }

        Ok(ExitCode::SUCCESS)
    }
}
