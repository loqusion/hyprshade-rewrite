use std::path::PathBuf;

use crate::{builtin::BuiltinShader, hyprctl, util::rsplit_file_at_dot};

const TEMPLATE_EXTENSION: &str = "mustache";

pub struct Shader(ShaderInner);

enum ShaderInner {
    Path(PathBuf),
    Builtin(&'static BuiltinShader),
}

impl Shader {
    pub fn from_path_buf(path_buf: PathBuf) -> Self {
        Self(ShaderInner::Path(path_buf))
    }

    pub fn from_builtin(builtin: &'static BuiltinShader) -> Self {
        Self(ShaderInner::Builtin(builtin))
    }

    pub fn current() -> eyre::Result<Option<Self>> {
        match hyprctl::shader::get()? {
            Some(path) => Ok(Some(Self(ShaderInner::Path(path)))),
            None => Ok(None),
        }
    }

    pub fn off() -> eyre::Result<()> {
        hyprctl::shader::clear()
    }

    pub fn on(&self) -> eyre::Result<()> {
        let path = match &self.0 {
            ShaderInner::Path(path) => match path.file_name().map(rsplit_file_at_dot) {
                Some((Some(_prefix), Some(extension))) if extension == TEMPLATE_EXTENSION => {
                    todo!("compile filesystem template shader");
                }
                _ => path,
            },
            ShaderInner::Builtin(builtin) => {
                if builtin.is_template() {
                    todo!("compile builtin shader");
                } else {
                    todo!("write shader to filesystem");
                }
            }
        };
        hyprctl::shader::set(path)
    }
}

impl std::fmt::Display for Shader {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("Display for Shader")
    }
}

impl PartialEq for Shader {
    fn eq(&self, _other: &Self) -> bool {
        todo!("PartialEq for Shader")
    }
}

pub trait OnOrOff {
    fn on_or_off(&self) -> eyre::Result<()>;
}

impl OnOrOff for Option<Shader> {
    fn on_or_off(&self) -> eyre::Result<()> {
        if let Some(shader) = self {
            shader.on()
        } else {
            Shader::off()
        }
    }
}
