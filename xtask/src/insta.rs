use std::process::ExitCode;

use xshell::{cmd, Shell};

use crate::hooks::RestoreShaderHook;

pub fn main(shell: Shell, args: &[String]) -> eyre::Result<ExitCode> {
    let hook = RestoreShaderHook::new();

    let result = cmd!(shell, "cargo insta {args...}").run();

    hook.after();
    result?;

    Ok(ExitCode::SUCCESS)
}
