use std::process::ExitCode;

use xshell::{cmd, Shell};

use crate::hooks::RestoreShaderHook;

pub const CARGO_TEST_FLAGS: &[&str] = &["--features", "mock-time"];

pub fn main(shell: Shell, args: &[String]) -> eyre::Result<ExitCode> {
    let hook = RestoreShaderHook::new();

    let result = cmd!(shell, "cargo test {CARGO_TEST_FLAGS...} -- {args...}").run();

    hook.after();
    result?;

    Ok(ExitCode::SUCCESS)
}
