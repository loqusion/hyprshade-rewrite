use std::process::ExitCode;

use crate::{
    cli::CommandExecute, config::Config, constants::README_CONFIGURATION, schedule::Schedule,
    shader::Shader,
};
use clap::Parser;
use color_eyre::Section;
use eyre::{Context, OptionExt};

/**
TODO: write help text
*/
#[derive(Debug, Parser)]
pub struct Auto;

impl CommandExecute for Auto {
    #[tracing::instrument(level = "debug", skip_all)]
    fn execute(self, config: Option<&Config>) -> eyre::Result<ExitCode> {
        let now = chrono::Local::now();
        let config = config
            .ok_or_eyre("requires config")
            .with_suggestion(|| format!("For more information, see {README_CONFIGURATION}"))?;

        if let Some(shader) = Schedule::with_config(config)
            .scheduled_shader(&now.time())
            .wrap_err_with(|| {
                format!("error resolving shader name in {}", config.path().display())
            })?
        {
            let data = config.data(shader.name());
            shader.on(data.unwrap_or(&Default::default()))?;
        } else {
            Shader::off()?;
        }

        Ok(ExitCode::SUCCESS)
    }
}
