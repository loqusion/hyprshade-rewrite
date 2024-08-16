use std::{
    env,
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

const FIXTURE_SIMPLE: &str = include_str!("./fixtures/simple.glsl");

#[track_caller]
fn bootstrap_home(path: &Path) -> PathBuf {
    let home = path.join("hyprshade-test-dir/home");
    for (_, dir_name) in DIRS {
        let p = home.join(dir_name);
        fs::create_dir_all(p).unwrap();
    }
    for dir_name in CONFIG_DIRS {
        let mut p = home.join(".config");
        p.push(dir_name);
        fs::create_dir_all(p).unwrap();
    }
    home
}

fn get_bin() -> PathBuf {
    get_cargo_bin(env!("CARGO_PKG_NAME"))
}

pub struct Space {
    #[allow(dead_code)]
    tempdir: TempDir,
    working_dir: PathBuf,
    home: PathBuf,
    time: Option<String>,
}

impl Space {
    #[track_caller]
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

    #[track_caller]
    pub fn with_config(&mut self, config: &str) -> &mut Self {
        let config_path = self.home().join(".config/hyprshade/config.toml");
        if let Err(err) = fs::write(&config_path, config) {
            panic!("failed writing to {}: {}", config_path.display(), err);
        }
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

    #[track_caller]
    pub fn runtime_dir(&self) -> PathBuf {
        let runtime_dir = match env::var("XDG_RUNTIME_DIR") {
            Ok(dir) => dir,
            Err(err) => panic!("fetching XDG_RUNTIME_DIR: {err}"),
        };
        PathBuf::from(runtime_dir).join("hyprshade")
    }

    #[track_caller]
    pub fn fixture_simple(&self) -> PathBuf {
        let fixture_path = self.runtime_dir().join("simple.glsl");
        if let Err(err) = fs::write(&fixture_path, FIXTURE_SIMPLE) {
            panic!("writing to {}: {}", fixture_path.display(), err);
        }
        fixture_path
    }

    #[track_caller]
    pub fn read_runtime_shader(&self, shader_ident: impl Into<ShaderIdentifier>) -> String {
        self._read_runtime_shader(shader_ident.into())
    }

    #[track_caller]
    fn _read_runtime_shader(&self, shader_ident: ShaderIdentifier) -> String {
        let path = self.runtime_dir().join(shader_ident.to_path());
        match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(err) => panic!("failed reading {}: {}", path.display(), err),
        }
    }

    pub fn stash_runtime_shader(&self, shader_ident: impl Into<ShaderIdentifier>) -> FileDropGuard {
        self._stash_runtime_shader(shader_ident.into())
    }

    fn _stash_runtime_shader(&self, shader_ident: ShaderIdentifier) -> FileDropGuard {
        let path = self.runtime_dir().join(shader_ident.to_path());
        FileDropGuard::stash(path)
    }
}

pub struct ShaderIdentifier {
    name: &'static str,
}

impl ShaderIdentifier {
    pub fn to_path(&self) -> PathBuf {
        PathBuf::from(format!("{}.glsl", self.name))
    }
}

impl From<&'static str> for ShaderIdentifier {
    fn from(value: &'static str) -> Self {
        Self { name: value }
    }
}

pub struct FileDropGuard {
    path: PathBuf,
    stash: Option<String>,
}

impl FileDropGuard {
    pub fn stash<P: AsRef<Path>>(path: P) -> Self {
        Self::_stash(path.as_ref().to_owned())
    }

    fn _stash(path: PathBuf) -> Self {
        let stash = fs::read_to_string(&path).ok();
        if stash.is_some() {
            fs::remove_file(&path).ok();
        }
        Self { path, stash }
    }
}

impl Drop for FileDropGuard {
    fn drop(&mut self) {
        if let Some(contents) = &self.stash {
            fs::write(&self.path, contents).unwrap_or_else(|err| {
                eprintln!("failed restoring stash to {}: {}", self.path.display(), err);
            });
        }
    }
}

pub trait CommandExt {
    #[track_caller]
    fn run(&mut self);
}

impl CommandExt for Command {
    fn run(&mut self) {
        let output = match self.output() {
            Ok(output) => output,
            Err(err) => panic!("failed running {:?}: {}", self, err),
        };

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

#[cfg(test)]
mod tests {
    use super::*;

    fn stash(path: &Path) {
        let _stash = FileDropGuard::stash(path);
        assert!(!path.exists());
    }

    #[test]
    fn test_stash() {
        let space = Space::new();
        let path = space.working_dir().join("file.txt");
        fs::write(&path, "content").unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "content");
        stash(&path);
        assert_eq!(fs::read_to_string(&path).unwrap(), "content");
    }
}
