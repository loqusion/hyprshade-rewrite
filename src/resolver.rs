use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf, MAIN_SEPARATOR},
};

use tracing::{debug, trace};
use walkdir::WalkDir;

use crate::{builtin::BuiltinShader, dirs::shader_dirs, shader::Shader, util::PathExt};

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

    pub fn with_name<S: ?Sized + AsRef<OsStr>>(name: &'a S) -> Self {
        Self(ResolverInner::WithName(ResolverWithName(name.as_ref())))
    }

    pub fn resolve(&self) -> Result<Shader, Error> {
        match &self.0 {
            ResolverInner::WithPath(r) => Ok(r.resolve()?),
            ResolverInner::WithName(r) => Ok(r.resolve()?),
        }
    }
}

impl ResolverWithPath<'_> {
    #[tracing::instrument(level = "debug", skip(self), fields(path = ?self.0))]
    fn resolve(&self) -> Result<Shader, ErrorFromPath> {
        let Self(path) = *self;

        match path.try_exists() {
            Ok(true) => {
                let path =
                    fs::canonicalize(path).map_err(|source| ErrorFromPath::Canonicalize {
                        path: path.to_path_buf(),
                        source,
                    })?;
                Ok(Shader::from_path_buf(path.to_path_buf()))
            }
            Ok(false) => Err(ErrorFromPath::io_error_not_found(path.to_path_buf())),
            Err(e) => Err(ErrorFromPath::IoError(path.to_path_buf(), e)),
        }
    }
}

impl ResolverWithName<'_> {
    #[tracing::instrument(level = "debug", skip(self), fields(name = ?self.0.to_string_lossy()))]
    fn resolve(&self) -> Result<Shader, ErrorFromName> {
        let Self(name) = &self;

        for dir in shader_dirs() {
            if let Some(path) = self.resolve_in(&dir) {
                let path =
                    fs::canonicalize(&path).map_err(|source| ErrorFromName::Canonicalize {
                        name: name.to_string_lossy().into_owned(),
                        path,
                        source,
                    })?;
                trace!("Resolved {name:?} to {path:?}");
                return Ok(Shader::from_path_buf(path));
            }
        }

        if let Some(builtin_shader) = BuiltinShader::get(name.as_encoded_bytes()) {
            trace!("Resolved {name:?} to builtin shader");
            return Ok(Shader::from_builtin(builtin_shader));
        }

        Err(ErrorFromName::ShaderNameNotFound(
            name.to_string_lossy().into_owned(),
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
}

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    FromPath(#[from] ErrorFromPath),
    #[error(transparent)]
    FromName(#[from] ErrorFromName),
}

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum ErrorFromPath {
    #[error("could not read path {0:?}")]
    IoError(PathBuf, #[source] io::Error),
    #[error("could not canonicalize path {path:?}")]
    Canonicalize { path: PathBuf, source: io::Error },
}

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum ErrorFromName {
    #[error("shader named {0:?} not found")]
    ShaderNameNotFound(String),
    #[error("could not canonicalize path {path:?} for shader {name:?}")]
    Canonicalize {
        name: String,
        path: PathBuf,
        source: io::Error,
    },
}

impl ErrorFromPath {
    fn io_error_not_found(path: PathBuf) -> Self {
        Self::IoError(
            path,
            io::Error::new(io::ErrorKind::NotFound, "No such file or directory"),
        )
    }
}
