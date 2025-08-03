use std::{
    collections::HashSet,
    env,
    ffi::OsStr,
    fmt, fs,
    hash::Hash,
    io,
    path::{Path, PathBuf},
    process::Command,
};

use insta_cmd::get_cargo_bin;
use parking_lot::{Mutex, MutexGuard};
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

static TEST_MUTEX: Mutex<()> = Mutex::new(());

pub struct Space {
    #[allow(dead_code)]
    tempdir: TempDir,
    working_dir: PathBuf,
    home: PathBuf,
    time: Option<String>,

    /// Used to enforce sequential test execution
    _lock: MutexGuard<'static, ()>,
}

impl Space {
    #[track_caller]
    pub fn new() -> Self {
        let lock = TEST_MUTEX.lock();

        let tempdir = TempDir::new().unwrap();
        let working_dir = tempdir.path().join("working_dir");
        let home = bootstrap_home(tempdir.path());
        fs::create_dir_all(&working_dir).unwrap();
        Self {
            tempdir,
            working_dir,
            home,
            time: None,
            _lock: lock,
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

        const ENV_REMOVE: &[&str] = &["RUST_BACKTRACE", "COLORBT_SHOW_HIDDEN"];
        for key in ENV_REMOVE {
            cmd.env_remove(key);
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
        let shader_ident = shader_ident.into();
        let path = self.runtime_dir().join(shader_ident.to_path());
        match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(err) => panic!("failed reading {}: {}", path.display(), err),
        }
    }

    pub fn stash_runtime_shader(&self, shader_ident: impl Into<ShaderIdentifier>) -> FileDropGuard {
        let shader_ident = shader_ident.into();
        let path = self.runtime_dir().join(shader_ident.to_path());
        FileDropGuard::stash(path)
    }

    #[track_caller]
    pub fn stash_runtime_shaders(
        &self,
        shader_idents: impl IntoIterator<Item = impl Into<ShaderIdentifier>>,
    ) -> Vec<FileDropGuard> {
        fn duplicates<T>(iter: impl IntoIterator<Item = T>) -> HashSet<T>
        where
            T: Eq + Hash,
        {
            let mut seen = HashSet::new();
            iter.into_iter().fold(HashSet::new(), |mut duplicates, i| {
                if seen.contains(&i) {
                    duplicates.insert(i);
                } else {
                    seen.insert(i);
                }
                duplicates
            })
        }

        #[track_caller]
        fn assert_no_duplicates<T>(shader_idents: impl IntoIterator<Item = T>)
        where
            T: Eq + Hash + fmt::Debug,
        {
            let duplicates = duplicates(shader_idents);
            assert!(
                duplicates.is_empty(),
                "Shader names should not be given more than once.\nDuplicates: {}",
                duplicates
                    .iter()
                    .map(|s| format!("{:?}", s))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        let shader_idents = shader_idents
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();

        assert_no_duplicates(&shader_idents);

        shader_idents
            .into_iter()
            .map(|shader_ident| self.stash_runtime_shader(shader_ident))
            .collect()
    }

    #[track_caller]
    pub fn current_shader(&self) -> Option<String> {
        let output = self.hyprshade_cmd().arg("current").output().unwrap();
        assert!(output.status.success());

        let s = String::from_utf8(output.stdout).unwrap();
        let s = s.strip_suffix('\n').map(ToOwned::to_owned).unwrap_or(s);

        (!s.is_empty()).then_some(s)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ShaderIdentifier {
    name: &'static str,
}

impl fmt::Debug for ShaderIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.name, f)
    }
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

impl From<&&'static str> for ShaderIdentifier {
    fn from(value: &&'static str) -> Self {
        Self { name: *value }
    }
}

pub struct FileDropGuard {
    path: PathBuf,
    stash: Option<String>,
}

impl FileDropGuard {
    pub fn stash<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_owned();
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

#[doc(hidden)]
#[derive(Debug, thiserror::Error)]
pub enum CommandRunError {
    #[error("failed running command: {command}: {source}")]
    Io { command: String, source: io::Error },
    #[error(transparent)]
    Status(#[from] CommandStatusError),
}

#[doc(hidden)]
#[derive(Debug, thiserror::Error)]
#[error(
    "\
    failed running command: {command}\n\
    exit_code: {}\n\
    --- stdout ---\n\
    {}\n\
    --- stderr ---\n\
    {}\n\
    ",
    status.code().map_or_else(|| "none".to_owned(), |code| code.to_string()),
    String::from_utf8_lossy(.stdout),
    String::from_utf8_lossy(.stderr),
)]
pub struct CommandStatusError {
    command: String,
    status: std::process::ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

pub trait CommandExt {
    #[track_caller]
    fn run(&mut self) -> std::process::Output;

    #[doc(hidden)]
    fn _run(&mut self) -> Result<std::process::Output, CommandRunError>;
}

impl CommandExt for Command {
    fn run(&mut self) -> std::process::Output {
        match self._run() {
            Ok(output) => output,
            Err(err) => panic!("{}", err),
        }
    }

    fn _run(&mut self) -> Result<std::process::Output, CommandRunError> {
        let output = self.output().map_err(|source| CommandRunError::Io {
            command: format!("{:?}", self),
            source,
        })?;

        if output.status.success() {
            Ok(output)
        } else {
            let std::process::Output {
                status,
                stdout,
                stderr,
            } = output;

            Err(CommandRunError::Status(CommandStatusError {
                command: format!("{:?}", self),
                status,
                stdout,
                stderr,
            }))
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
