//! Wrapper around the `hyprctl` binary
use std::{
    io,
    os::unix::process::ExitStatusExt,
    process::{Command, Output, Stdio},
};

use color_eyre::{
    eyre::{eyre, WrapErr},
    Section, SectionExt,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub const PROGRAM_NAME: &str = "hyprctl";

/// Special value for `decoration:screen_shader` meaning no shader is applied
const SHADER_EMPTY_STRING: &str = "[[EMPTY]]";

pub mod shader {
    use super::{hyprctl_command, HyprctlOption, JsonExt, OutputExt, SHADER_EMPTY_STRING};

    const VARIABLE_NAME: &str = "decoration:screen_shader";

    #[tracing::instrument(level = "debug")]
    pub fn get() -> eyre::Result<Option<String>> {
        let option = hyprctl_command()
            .args(["-j", "getoption", VARIABLE_NAME])
            .json::<HyprctlOption>()?;

        Ok(option.into_value())
    }

    #[tracing::instrument(level = "debug")]
    pub fn set(shader_path: &str) -> eyre::Result<()> {
        hyprctl_command()
            .args(["keyword", VARIABLE_NAME, shader_path])
            .output_with_check()?;

        Ok(())
    }

    #[tracing::instrument(level = "debug")]
    pub fn clear() -> eyre::Result<()> {
        set(SHADER_EMPTY_STRING)
    }
}

fn hyprctl_command() -> Command {
    let mut command = Command::new(PROGRAM_NAME);
    command.stdin(Stdio::null());
    command
}

#[derive(Debug, Serialize, Deserialize)]
struct HyprctlOption {
    option: String,
    str: String,
    set: bool,
}

impl HyprctlOption {
    fn is_empty(&self) -> bool {
        self.str == SHADER_EMPTY_STRING || self.str.is_empty()
    }

    fn into_value(self) -> Option<String> {
        if self.is_empty() {
            None
        } else {
            Some(self.str)
        }
    }
}

trait OutputExt {
    fn output_with_check(&mut self) -> eyre::Result<Output>;
}

impl OutputExt for Command {
    fn output_with_check(&mut self) -> eyre::Result<Output> {
        let output = self
            .output()
            .map_err(|err| {
                if err.kind() == io::ErrorKind::NotFound {
                    eyre::Report::new(err)
                        .with_suggestion(|| format!("Is {PROGRAM_NAME} located in your PATH?"))
                } else {
                    err.into()
                }
            })
            .wrap_err_with(|| format!("failed to execute {PROGRAM_NAME}"))?;

        if output.status.success() {
            Ok(output)
        } else if let Some(signal) = output.status.signal() {
            Err(eyre!("{PROGRAM_NAME} terminated by signal {signal}"))
        } else {
            let err = if let Some(code) = output.status.code() {
                Err(eyre!("{PROGRAM_NAME} terminated with exit code {code}"))
            } else {
                Err(eyre!(
                    "{PROGRAM_NAME} terminated unsuccessfully (unknown cause)"
                ))
            };
            err.command_sections(self, &output)
        }
    }
}

trait JsonExt {
    fn json<T: DeserializeOwned>(&mut self) -> eyre::Result<T>;
}

impl JsonExt for Command {
    fn json<T: DeserializeOwned>(&mut self) -> eyre::Result<T> {
        let output = self.output_with_check()?;
        let value = serde_json::from_slice(&output.stdout)
            .wrap_err_with(|| format!("failed to parse JSON returned by {PROGRAM_NAME}"))
            .command_sections(self, &output)
            .suggestion("This is likely a bug in Hyprland. Go bug Vaxry about it (nicely :))")?;

        Ok(value)
    }
}

trait CommandSectionExt: Section {
    fn command_sections(self, command: &Command, output: &Output) -> Self::Return;
}

impl<T, E> CommandSectionExt for eyre::Result<T, E>
where
    E: Into<eyre::Report>,
{
    fn command_sections(self, command: &Command, output: &Output) -> Self::Return {
        self.with_section(|| format!("{:?}", command).header("Command:"))
            .with_section(|| {
                String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string()
                    .header("Stdout:")
            })
            .with_section(|| {
                String::from_utf8_lossy(&output.stderr)
                    .trim()
                    .to_string()
                    .header("Stderr:")
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_with_check_error_on_non_zero_exit_code() {
        let err = Command::new("false").output_with_check().unwrap_err();
        assert_eq!(
            err.to_string(),
            format!("{PROGRAM_NAME} terminated with exit code 1")
        );
    }

    #[test]
    fn test_json_valid_json() {
        let value = Command::new("echo")
            .args([r#"{ "life": 42 }"#])
            .json::<serde_json::Value>()
            .unwrap();
        assert_eq!(value, serde_json::json!({"life": 42}));
    }

    #[test]
    fn test_json_invalid_json() {
        let err = Command::new("echo")
            .args(["{"])
            .json::<serde_json::Value>()
            .unwrap_err();
        assert!(err.downcast_ref::<serde_json::Error>().is_some());
    }
}
