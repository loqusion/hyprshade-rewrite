use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use insta_cmd::get_cargo_bin;
use tempfile::TempDir;

const DIRS: &[(&str, &str)] = &[("HOME", ""), ("XDG_CONFIG_HOME", ".config")];
const CONFIG_DIRS: &[&str] = &["hypr", "hyprshade"];

fn bootstrap_home(path: &Path) -> PathBuf {
    let home = path.join("home");
    for (_, path) in DIRS {
        let path = home.join(path);
        fs::create_dir_all(path).unwrap();
    }
    for config_dir in CONFIG_DIRS {
        let mut path = home.join(".config");
        path.push(config_dir);
        fs::create_dir_all(path).unwrap();
    }
    home
}

fn get_bin() -> PathBuf {
    get_cargo_bin(&env!("CARGO_PKG_NAME").replace('-', "_"))
}

pub struct Space {
    #[allow(dead_code)]
    tempdir: TempDir,
    working_dir: PathBuf,
    home: PathBuf,
    time: Option<String>,
}

impl Space {
    pub fn new() -> Self {
        let tempdir = TempDir::new().unwrap();
        let working_dir = tempdir.path().join("working_dir");
        let home = bootstrap_home(tempdir.path());
        fs::create_dir_all(&working_dir).unwrap();
        Self {
            tempdir,
            working_dir,
            home,
            time: None,
        }
    }

    pub fn with_time(&mut self, time: &str) -> &mut Self {
        self.time = Some(time.to_string());
        self
    }

    pub fn hyprshade_cmd(&self) -> Command {
        self.cmd(get_bin())
    }

    fn cmd<S: AsRef<OsStr>>(&self, program: S) -> Command {
        let mut cmd = Command::new(program);
        for (key, path) in DIRS {
            let path = self.home().join(path);
            cmd.env(key, path);
        }
        if let Some(time) = &self.time {
            cmd.env("__HYPRSHADE_MOCK_TIME", time);
        } else {
            cmd.env_remove("__HYPRSHADE_MOCK_TIME");
        }
        cmd.current_dir(&self.working_dir);
        cmd
    }

    pub fn working_dir(&self) -> &Path {
        self.working_dir.as_ref()
    }

    pub fn home(&self) -> &Path {
        self.home.as_ref()
    }
}
