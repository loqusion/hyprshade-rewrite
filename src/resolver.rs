use std::{
    env, io,
    path::{Path, PathBuf, MAIN_SEPARATOR},
};

use directories::ProjectDirs;

const SYSTEM_HYPRSHADE_DIR: &str = concat!("/usr/share/", env!("CARGO_PKG_NAME"));

pub struct Resolver<'a>(ResolverInner<'a>);

enum ResolverInner<'a> {
    FromPath(ResolverFromPath<'a>),
    FromName(ResolverFromName<'a>),
}

struct ResolverFromPath<'a>(&'a Path);
struct ResolverFromName<'a>(&'a str);

impl<'a> Resolver<'a> {
    pub fn new(shader: &'a str) -> Self {
        if shader.contains(MAIN_SEPARATOR) {
            Self(ResolverInner::FromPath(ResolverFromPath(Path::new(shader))))
        } else {
            Self(ResolverInner::FromName(ResolverFromName(shader)))
        }
    }

    pub fn resolve(&self) -> Result<PathBuf, ResolverError> {
        match &self.0 {
            ResolverInner::FromPath(r) => r.resolve(),
            ResolverInner::FromName(r) => r.resolve(),
        }
    }
}

impl<'a> ResolverFromPath<'a> {
    fn resolve(&self) -> Result<PathBuf, ResolverError> {
        let Self(path) = self;
        let path = path.to_path_buf();

        match path.try_exists() {
            Ok(true) => Ok(path),
            Ok(false) => Err(ResolverError::IoError(
                path,
                io::Error::new(io::ErrorKind::NotFound, "No such file or directory"),
            )),
            Err(e) => Err(ResolverError::IoError(path, e)),
        }
    }
}

impl<'a> ResolverFromName<'a> {
    fn resolve(&self) -> Result<PathBuf, ResolverError> {
        for dir in Self::all_dirs() {
            if let Some(path) = self.resolve_in(&dir) {
                return Ok(path);
            }
        }

        todo!("ResolverFromName::resolve");
    }

    fn resolve_in(&self, dir: &Path) -> Option<PathBuf> {
        eprintln!("{dir:?}");

        None
    }

    fn all_dirs() -> Vec<PathBuf> {
        [
            env::var("HYPRSHADE_SHADERS_DIR").map(PathBuf::from).ok(),
            ProjectDirs::from("", "", "hypr").map(|p| p.config_dir().to_path_buf().join("shaders")),
            ProjectDirs::from("", "", env!("CARGO_PKG_NAME"))
                .map(|p| p.config_dir().to_path_buf().join("shaders")),
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
    #[error("Could not read path {0}")]
    IoError(PathBuf, #[source] io::Error),
}
