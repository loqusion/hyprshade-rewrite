use std::{cell::LazyCell, process::ExitCode};

use clap::Parser;
use color_eyre::Section;
use const_format::{concatcp, formatcp};
use eyre::{OptionExt, eyre};

use crate::{
    cli::{
        CommandExecute,
        arg::{
            help::{SHADER_HELP, SHADER_HELP_LONG as SHADER_HELP_LONG_SOURCE},
            var::{MergeVarArg, VarArg, VarArgParser},
        },
    },
    config::Config,
    constants::README_CONFIGURATION,
    resolver::{self, Resolver},
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

        fn with_readme_suggestion<S: color_eyre::Section<Return = S>>(s: S) -> S {
            s.with_suggestion(|| format!("For more information, see {README_CONFIGURATION}"))
        }

        #[derive(Debug)]
        enum ScheduledShaderResult {
            NoConfig,
            ResolverError(resolver::Error),
            Shader(Option<Shader>),
        }

        let scheduled_shader_cell: LazyCell<ScheduledShaderResult, _> = LazyCell::new(|| {
            config.map_or(ScheduledShaderResult::NoConfig, |config| {
                Schedule::with_config(config)
                    .scheduled_shader(&now.time())
                    .map_or_else(
                        ScheduledShaderResult::ResolverError,
                        ScheduledShaderResult::Shader,
                    )
            })
        });

        #[derive(Clone, Debug)]
        enum ScheduledShaderCause {
            OmittedShader,
            FallbackAuto,
        }

        let scheduled_shader = |cause: ScheduledShaderCause| -> eyre::Result<Option<Shader>> {
            match &*scheduled_shader_cell {
                ScheduledShaderResult::NoConfig => {
                    let mut err = eyre!("no configuration file found");
                    err = match &cause {
                        ScheduledShaderCause::OmittedShader => err.warning(
                            "A configuration file is required to call this command without SHADER",
                        ),
                        ScheduledShaderCause::FallbackAuto => {
                            err.warning("A configuration file is required to use --fallback-auto")
                        }
                    };
                    err = with_readme_suggestion(err);
                    Err(err)
                }
                ScheduledShaderResult::ResolverError(err) => {
                    let mut err = eyre!("{}", err)
                        .wrap_err("resolving shader in config")
                        .config_section(config.expect("config file should exist").path());
                    err = match &cause {
                        ScheduledShaderCause::OmittedShader => err.note(
                            "Since you omitted SHADER from cli arguments, it was inferred from the schedule in your configuration",
                        ),
                        ScheduledShaderCause::FallbackAuto => err.note(
                            "Tried to resolve scheduled shader because of --fallback-auto",
                        ),
                    };
                    err = err.suggestion(
                        "Change the shader name in your configuration, or make sure a shader by that name exists",
                    );
                    err = with_readme_suggestion(err);
                    Err(err)
                }
                ScheduledShaderResult::Shader(shader) => Ok(shader.to_owned()),
            }
        };

        #[derive(Debug)]
        enum DefaultShaderResult {
            NoConfig,
            ResolverError(resolver::Error),
            None,
            Shader(Shader),
        }

        let default_shader_cell: LazyCell<DefaultShaderResult, _> = LazyCell::new(|| {
            config.map_or(DefaultShaderResult::NoConfig, |config| {
                config
                    .default_shader()
                    .map_or(DefaultShaderResult::None, |shader| {
                        Resolver::with_name(&shader.name).resolve().map_or_else(
                            DefaultShaderResult::ResolverError,
                            DefaultShaderResult::Shader,
                        )
                    })
            })
        });

        #[derive(Clone, Debug)]
        enum DefaultShaderCause {
            FallbackDefault,
            FallbackAuto,
        }

        let default_shader = |cause: DefaultShaderCause| -> eyre::Result<Option<Shader>> {
            match &*default_shader_cell {
                DefaultShaderResult::NoConfig => {
                    let mut err = eyre!("no configuration file found");
                    err = match &cause {
                        DefaultShaderCause::FallbackDefault => err
                            .warning("A configuration file is required to use --fallback-default"),
                        DefaultShaderCause::FallbackAuto => {
                            err.warning("A configuration file is required to use --fallback-auto")
                        }
                    };
                    err = with_readme_suggestion(err);
                    Err(err)
                }
                DefaultShaderResult::ResolverError(err) => {
                    let mut err = eyre!("{}", err)
                        .wrap_err("resolving default shader in config")
                        .config_section(config.expect("config file should exist").path());
                    err = match &cause {
                        DefaultShaderCause::FallbackDefault => err,
                        DefaultShaderCause::FallbackAuto => {
                            err.note("Tried to resolve default shader because of --fallback-auto")
                        }
                    };
                    err = err
                        .suggestion(
                            "Change the shader name in your configuration, or make sure a shader by that name exists",
                        );
                    err = with_readme_suggestion(err);
                    Err(err)
                }
                DefaultShaderResult::None => Ok(None),
                DefaultShaderResult::Shader(shader) => Ok(Some(shader.to_owned())),
            }
        };

        // Eagerly evaluate --var and --var-fallback so that feedback is presented unconditionally
        let fallback_data = Self::merge_into_data(var_fallback)?;
        let shader_data = Self::merge_into_data(var)?;

        let shader: Option<Shader> = match &shader {
            Some(shader) => Some(Resolver::with_cli_arg(shader).resolve()?),
            None => scheduled_shader(ScheduledShaderCause::OmittedShader)?,
        };

        let fallback = match (&fallback, fallback_default, fallback_auto) {
            (None, false, false) => None,
            (Some(fallback), false, false) => Some(Resolver::with_cli_arg(fallback).resolve()?),
            (None, true, false) => Some(
                default_shader(DefaultShaderCause::FallbackDefault).and_then(|shader| {
                    let result = shader
                        .ok_or_eyre("no default shader found in config")
                        .config_section(config.expect("config file should exist").path())
                        .suggestion("Make sure a default shader is defined (default = true)");
                    with_readme_suggestion(result)
                })?,
            ),
            (None, false, true) => {
                let scheduled_shader = scheduled_shader(ScheduledShaderCause::FallbackAuto)?;
                if shader == scheduled_shader {
                    default_shader(DefaultShaderCause::FallbackAuto)?
                } else {
                    scheduled_shader
                }
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
