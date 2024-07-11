//! Wrapper around the `hyprctl` binary
use std::{
    ffi::OsStr,
    fmt::{self, Debug, Formatter},
    io, iter,
    os::unix::process::ExitStatusExt,
    process::{Command, Output, Stdio},
    str,
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub const PROGRAM_NAME: &str = "hyprctl";

/// Special value for `decoration:screen_shader` meaning no shader is applied
const SHADER_EMPTY_STRING: &str = "[[EMPTY]]";

struct HyprctlCommand {
    command: Command,
}

impl HyprctlCommand {
    fn new() -> HyprctlCommand {
        let mut command = Command::new(PROGRAM_NAME);
        command.stdin(Stdio::null());
        HyprctlCommand { command }
    }

    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(args);
        self
    }

    fn output_with_check(&mut self) -> Result<Output> {
        let output = self.command.output()?;

        if output.status.success() {
            Ok(output)
        } else if let Some(signal) = output.status.signal() {
            Err(Error::Signal(signal))
        } else if let Some(code) = output.status.code() {
            Err(Error::Code {
                code,
                command: format!("{:?}", self),
                output,
            })
        } else {
            Err(Error::Unknown {
                command: format!("{:?}", self),
                output,
            })
        }
    }

    fn json<T: DeserializeOwned>(&mut self) -> Result<T> {
        let output = self.output_with_check()?;
        let value = serde_json::from_slice(&output.stdout).map_err(|source| Error::SerdeJson {
            source,
            command: format!("{:?}", self),
            output,
        })?;

        Ok(value)
    }

    fn json_option(&mut self) -> Result<HyprctlOption> {
        self.json()
    }
}

impl Debug for HyprctlCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let full_command = iter::once(self.command.get_program())
            .chain(self.command.get_args())
            .collect::<Vec<_>>()
            .join(OsStr::new(" "))
            .into_string()
            .unwrap_or("<invalid UTF-8>".into());

        write!(f, "{full_command}")
    }
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

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to execute {PROGRAM_NAME}")]
    Io {
        #[from]
        source: io::Error,
    },
    #[error(
        "{PROGRAM_NAME} returned invalid JSON, but failed to signal an error via non-zero exit code\n{}\n\n{}",
        Error::additional_context(.command, .output),
        "This is likely a bug in Hyprland. Go bug Vaxry about it (nicely :))",
    )]
    SerdeJson {
        source: serde_json::Error,
        command: String,
        output: Output,
    },
    #[error("{PROGRAM_NAME} terminated by signal {0}")]
    Signal(i32),
    #[error(
        "{PROGRAM_NAME} terminated with exit code {code}\n{}",
        Error::additional_context(.command, .output),
    )]
    Code {
        code: i32,
        command: String,
        output: Output,
    },
    #[error(
        "{PROGRAM_NAME} terminated unsuccessfully (unknown cause)\n{}",
        Error::additional_context(.command, .output),
    )]
    Unknown { command: String, output: Output },
}

impl Error {
    fn additional_context(command: &str, output: &Output) -> String {
        let stdout = str::from_utf8(&output.stdout)
            .map(str::trim)
            .map(|s| if s.is_empty() { "<empty>" } else { s })
            .unwrap_or("<invalid UTF-8>");
        let stderr = str::from_utf8(&output.stderr)
            .map(str::trim)
            .map(|s| if s.is_empty() { "<empty>" } else { s })
            .unwrap_or("<invalid UTF-8>");

        format!(
            "command:
{command}

stdout:
{stdout}

stderr:
{stderr}",
        )
    }
}

type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStrExt;

    fn hyprctl_command_from(program: &str) -> HyprctlCommand {
        let mut command = Command::new(program);
        command.stdin(Stdio::null());
        HyprctlCommand { command }
    }

    #[test]
    fn test_output_with_check_error_on_non_zero_exit_code() {
        let err = hyprctl_command_from("false")
            .output_with_check()
            .unwrap_err();
        assert!(matches!(err, Error::Code { code: 1, .. }))
    }

    #[test]
    fn test_json_valid_json() {
        let value = hyprctl_command_from("echo")
            .args([r#"{ "life": 42 }"#])
            .json::<serde_json::Value>()
            .unwrap();
        assert_eq!(value, serde_json::json!({"life": 42}));
    }

    #[test]
    fn test_json_invalid_json() {
        let err = hyprctl_command_from("echo")
            .args(["{"])
            .json::<serde_json::Value>()
            .unwrap_err();
        assert!(matches!(err, Error::SerdeJson { .. }))
    }

    #[test]
    fn test_hyprctl_command_debug() {
        assert_eq!(
            format!(
                "{:?}",
                hyprctl_command_from("echo").args(["hello", "world"])
            ),
            "echo hello world"
        );
    }

    #[test]
    fn test_hyprctl_command_debug_invalid_utf8() {
        let mut command = hyprctl_command_from("echo");
        let invalid_utf8_str = OsStr::from_bytes(&[0xFF, 0xFF, 0xFF]);
        command.command.arg(invalid_utf8_str);
        assert_eq!(format!("{:?}", command), "<invalid UTF-8>");
    }
}

pub mod shader {
    use super::{HyprctlCommand, Result, SHADER_EMPTY_STRING};

    const VARIABLE_NAME: &str = "decoration:screen_shader";

    pub fn get() -> Result<Option<String>> {
        let option = HyprctlCommand::new()
            .args(["-j", "getoption", VARIABLE_NAME])
            .json_option()?;

        Ok(option.get_value_string())
    }

    pub fn set(shader_path: &str) -> Result<()> {
        HyprctlCommand::new()
            .args(["keyword", VARIABLE_NAME, shader_path])
            .output_with_check()?;

        Ok(())
    }

    pub fn clear() -> Result<()> {
        set(SHADER_EMPTY_STRING)
    }
}
