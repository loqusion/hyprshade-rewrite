use std::{
    ffi::OsStr,
    fmt::{self, Display, Formatter},
    iter,
    os::unix::process::ExitStatusExt,
    process::{Command, Output, Stdio},
    str,
};

use anyhow::{anyhow, Context};
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

    fn output_with_check(&mut self) -> anyhow::Result<Output> {
        let output = self
            .command
            .output()
            .with_context(|| format!("Failed to execute {PROGRAM_NAME}"))?;

        if output.status.success() {
            Ok(output)
        } else if let Some(signal) = output.status.signal() {
            Err(anyhow!("'{PROGRAM_NAME}' terminated by signal {signal}"))
        } else {
            let prelude = match output.status.code() {
                Some(0) | None => {
                    format!("{PROGRAM_NAME} terminated unsuccessfully (unknown cause)")
                }
                Some(code) => format!("{PROGRAM_NAME} terminated with exit code {code}"),
            };
            Err(anyhow!(self.error_context(&prelude, None, &output)))
        }
    }

    fn json<T: DeserializeOwned>(&mut self) -> anyhow::Result<T> {
        let output = self.output_with_check()?;
        serde_json::from_slice(&output.stdout)
            .with_context(|| self.error_context(
                &format!("{PROGRAM_NAME} returned invalid JSON, but failed to signal error via a non-zero exit code."),
                Some("This is likely a bug in Hyprland. Go bug Vaxry about it (nicely :))"),
                &output,
            ))
    }

    fn json_option(&mut self) -> anyhow::Result<HyprctlOption> {
        self.json()
    }

    fn error_context(&self, preamble: &str, epilogue: Option<&str>, output: &Output) -> String {
        let stdout = str::from_utf8(&output.stdout)
            .expect("stdout is not valid UTF-8")
            .trim();
        let stderr = str::from_utf8(&output.stderr)
            .expect("stderr is not valid UTF-8")
            .trim();

        let mut context = format!(
            "{preamble}
command:
{command}

stdout:
{stdout}

stderr:
{stderr}",
            command = self,
            stdout = if stdout.is_empty() { "<empty>" } else { stdout },
            stderr = if stderr.is_empty() { "<empty>" } else { stderr },
        );
        if let Some(epilogue) = epilogue {
            context.push_str(&format!("\n\n{}", epilogue));
        }
        context
    }
}

impl Display for HyprctlCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let full_command = iter::once(self.command.get_program())
            .chain(self.command.get_args())
            .collect::<Vec<_>>()
            .join(OsStr::new(" "))
            .into_string()
            .expect("command is not valid UTF-8");

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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(err.downcast_ref::<std::io::Error>().is_none());
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
        assert!(err.downcast_ref::<serde_json::Error>().is_some());
    }

    #[test]
    fn test_hyprctl_command_display() {
        assert_eq!(
            format!("{}", hyprctl_command_from("echo").args(["hello", "world"])),
            "echo hello world"
        );
    }
}

pub mod shader {
    use super::{HyprctlCommand, SHADER_EMPTY_STRING};
    use std::str;

    const VARIABLE_NAME: &str = "decoration:screen_shader";

    pub fn get() -> anyhow::Result<Option<String>> {
        let option = HyprctlCommand::new()
            .args(["-j", "getoption", VARIABLE_NAME])
            .json_option()?;

        Ok(option.get_value_string())
    }

    pub fn set(shader_path: &str) -> anyhow::Result<()> {
        HyprctlCommand::new()
            .args(["keyword", VARIABLE_NAME, shader_path])
            .output_with_check()?;

        Ok(())
    }

    pub fn clear() -> anyhow::Result<()> {
        set(SHADER_EMPTY_STRING)
    }
}
