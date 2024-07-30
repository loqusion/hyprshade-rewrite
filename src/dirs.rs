use std::{env, path::PathBuf};

use crate::constants::{HYPRLAND_CONFIG_DIR, HYPRSHADE_CONFIG_DIR, HYPRSHADE_SHADERS_DIR_ENV};

pub fn shader_dirs() -> Vec<PathBuf> {
    [
        Some(HYPRLAND_CONFIG_DIR.to_owned().join("shaders")),
        Some(HYPRSHADE_CONFIG_DIR.to_owned().join("shaders")),
        env::var(HYPRSHADE_SHADERS_DIR_ENV).map(PathBuf::from).ok(),
    ]
    .into_iter()
    .flatten()
    .collect()
}
