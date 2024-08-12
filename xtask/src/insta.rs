use std::process::ExitCode;

use xshell::{cmd, Shell};

use crate::{hooks::RestoreShaderHook, test::CARGO_TEST_FLAGS};

pub fn main(shell: Shell, args: &[String]) -> eyre::Result<ExitCode> {
    let hook = RestoreShaderHook::new();

    let result = match args {
        [subcommand, args @ ..] if subcommand == "test" => {
            cmd!(shell, "cargo insta test {CARGO_TEST_FLAGS...} {args...}").run()
        }
        args => cmd!(shell, "cargo insta {args...}").run(),
    };

    hook.after();
    result?;

    Ok(ExitCode::SUCCESS)
}
