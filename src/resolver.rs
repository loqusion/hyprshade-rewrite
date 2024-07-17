use std::{
    io,
    path::{PathBuf, MAIN_SEPARATOR},
};

pub fn resolve(shader: &str) -> Result<PathBuf, ResolverError> {
    if shader.contains(MAIN_SEPARATOR) {
        let path = PathBuf::from(shader);
        let path_exists = match path.try_exists() {
            Ok(exists) => exists,
            Err(e) => return Err(ResolverError::IoError(path, e)),
        };

        if path_exists {
            Ok(path)
        } else {
            Err(ResolverError::IoError(
                path,
                io::Error::new(io::ErrorKind::NotFound, "No such file or directory"),
            ))
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
