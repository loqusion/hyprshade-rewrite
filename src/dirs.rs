use std::{env, fs::create_dir_all, path::PathBuf};

use directories::{BaseDirs, ProjectDirs};

pub fn runtime_dir() -> PathBuf {
    let dir = BaseDirs::new()
        .and_then(|b| b.runtime_dir().map(PathBuf::from))
        .expect("failed to get XDG_RUNTIME_DIR")
        .join("hyprshade");
    create_dir_all(&dir).expect("failed to create runtime directory");
    dir
}

pub fn shader_dirs() -> Vec<PathBuf> {
    [
        ProjectDirs::from("", "", "hypr").map(|p| p.config_dir().to_path_buf().join("shaders")),
        ProjectDirs::from("", "", env!("CARGO_PKG_NAME"))
            .map(|p| p.config_dir().to_path_buf().join("shaders")),
        env::var("HYPRSHADE_SHADERS_DIR").map(PathBuf::from).ok(),
    ]
    .into_iter()
    .flatten()
    .collect()
}
