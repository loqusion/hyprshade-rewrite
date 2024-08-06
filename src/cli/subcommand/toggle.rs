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
    constants::README_CONFIGURATION,
    resolver::Resolver,
    schedule::Schedule,
    shader::Shader,
    template::MergeDeep,
};
use clap::Parser;
use color_eyre::{owo_colors::OwoColorize, Section, SectionExt};
use eyre::{Context, OptionExt};

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

        let now = chrono::Local::now();

        // Eagerly evaluate --var and --var-fallback so that feedback is presented unconditionally
        let fallback_data = VarArg::merge_into_data(var_fallback, "var-fallback")?;
        let shader_data = VarArg::merge_into_data(var, "var")?;

        let fallback = match (&fallback, fallback_default, fallback_auto) {
            (None, false, false) => None,
            (Some(fallback), false, false) => Some(Resolver::with_cli_arg(fallback).resolve()?),
            (None, true, false) => {
                let name = config
                    .ok_or_eyre("--fallback-default requires a config")
                    .and_then(|c| {
                        c.default_shader()
                            .ok_or_eyre("no default shader found in config")
                    })
                    .with_suggestion(|| format!("For more information, see {README_CONFIGURATION}"))
                    .map(|s| &s.name)?;
                Some(Resolver::with_name(name).resolve()?)
            }
            (None, false, true) => {
                config
                    .ok_or_eyre("no configuration file found")
                    .warning("A configuration file is required to use --fallback-auto")
                    .and_then(|c| {
                        if shader.is_some() {
                            Schedule::with_config(c).scheduled_shader(&now.time())
                                .wrap_err("error resolving shader in config")
                                .with_section(|| c.path().display().yellow().to_string().header("Configuration"))
                                .note("--fallback-auto tried to use the currently scheduled shader")
                                .suggestion("Change the shader name in your configuration, or make sure a shader by that name exists")
                        } else {
                            c.default_shader()
                                .ok_or_eyre("no default shader found in config")
                                .with_section(|| c.path().display().yellow().to_string().header("Configuration"))
                                .note("--fallback-auto tried to use the default shader because you didn't specify SHADER")
                                .suggestion("Define a default shader (default = true), or specify SHADER in cli arguments")
                                .and_then(|s| {
                                    Some(Resolver::with_name(&s.name).resolve())
                                        .transpose()
                                        .wrap_err("error resolving shader in config")
                                        .with_section(|| c.path().display().yellow().to_string().header("Configuration"))
                                        .note("--fallback-auto tried to use the default shader because you didn't specify SHADER")
                                        .suggestion("Change the shader name in your configuration, or make sure a shader by that name exists")
                                })
                        }
                    })
                    .with_suggestion(|| {
                        format!("For more information, see {README_CONFIGURATION}")
                    })?
            }
            _ => {
                unreachable!(
                    "--fallback, --fallback-default, and --fallback-auto are mutually exclusive"
                )
            }
        };

        let shader = match &shader {
            Some(shader) => Some(Resolver::with_cli_arg(shader).resolve()?),
            None => config
                .ok_or_eyre("no configuration file found")
                .warning("A configuration file is required to call this command without SHADER")
                .and_then(|c| {
                    Schedule::with_config(c).scheduled_shader(&now.time())
                        .wrap_err("error resolving shader in config")
                        .with_section(|| c.path().display().yellow().to_string().header("Configuration"))
                        .note("If you omit SHADER from cli arguments, it will be inferred from the schedule in your configuration")
                        .suggestion("Change the shader name in your configuration, or make sure a shader by that name exists")
                })
                .with_suggestion(|| format!("For more information, see {README_CONFIGURATION}"))?,
        };

        let current_shader = Shader::current()?;

        let (designated_shader, designated_data) = if shader == current_shader {
            (fallback, fallback_data)
        } else {
            (shader, shader_data)
        };

        if let Some(designated_shader) = designated_shader {
            let mut designated_data = designated_data;
            if let Some(config_data) = config.and_then(|c| c.data(designated_shader.name())) {
                designated_data.merge_deep_keep(config_data.clone());
            }
            let designated_data = designated_data;
            designated_shader.on(&designated_data)?;
        } else {
            Shader::off()?;
        }

        Ok(ExitCode::SUCCESS)
    }
}
