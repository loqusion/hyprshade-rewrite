use std::process::ExitCode;

use hyprshade::Shader;
use xshell::{cmd, Shell};

struct Hook {
    saved_shader: Option<Shader>,
}

impl Hook {
    fn new() -> Self {
        let saved_shader = Shader::current().unwrap_or_else(|err| {
            eprintln!("{err}");
            None
        });
        eprintln!("Saved shader: {saved_shader:?}");
        Self { saved_shader }
    }

    fn after(self) {
        let Self { saved_shader } = self;
        eprintln!("Restoring shader: {saved_shader:?}");
        match saved_shader {
            Some(shader) => shader
                .on(&Default::default())
                .unwrap_or_else(|err| eprintln!("{err}")),
            None => Shader::off().unwrap_or_else(|err| eprintln!("{err}")),
        }
    }
}

pub fn main(shell: Shell) -> eyre::Result<ExitCode> {
    let hook = Hook::new();

    let result = cmd!(shell, "cargo test").run();

    hook.after();
    result?;

    Ok(ExitCode::SUCCESS)
}
