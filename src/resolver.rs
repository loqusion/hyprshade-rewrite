use std::{
    io,
    path::{PathBuf, MAIN_SEPARATOR},
};

pub fn resolve(shader: &str) -> Result<PathBuf, ResolverError> {
    if shader.contains(MAIN_SEPARATOR) {
        let path = PathBuf::from(shader);

        match path.try_exists() {
            Ok(true) => Ok(path),
            Ok(false) => Err(ResolverError::IoError(
                path,
                io::Error::new(io::ErrorKind::NotFound, "No such file or directory"),
            )),
            Err(e) => Err(ResolverError::IoError(path, e)),
        }
    } else {
        resolve_from_name(shader)
    }
}

fn resolve_from_name(_shader_name: &str) -> Result<PathBuf, ResolverError> {
    todo!()
}

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum ResolverError {
    #[error("Could not read path {0}")]
    IoError(PathBuf, #[source] io::Error),
}
