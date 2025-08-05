use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

use color_eyre::{Section, SectionExt, owo_colors::OwoColorize};

use crate::constants::HYPRSHADE_RUNTIME_DIR;

pub trait ConfigSection: Section {
    fn config_section(self, path: &Path) -> Self::Return;
}

impl ConfigSection for eyre::Report {
    fn config_section(self, path: &Path) -> Self::Return {
        self.with_section(|| path.display().yellow().to_string().header("Configuration:"))
    }
}

impl<T, E> ConfigSection for eyre::Result<T, E>
where
    E: Into<eyre::Report>,
{
    fn config_section(self, path: &Path) -> Self::Return {
        self.map_err(|error| error.into())
            .map_err(|report| report.config_section(path))
    }
}

pub fn make_runtime_path<P: AsRef<Path>>(file_name: P) -> io::Result<PathBuf> {
    fn _make_runtime_path(file_name: &Path) -> io::Result<PathBuf> {
        let out_path = HYPRSHADE_RUNTIME_DIR.to_owned().join(file_name);
        let parent = out_path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("failed to get parent of {out_path:?}"),
            )
        })?;
        fs::create_dir_all(parent)?;
        Ok(out_path)
    }

    _make_runtime_path(file_name.as_ref())
}

pub trait PathExt {
    #[must_use]
    fn file_prefix(&self) -> Option<&OsStr>;

    #[must_use]
    fn file_stem_extension(&self) -> (Option<&OsStr>, Option<&OsStr>);
}

impl PathExt for Path {
    fn file_prefix(&self) -> Option<&OsStr> {
        self.file_name()
            .map(split_file_at_dot)
            .map(|(before, _after)| before)
    }

    fn file_stem_extension(&self) -> (Option<&OsStr>, Option<&OsStr>) {
        self.file_name()
            .map(rsplit_file_at_dot)
            .unwrap_or((None, None))
    }
}

// basic workhorse for splitting stem and extension
fn rsplit_file_at_dot(file: &OsStr) -> (Option<&OsStr>, Option<&OsStr>) {
    if file.as_encoded_bytes() == b".." {
        return (Some(file), None);
    }

    // The unsafety here stems from converting between &OsStr and &[u8]
    // and back. This is safe to do because (1) we only look at ASCII
    // contents of the encoding and (2) new &OsStr values are produced
    // only from ASCII-bounded slices of existing &OsStr values.
    let mut iter = file.as_encoded_bytes().rsplitn(2, |b| *b == b'.');
    let after = iter.next();
    let before = iter.next();
    if before == Some(b"") {
        (Some(file), None)
    } else {
        unsafe {
            (
                before.map(|s| OsStr::from_encoded_bytes_unchecked(s)),
                after.map(|s| OsStr::from_encoded_bytes_unchecked(s)),
            )
        }
    }
}

fn split_file_at_dot(file: &OsStr) -> (&OsStr, Option<&OsStr>) {
    let slice = file.as_encoded_bytes();
    if slice == b".." {
        return (file, None);
    }

    // The unsafety here stems from converting between &OsStr and &[u8]
    // and back. This is safe to do because (1) we only look at ASCII
    // contents of the encoding and (2) new &OsStr values are produced
    // only from ASCII-bounded slices of existing &OsStr values.
    let i = match slice[1..].iter().position(|b| *b == b'.') {
        Some(i) => i + 1,
        None => return (file, None),
    };
    let before = &slice[..i];
    let after = &slice[i + 1..];
    unsafe {
        (
            OsStr::from_encoded_bytes_unchecked(before),
            Some(OsStr::from_encoded_bytes_unchecked(after)),
        )
    }
}
