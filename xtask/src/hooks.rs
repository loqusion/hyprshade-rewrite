use hyprshade::__private::{Shader, ShaderInstance};

pub struct RestoreShaderHook {
    saved_shader: Option<ShaderInstance>,
}

impl RestoreShaderHook {
    pub fn new() -> Self {
        let saved_shader = Shader::current().unwrap_or_else(|err| {
            eprintln!("{err}");
            None
        });
        eprintln!("Saved shader: {saved_shader:?}");
        Self { saved_shader }
    }

    pub fn after(self) {
        let Self { saved_shader } = self;
        eprintln!("Restoring shader: {saved_shader:?}");
        match saved_shader {
            Some(shader) => shader.restore().unwrap_or_else(|err| eprintln!("{err}")),
            None => Shader::off().unwrap_or_else(|err| eprintln!("{err}")),
        }
    }
}
