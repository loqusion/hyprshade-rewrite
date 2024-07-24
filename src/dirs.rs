use std::{env, path::PathBuf};

use directories::ProjectDirs;

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
