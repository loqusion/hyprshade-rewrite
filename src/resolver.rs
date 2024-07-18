use std::{
    env,
    ffi::OsStr,
    io,
    path::{Path, PathBuf, MAIN_SEPARATOR},
};

use crate::util::PathExt;
use directories::ProjectDirs;
use tracing::{debug, trace};
use walkdir::WalkDir;

const SYSTEM_HYPRSHADE_DIR: &str = concat!("/usr/share/", env!("CARGO_PKG_NAME"));

const MAX_DEPTH: usize = 10;

pub struct Resolver<'a>(ResolverInner<'a>);

enum ResolverInner<'a> {
    FromPath(ResolverFromPath<'a>),
    FromName(ResolverFromName<'a>),
}

struct ResolverFromPath<'a>(&'a Path);
struct ResolverFromName<'a>(&'a OsStr);

impl<'a> Resolver<'a> {
    pub fn new(shader: &'a str) -> Self {
        if shader.contains(MAIN_SEPARATOR) {
            Self(ResolverInner::FromPath(ResolverFromPath(Path::new(shader))))
        } else {
            Self(ResolverInner::FromName(ResolverFromName(OsStr::new(
                shader,
            ))))
        }
    }

    pub fn resolve(&self) -> Result<PathBuf, ResolverError> {
        match &self.0 {
            ResolverInner::FromPath(r) => r.resolve(),
            ResolverInner::FromName(r) => r.resolve(),
        }
    }
}

impl ResolverFromPath<'_> {
    #[tracing::instrument(level = "debug", skip(self), fields(path = ?self.0))]
    fn resolve(&self) -> Result<PathBuf, ResolverError> {
        let Self(path) = *self;
        let path = path.to_path_buf();

        match path.try_exists() {
            Ok(true) => Ok(path),
            Ok(false) => Err(ResolverError::io_error_not_found(path)),
            Err(e) => Err(ResolverError::IoError(path, e)),
        }
    }
}

impl ResolverFromName<'_> {
    #[tracing::instrument(level = "debug", skip(self), fields(name = ?self.0.to_string_lossy()))]
    fn resolve(&self) -> Result<PathBuf, ResolverError> {
        for dir in Self::all_dirs() {
            if let Some(path) = self.resolve_in(&dir) {
                return Ok(path);
            }
        }

        Err(ResolverError::ShaderNameNotFound(
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
            Some([SYSTEM_HYPRSHADE_DIR, "shaders"].iter().collect()),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum ResolverError {
    #[error("Could not read path {0:?}")]
    IoError(PathBuf, #[source] io::Error),
    #[error("Shader named {0:?} not found")]
    ShaderNameNotFound(String),
}

impl ResolverError {
    fn io_error_not_found(path: PathBuf) -> Self {
        Self::IoError(
            path,
            io::Error::new(io::ErrorKind::NotFound, "No such file or directory"),
        )
    }
}
