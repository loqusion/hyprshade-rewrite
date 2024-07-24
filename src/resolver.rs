use std::{
    env,
    ffi::OsStr,
    io,
    path::{Path, PathBuf, MAIN_SEPARATOR},
};

use crate::{builtin::BUILTIN_SHADERS, shader::Shader, util::PathExt};
use directories::ProjectDirs;
use tracing::{debug, trace};
use walkdir::WalkDir;

#[cfg(feature = "compat")]
const SYSTEM_HYPRSHADE_DIR: &str = concat!("/usr/share/", env!("CARGO_PKG_NAME"));

const MAX_DEPTH: usize = 10;

pub struct Resolver<'a>(ResolverInner<'a>);

enum ResolverInner<'a> {
    WithPath(ResolverWithPath<'a>),
    WithName(ResolverWithName<'a>),
}

struct ResolverWithPath<'a>(&'a Path);
struct ResolverWithName<'a>(&'a OsStr);

impl<'a> Resolver<'a> {
    pub fn with_cli_arg(shader: &'a str) -> Self {
        if shader.contains(MAIN_SEPARATOR) {
            Self::with_path(Path::new(shader))
        } else {
            Self::with_name(OsStr::new(shader))
        }
    }

    pub fn with_path(path: &'a Path) -> Self {
        Self(ResolverInner::WithPath(ResolverWithPath(path)))
    }

    pub fn with_name(name: &'a OsStr) -> Self {
        Self(ResolverInner::WithName(ResolverWithName(name)))
    }

    pub fn resolve(self) -> Result<Shader, Error> {
        match self.0 {
            ResolverInner::WithPath(r) => r.resolve(),
            ResolverInner::WithName(r) => r.resolve(),
        }
    }
}

impl ResolverWithPath<'_> {
    #[tracing::instrument(level = "debug", skip(self), fields(path = ?self.0))]
    fn resolve(self) -> Result<Shader, Error> {
        let Self(path) = self;

        match path.try_exists() {
            Ok(true) => Ok(Shader::from_path_buf(path.to_path_buf())),
            Ok(false) => Err(Error::io_error_not_found(path.to_path_buf())),
            Err(e) => Err(Error::IoError(path.to_path_buf(), e)),
        }
    }
}

impl ResolverWithName<'_> {
    #[tracing::instrument(level = "debug", skip(self), fields(name = ?self.0.to_string_lossy()))]
    fn resolve(self) -> Result<Shader, Error> {
        let Self(name) = self;

        for dir in Self::all_dirs() {
            if let Some(path) = self.resolve_in(&dir) {
                trace!("Resolved {name:?} to {path:?}");
                return Ok(Shader::from_path_buf(path));
            }
        }

        if let Some(builtin_shader) = BUILTIN_SHADERS.get_entry(name.as_encoded_bytes()) {
            trace!("Resolved {name:?} to builtin shader");
            return Ok(Shader::from_builtin(builtin_shader));
        }

        Err(Error::ShaderNameNotFound(
            self.0.to_string_lossy().into_owned(),
        ))
    }

    #[tracing::instrument(level = "debug", skip(self), fields(name = ?self.0.to_string_lossy(), ?dir))]
    fn resolve_in(&self, dir: &Path) -> Option<PathBuf> {
        let Self(name) = *self;

        if !dir.is_dir() {
            debug!("Not a directory: {dir:?}");
            return None;
        }

        trace!("Searching for {name:?} in {dir:?}");

        for entry in WalkDir::new(dir)
            .max_depth(MAX_DEPTH)
            .into_iter()
            .filter_map(|e| {
                e.inspect_err(|err| {
                    debug!("Ignoring error encountered when walking directory {dir:?}");
                    debug!(?err);
                })
                .ok()
                .and_then(|e| e.file_type().is_file().then_some(e))
            })
        {
            trace!("Checking {entry:?}");

            let prefix = PathExt::file_prefix(entry.path());
            if Some(name) == prefix {
                trace!("Entry matches {name:?}");

                return Some(entry.into_path());
            }
        }

        None
    }

    fn all_dirs() -> Vec<PathBuf> {
        [
            ProjectDirs::from("", "", "hypr").map(|p| p.config_dir().to_path_buf().join("shaders")),
            ProjectDirs::from("", "", env!("CARGO_PKG_NAME"))
                .map(|p| p.config_dir().to_path_buf().join("shaders")),
            env::var("HYPRSHADE_SHADERS_DIR").map(PathBuf::from).ok(),
            #[cfg(feature = "compat")]
            Some([SYSTEM_HYPRSHADE_DIR, "shaders"].iter().collect()),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not read path {0:?}")]
    IoError(PathBuf, #[source] io::Error),
    #[error("Shader named {0:?} not found")]
    ShaderNameNotFound(String),
}

impl Error {
    fn io_error_not_found(path: PathBuf) -> Self {
        Self::IoError(
            path,
            io::Error::new(io::ErrorKind::NotFound, "No such file or directory"),
        )
    }
}
