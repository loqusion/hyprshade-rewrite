//! Wrapper around the `hyprctl` binary
use std::{
    os::unix::process::ExitStatusExt,
    process::{Command, Output, Stdio},
};

use color_eyre::{
    eyre::{eyre, Context},
    Section, SectionExt,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub const PROGRAM_NAME: &str = "hyprctl";

/// Special value for `decoration:screen_shader` meaning no shader is applied
const SHADER_EMPTY_STRING: &str = "[[EMPTY]]";

trait OutputExt {
    fn output_with_check(&mut self) -> eyre::Result<Output>;
}

impl OutputExt for Command {
    fn output_with_check(&mut self) -> eyre::Result<Output> {
        let output = self
            .output()
            .wrap_err_with(|| format!("Failed to execute {PROGRAM_NAME}"))?;

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
            err.with_section(|| format!("{:?}", self).header("Command:"))
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
}

trait CommandJsonExt {
    fn json<T: DeserializeOwned>(&mut self) -> eyre::Result<T>;
}

impl CommandJsonExt for Command {
    fn json<T: DeserializeOwned>(&mut self) -> eyre::Result<T> {
        let output = self.output_with_check()?;
        let value = serde_json::from_slice(&output.stdout)
            .wrap_err_with(|| format!("{PROGRAM_NAME} returned invalid JSON, but failed to signal an error via non-zero exit code"))
            .with_section(|| format!("{:?}", self).header("Command:"))
            .with_section(|| String::from_utf8_lossy(&output.stdout).trim().to_string().header("Stdout:"))
            .with_section(|| String::from_utf8_lossy(&output.stderr).trim().to_string().header("Stderr:"))
            .suggestion("This is likely a bug in Hyprland. Go bug Vaxry about it (nicely :))")?;

        Ok(value)
    }
}

fn hyprctl_command() -> Command {
    let mut command = Command::new(PROGRAM_NAME);
    command.stdin(Stdio::null());
    command
}

#[derive(Serialize, Deserialize)]
struct HyprctlOption {
    option: String,
    str: String,
    set: bool,
}

impl HyprctlOption {
    fn is_empty(&self) -> bool {
        self.str == SHADER_EMPTY_STRING || self.str.is_empty()
    }

    #[allow(dead_code)]
    fn get_value_str(&self) -> Option<&str> {
        if self.is_empty() {
            None
        } else {
            Some(&self.str)
        }
    }

    fn get_value_string(&self) -> Option<String> {
        if self.is_empty() {
            None
        } else {
            Some(self.str.clone())
        }
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
        assert!(err
            .to_string()
            .starts_with(&format!("{PROGRAM_NAME} returned invalid JSON")));
    }
}

pub mod shader {
    use super::{hyprctl_command, CommandJsonExt, HyprctlOption, OutputExt, SHADER_EMPTY_STRING};

    const VARIABLE_NAME: &str = "decoration:screen_shader";

    pub fn get() -> eyre::Result<Option<String>> {
        let option = hyprctl_command()
            .args(["-j", "getoption", VARIABLE_NAME])
            .json::<HyprctlOption>()?;

        Ok(option.get_value_string())
    }

    pub fn set(shader_path: &str) -> eyre::Result<()> {
        hyprctl_command()
            .args(["keyword", VARIABLE_NAME, shader_path])
            .output_with_check()?;

        Ok(())
    }

    pub fn clear() -> eyre::Result<()> {
        set(SHADER_EMPTY_STRING)
    }
}
