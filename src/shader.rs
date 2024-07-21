#![allow(clippy::enum_variant_names)]

use std::{
    ffi::OsStr,
    path::{Path, PathBuf, MAIN_SEPARATOR},
};

use crate::{hyprctl, resolver::Resolver, util::rsplit_file_at_dot};

const TEMPLATE_EXTENSION: &str = "mustache";

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
            Some(path) => Ok(Some(Self(ShaderInner::FromOwnedPath(path)))),
            None => Ok(None),
        }
    }

    pub fn off() -> eyre::Result<()> {
        hyprctl::shader::clear()
    }

    pub fn on(&self) -> eyre::Result<()> {
        let path = Resolver::from(self).resolve()?;
        let path = match path.file_name().map(rsplit_file_at_dot) {
            Some((Some(_prefix), Some(extension))) if extension == TEMPLATE_EXTENSION => {
                todo!("Shader::on template shaders");
            }
            _ => path,
        };
        hyprctl::shader::set(&path)
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

impl<'a> From<&'a Shader<'a>> for Resolver<'a> {
    fn from(value: &'a Shader<'a>) -> Self {
        match &value.0 {
            ShaderInner::FromPath(path) => Resolver::from_path(path),
            ShaderInner::FromOwnedPath(path) => Resolver::from_path(path),
            ShaderInner::FromName(name) => Resolver::from_name(name),
        }
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
