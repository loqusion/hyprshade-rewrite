use std::{path::Path, sync::LazyLock};

use directories::ProjectDirs;

pub const HYPRSHADE_CONFIG_FILE_ENV: &str = "HYPRSHADE_CONFIG_FILE";
pub const HYPRSHADE_SHADERS_DIR_ENV: &str = "HYPRSHADE_SHADERS_DIR";

static HYPRSHADE_PROJECT_DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    ProjectDirs::from("", "", &env!("CARGO_PKG_NAME").replace('-', "_"))
        .expect("failed to get HOME")
});

pub static HYPRSHADE_CONFIG_DIR: LazyLock<&'static Path> =
    LazyLock::new(|| HYPRSHADE_PROJECT_DIRS.config_dir());
pub static HYPRSHADE_RUNTIME_DIR: LazyLock<&'static Path> = LazyLock::new(|| {
    HYPRSHADE_PROJECT_DIRS
        .runtime_dir()
        .expect("failed to get XDG_RUNTIME_DIR")
});

static HYPRLAND_PROJECT_DIRS: LazyLock<ProjectDirs> =
    LazyLock::new(|| ProjectDirs::from("", "", "hypr").expect("failed to get HOME"));

pub static HYPRLAND_CONFIG_DIR: LazyLock<&'static Path> =
    LazyLock::new(|| HYPRLAND_PROJECT_DIRS.config_dir());

pub const README_CONFIGURATION: &str = concat!(env!("CARGO_PKG_REPOSITORY"), "#configuration");
pub const README_SCHEDULING: &str = concat!(env!("CARGO_PKG_REPOSITORY"), "#scheduling");
