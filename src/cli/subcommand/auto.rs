use std::process::ExitCode;

use crate::{
    cli::CommandExecute,
    config::Config,
    constants::{README_CONFIGURATION, README_SCHEDULING},
    schedule::Schedule,
    shader::Shader,
};
use clap::Parser;
use color_eyre::{owo_colors::OwoColorize, Section, SectionExt};
use const_format::formatcp;
use eyre::{Context, OptionExt};

const ABOUT: &str = "Activate the currently scheduled shader";
const LONG_ABOUT: &str = formatcp!(
    "\
    {ABOUT}\n\
    \n\
    Consults the configuration file to determine what shader to activate.\n\
    For more information, see {README_SCHEDULING}\
    "
);

#[derive(Debug, Parser)]
#[command(about = ABOUT, long_about = LONG_ABOUT)]
pub struct Auto;

impl CommandExecute for Auto {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self, config: Option<&Config>) -> eyre::Result<ExitCode> {
        let now = chrono::Local::now();
        let config = config
            .ok_or_eyre("no configuration file found")
            .warning("A configuration file is required to call this command")
            .with_suggestion(|| format!("For more information, see {README_CONFIGURATION}"))?;

        if let Some(shader) = Schedule::with_config(config).scheduled_shader(&now.time())
            .wrap_err("error resolving shader in config")
            .with_section(|| config.path().display().yellow().to_string().header("Configuration"))
            .suggestion("Change the shader name in your configuration, or make sure a shader by that name exists")
            .with_suggestion(|| format!("For more information, see {README_CONFIGURATION}"))?
        {
            let data = config.data(shader.name());
            shader.on(data.unwrap_or(&Default::default()))?;
        } else {
            Shader::off()?;
        }

        Ok(ExitCode::SUCCESS)
    }
}
