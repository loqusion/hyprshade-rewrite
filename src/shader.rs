use std::{
    ffi::OsStr,
    path::{Path, PathBuf, MAIN_SEPARATOR},
};

use crate::hyprctl;

pub struct Shader<'a>(ShaderInner<'a>);

enum ShaderInner<'a> {
    FromPath(&'a Path),
    FromOwnedPath(PathBuf),
    FromName(&'a OsStr),
}

impl<'a> Shader<'a> {
    pub fn from_cli_arg(shader: &'a str) -> Self {
        if shader.contains(MAIN_SEPARATOR) {
            Self(ShaderInner::FromPath(Path::new(shader)))
        } else {
            Self(ShaderInner::FromName(OsStr::new(shader)))
        }
    }

    pub fn current() -> eyre::Result<Option<Self>> {
        match hyprctl::shader::get()? {
            Some(shader) => {
                let path_buf = PathBuf::from(&shader);
                Ok(Some(Self(ShaderInner::FromOwnedPath(path_buf))))
            }
            None => Ok(None),
        }
    }

    pub fn off() -> eyre::Result<()> {
        hyprctl::shader::clear()
    }

    pub fn on(&self) -> eyre::Result<()> {
        todo!("Shader::on");
    }
}

impl std::fmt::Display for Shader<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("Display for Shader")
    }
}

impl PartialEq for Shader<'_> {
    fn eq(&self, other: &Self) -> bool {
        todo!("PartialEq for Shader")
    }
}

pub trait OnOrOff {
    fn on_or_off(&self) -> eyre::Result<()>;
}

impl OnOrOff for Option<Shader<'_>> {
    fn on_or_off(&self) -> eyre::Result<()> {
        if let Some(shader) = self {
            shader.on()
        } else {
            Shader::off()
        }
    }
}
