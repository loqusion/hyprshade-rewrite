use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fmt::{Display, Formatter},
    os::unix::process::ExitStatusExt,
    process::{Command, Output, Stdio},
    str,
};

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
        let output = self.command.output()?;
        if output.status.success() {
            Ok(output)
        } else if let Some(signal) = output.status.signal() {
            Err(anyhow!("'{PROGRAM_NAME}' terminated by signal {signal}"))
        } else {
            Err(anyhow!(self.error_context(
                &format!("{PROGRAM_NAME} returned a non-zero exit code"),
                None,
                &output
            )))
        }
    }

    fn json<T: for<'de> Deserialize<'de>>(&mut self) -> anyhow::Result<T> {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let command = self
            .command
            .get_program()
            .to_str()
            .expect("program name is not valid UTF-8");
        let args = self
            .command
            .get_args()
            .map(|s| s.to_str().expect("argument is not valid UTF-8"))
            .collect::<Vec<_>>()
            .join(" ");
        write!(f, "{command} {args}")
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

    fn get_value(&self) -> Option<&str> {
        if self.is_empty() {
            None
        } else {
            Some(&self.str)
        }
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

        Ok(option.get_value().map(|s| s.to_owned()))
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
