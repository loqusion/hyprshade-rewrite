use std::{
    env, io,
    path::{Path, PathBuf, MAIN_SEPARATOR},
};

use directories::ProjectDirs;

const SYSTEM_HYPRSHADE_DIR: &str = concat!("/usr/share/", env!("CARGO_PKG_NAME"));

trait ResolverTrait {
    fn resolve(&self) -> Result<PathBuf, ResolverError>;
}

pub struct Resolver<'a>(Box<dyn ResolverTrait + 'a>);

struct ResolverFromName<'a>(&'a str);
struct ResolverFromPath<'a>(&'a Path);

impl<'a> Resolver<'a> {
    pub fn new(shader: &'a str) -> Self {
        if shader.contains(MAIN_SEPARATOR) {
            Self(Box::new(ResolverFromPath(Path::new(shader))))
        } else {
            Self(Box::new(ResolverFromName(shader)))
        }
    }

    pub fn resolve(&self) -> Result<PathBuf, ResolverError> {
        self.0.resolve()
    }
}

impl<'a> ResolverFromName<'a> {
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

impl<'a> ResolverTrait for ResolverFromName<'a> {
    fn resolve(&self) -> Result<PathBuf, ResolverError> {
        for dir in Self::all_dirs() {
            eprintln!("{dir:?}");
        }

        todo!()
    }
}

impl<'a> ResolverTrait for ResolverFromPath<'a> {
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

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum ResolverError {
    #[error("Could not read path {0}")]
    IoError(PathBuf, #[source] io::Error),
}
