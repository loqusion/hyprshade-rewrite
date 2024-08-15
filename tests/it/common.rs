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

pub const INSTA_FILTERS: &[(&str, &str)] = &[
    (r"/run/user/[[:digit:]]+/\S+?", "[RUNTIME_FILE]"),
    (
        r"/.*?/hyprshade-test-dir/home/.config/hyprshade/config.toml",
        "[HYPRSHADE_CONFIG]",
    ),
    (r"/.*?/hyprshade-test-dir/home", "[HYPRSHADE_HOME]"),
    (r"/tmp/.tmp\S+", "[TEMP_FILE]"),
    (
        r"(?:https?|ftp)://(?:[[:alnum:]_-]+\.)+[[:alpha:]]+(?:/[[:alnum:]_-]+)*(?:#[[:alnum:]_-]+)?(?:\?[[:alnum:]_-]*)?",
        "[URL]",
    ),
    (
        r"(Location:\s*)(?:\x1b\[\d+m)?.*?(?:\x1b\[\d+m)?:(?:\x1b\[\d+m)?\d+(?:\x1b\[\d+m)?(\s+)",
        "$1[LOCATION]$2",
    ),
    (
        r"\s?Backtrace omitted. Run with RUST_BACKTRACE=1 environment variable to display it.\s*Run with RUST_BACKTRACE=full to include source snippets.\s*",
        "",
    ),
];

fn bootstrap_home(path: &Path) -> PathBuf {
    let home = path.join("hyprshade-test-dir/home");
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

    pub fn with_any_time(&mut self) -> &mut Self {
        self.with_time("00:00:00")
    }

    pub fn with_config(&mut self, config: &str) -> &mut Self {
        let config_path = self.home().join(".config/hyprshade/config.toml");
        fs::write(&config_path, config)
            .unwrap_or_else(|err| panic!("failed writing to {}: {}", config_path.display(), err));
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

    #[allow(dead_code)]
    pub fn working_dir(&self) -> &Path {
        self.working_dir.as_ref()
    }

    pub fn home(&self) -> &Path {
        self.home.as_ref()
    }
}

pub trait CommandExt {
    fn run(&mut self);
}

impl CommandExt for Command {
    fn run(&mut self) {
        let output = self
            .output()
            .unwrap_or_else(|err| panic!("failed running {:?}: {}", self, err));

        if output.status.code() != Some(0) {
            let command = format!("{:?}", self);
            let code = output
                .status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "none".to_string());
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!(
                "\
                failed running {command}\n\
                exit_code: {code}\n\
                ---- stdout ----\n\
                {stdout}\n\
                ---- stderr ----\n\
                {stderr}\
                "
            );
        }
    }
}

macro_rules! hyprshade_cmd_snapshot {
    ($($arg:tt)*) => {{
        let mut settings = ::insta::Settings::clone_current();
        for &(matcher, replacement) in $crate::common::INSTA_FILTERS {
            settings.add_filter(matcher, replacement);
        }
        let _guard = settings.bind_to_scope();
        $crate::_hyprshade_cmd_snapshot_base!($($arg)*);
    }};
}

#[macro_export]
macro_rules! _hyprshade_cmd_snapshot_base {
    ($cmd:expr, @$snapshot:literal) => {
        ::insta_cmd::assert_cmd_snapshot!($cmd, @$snapshot);
    };
    ($name:expr, $cmd:expr) => {
        ::insta_cmd::assert_cmd_snapshot!($name, $cmd);
    };
}

pub(crate) use hyprshade_cmd_snapshot;
