use std::path::Path;

use directories::ProjectDirs;
use lazy_static::lazy_static;

pub const HYPRSHADE_CONFIG_FILE_ENV: &str = "HYPRSHADE_CONFIG_FILE";
pub const HYPRSHADE_SHADERS_DIR_ENV: &str = "HYPRSHADE_SHADERS_DIR";

lazy_static! {
    static ref HYPRSHADE_PROJECT_DIRS: ProjectDirs =
        ProjectDirs::from("", "", &env!("CARGO_PKG_NAME").replace('-', "_"))
            .expect("failed to get HOME");
    static ref HYPRLAND_PROJECT_DIRS: ProjectDirs =
        ProjectDirs::from("", "", "hypr").expect("failed to get HOME");
    pub static ref HYPRSHADE_CONFIG_DIR: &'static Path = HYPRSHADE_PROJECT_DIRS.config_dir();
    pub static ref HYPRLAND_CONFIG_DIR: &'static Path = HYPRLAND_PROJECT_DIRS.config_dir();
    pub static ref HYPRSHADE_RUNTIME_DIR: &'static Path = HYPRSHADE_PROJECT_DIRS
        .runtime_dir()
        .expect("failed to get XDG_RUNTIME_DIR");
}

pub const README_CONFIGURATION: &str = concat!(env!("CARGO_PKG_REPOSITORY"), "#configuration");
