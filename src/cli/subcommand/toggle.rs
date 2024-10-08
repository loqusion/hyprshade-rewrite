use std::process::ExitCode;

use clap::Parser;
use color_eyre::Section;
use const_format::{concatcp, formatcp};
use eyre::{Context, OptionExt};

use crate::{
    cli::{
        arg::{
            help::{SHADER_HELP, SHADER_HELP_LONG as SHADER_HELP_LONG_SOURCE},
            var::{MergeVarArg, VarArg, VarArgParser},
        },
        CommandExecute,
    },
    config::Config,
    constants::README_CONFIGURATION,
    resolver::Resolver,
    schedule::Schedule,
    shader::Shader,
    template::MergeDeep,
    time::now,
    util::ConfigSection,
};

const EXAMPLE_SECTION: &str = color_print::cstr!(
    r#"<bold><underline>Examples:</underline></bold>
  # toggle between blue-light-filter and off
  hyprshade toggle blue-light-filter

  # toggle between scheduled shader and off
  hyprshade toggle

  # toggle between blue-light-filter and automatically inferred shader
  hyprshade toggle blue-light-filter --fallback-auto
"#
);
const NOTE_SECTION: &str = color_print::cstr!(
    r#"<bold><underline>Note:</underline></bold>
  This subcommand is mostly intended for use as a keybind. `hyprshade toggle <<SHADER>> --fallback-auto` probably does what you want.
"#
);
const AFTER_HELP: &str = NOTE_SECTION;
const AFTER_LONG_HELP: &str = concatcp!(EXAMPLE_SECTION, "\n", NOTE_SECTION);

const SHADER_HELP_LONG: &str = formatcp!(
    "{}\n\
    \n\
    If omitted, will be inferred from configuration schedule\
",
    SHADER_HELP_LONG_SOURCE
);

/**
Toggle between different shaders

Specifically, toggle between SHADER and FALLBACK, with SHADER defaulting to the currently scheduled
shader and FALLBACK defaulting to off.

--fallback-auto shader inference works by using the currently scheduled shader if SHADER is not
equal to it, or using the default shader otherwise.
*/
#[derive(Debug, Parser)]
#[command(after_help = AFTER_HELP, after_long_help = AFTER_LONG_HELP)]
pub struct Toggle {
    #[arg(help = SHADER_HELP, long_help = SHADER_HELP_LONG)]
    shader: Option<String>,

    /// Configuration variable used in rendering SHADER (may be specified multiple times)
    #[arg(long, value_name = "KEY=VALUE", value_parser = VarArgParser)]
    var: Vec<VarArg>,

    /// Specify fallback shader
    #[arg(long, group = "fallback_args")]
    fallback: Option<String>,

    /// Use default shader as fallback
    #[arg(long, group = "fallback_args")]
    fallback_default: bool,

    /// Infer fallback shader from config
    #[arg(long, group = "fallback_args")]
    fallback_auto: bool,

    /// Configuration variable used in rendering fallback shader (may be specified multiple times)
    ///
    /// Applies to `--fallback`, `--fallback-default`, and `--fallback-auto`
    #[arg(long, value_name = "KEY=VALUE", value_parser = VarArgParser, requires = "fallback_args")]
    var_fallback: Vec<VarArg>,
}

impl MergeVarArg for Toggle {}

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

        let now = now();

        // Eagerly evaluate --var and --var-fallback so that feedback is presented unconditionally
        let fallback_data = Self::merge_into_data(var_fallback)?;
        let shader_data = Self::merge_into_data(var)?;

        let shader = match &shader {
            Some(shader) => Some(Resolver::with_cli_arg(shader).resolve()?),
            None => config
                .ok_or_eyre("no configuration file found")
                .warning("A configuration file is required to call this command without SHADER")
                .and_then(|config| {
                    Schedule::with_config(config).scheduled_shader(&now.time())
                        .wrap_err("resolving shader in config")
                        .config_section(config.path())
                        .note("Since you omitted SHADER from cli arguments, it was inferred from the schedule in your configuration")
                        .suggestion("Change the shader name in your configuration, or make sure a shader by that name exists")
                })
                .with_suggestion(|| format!("For more information, see {README_CONFIGURATION}"))?,
        };

        let fallback = match (&fallback, fallback_default, fallback_auto) {
            (None, false, false) => None,
            (Some(fallback), false, false) => Some(Resolver::with_cli_arg(fallback).resolve()?),
            (None, true, false) => {
                config
                    .ok_or_eyre("no configuration file found")
                    .warning("A configuration file is required to use --fallback-default")
                    .and_then(|config| {
                        let default_shader = config.default_shader()
                            .ok_or_eyre("no default shader found in config")
                            .config_section(config.path())
                            .suggestion("Make sure a default shader is defined (default = true)")?;
                        Some(Resolver::with_name(&default_shader.name).resolve())
                            .transpose()
                            .wrap_err("resolving default shader in config")
                            .config_section(config.path())
                            .suggestion("Change the shader name in your configuration, or make sure a shader by that name exists")
                    })
                    .with_suggestion(|| format!("For more information, see {README_CONFIGURATION}"))?
            }
            (None, false, true) => {
                config
                    .ok_or_eyre("no configuration file found")
                    .warning("A configuration file is required to use --fallback-auto")
                    .and_then(|config| {
                        let scheduled_shader = Schedule::with_config(config).scheduled_shader(&now.time())
                            .wrap_err("resolving shader in config")
                            .config_section(config.path())
                            .note("Tried to resolve scheduled shader because of --fallback-auto")
                            .suggestion("Change the shader name in your configuration, or make sure a shader by that name exists")?;
                        if shader == scheduled_shader {
                            config.default_shader().map(|default_shader| Resolver::with_name(&default_shader.name).resolve())
                                .transpose()
                                .wrap_err("resolving default shader in config")
                                .config_section(config.path())
                                .note("Tried to resolve default shader because of --fallback-auto")
                                .suggestion("Change the shader name in your configuration, or make sure a shader by that name exists")
                        } else {
                            Ok(scheduled_shader)
                        }
                    })
                    .with_suggestion(|| format!("For more information, see {README_CONFIGURATION}"))?
            }
            _ => {
                unreachable!(
                    "--fallback, --fallback-default, and --fallback-auto are mutually exclusive"
                )
            }
        };

        let current_shader = Shader::current()?.map(Shader::try_from).transpose()?;

        let (designated_shader, designated_data) = if shader == current_shader {
            (fallback, fallback_data)
        } else {
            (shader, shader_data)
        };

        if let Some(designated_shader) = designated_shader {
            let designated_data = {
                let mut designated_data = designated_data;
                if let Some(config_data) = config.and_then(|c| c.data(designated_shader.name())) {
                    designated_data.merge_deep_keep(config_data.clone());
                }
                designated_data
            };

            designated_shader.on(&designated_data)?;
        } else {
            Shader::off()?;
        }

        Ok(ExitCode::SUCCESS)
    }
}
