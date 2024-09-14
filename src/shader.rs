use std::{
    fs::{self, File},
    io,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    builtin::BuiltinShader,
    constants::HYPRSHADE_RUNTIME_DIR,
    hyprctl,
    resolver::{self, Resolver},
    template::TemplateDataMap,
    util::{make_runtime_path, rsplit_file_at_dot, PathExt},
};

const TEMPLATE_EXTENSION: &str = "mustache";

#[derive(Debug, Clone)]
pub struct Shader(ShaderInner);

#[derive(Debug, Clone)]
enum ShaderInner {
    Path(PathBuf),
    Builtin(BuiltinShader),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderInstance {
    source: ShaderSource,
    instance_path: PathBuf,
    data: TemplateDataMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ShaderSource {
    Path(PathBuf),
    Builtin(String),
}

impl Shader {
    pub fn from_path_buf(path_buf: PathBuf) -> Self {
        debug_assert!(
            path_buf.is_absolute(),
            "path should be canonicalized before passing to Shader::from_path_buf"
        );
        Self(ShaderInner::Path(path_buf))
    }

    pub fn from_builtin(builtin_shader: BuiltinShader) -> Self {
        Self(ShaderInner::Builtin(builtin_shader))
    }

    pub fn current() -> eyre::Result<Option<ShaderInstance>> {
        match hyprctl::shader::get()? {
            Some(path) => {
                if path.starts_with(*HYPRSHADE_RUNTIME_DIR) {
                    Ok(Some(ShaderInstance::read_alongside_shader(&path)?))
                } else {
                    Ok(Some(ShaderInstance::from_path_buf(path)))
                }
            }
            None => Ok(None),
        }
    }

    pub fn off() -> eyre::Result<()> {
        hyprctl::shader::clear()
    }

    pub fn on(&self, data: &TemplateDataMap) -> eyre::Result<()> {
        let path: PathBuf = match &self.0 {
            ShaderInner::Path(path) => match path
                .file_name()
                .map(rsplit_file_at_dot)
                .and_then(|v| v.0.zip(v.1))
            {
                Some((stem, extension)) if extension == TEMPLATE_EXTENSION => {
                    let template = mustache::compile_path(path)?;
                    let out_path = make_runtime_path(stem)?;
                    let mut out_file = File::create(&out_path)?;
                    template.render(&mut out_file, data)?;
                    out_path
                }
                _ => {
                    hyprctl::shader::set(path)?;
                    return Ok(());
                }
            },
            ShaderInner::Builtin(builtin_shader) => {
                let file_name = format!("{}.glsl", builtin_shader.name());
                let out_path = make_runtime_path(file_name)?;
                let mut out_file = File::create(&out_path)?;
                if builtin_shader.is_template() {
                    builtin_shader.render(&mut out_file, data)?;
                    out_path
                } else {
                    builtin_shader.write(&mut out_file)?;
                    out_path
                }
            }
        };
        hyprctl::shader::set(&path)?;

        let instance = ShaderInstance {
            source: self.0.clone().into(),
            instance_path: path,
            data: data.to_owned(),
        };
        instance.write_alongside_shader()?;

        Ok(())
    }

    pub fn name(&self) -> &str {
        match &self.0 {
            ShaderInner::Path(path) => {
                let prefix =
                    PathExt::file_prefix(path).unwrap_or_else(|| panic!("invalid path: {path:?}"));
                std::str::from_utf8(prefix.as_bytes())
                    .unwrap_or_else(|err| panic!("when converting {path:?}: {err}"))
            }
            ShaderInner::Builtin(builtin) => builtin.name(),
        }
    }
}

impl PartialEq for Shader {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (ShaderInner::Path(a), ShaderInner::Path(b)) => a == b,
            (ShaderInner::Builtin(a), ShaderInner::Builtin(b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for Shader {}

impl TryFrom<ShaderInstance> for Shader {
    type Error = ShaderConversionError;

    fn try_from(value: ShaderInstance) -> Result<Self, Self::Error> {
        value.to_shader()
    }
}

impl ShaderInstance {
    fn read_alongside_shader(path: &Path) -> Result<ShaderInstance, ReadShaderInstanceError> {
        let path = ShaderInstance::path_from_instance_path(path);
        let s = fs::read_to_string(&path).map_err(|source| ReadShaderInstanceError::Io {
            path: path.clone(),
            source,
        })?;
        serde_json::from_str(&s).map_err(|source| ReadShaderInstanceError::SerdeJson {
            path: path.clone(),
            source,
        })
    }

    fn write_alongside_shader(&self) -> Result<(), WriteShaderInstanceError> {
        let path = ShaderInstance::path_from_instance_path(&self.instance_path);
        let file = File::create(&path).map_err(|source| WriteShaderInstanceError::Io {
            path: path.clone(),
            source,
        })?;
        serde_json::to_writer(file, &self).map_err(|source| WriteShaderInstanceError::SerdeJson {
            path: path.clone(),
            source,
        })
    }

    fn path_from_instance_path(instance_path: &Path) -> PathBuf {
        instance_path.with_extension("json")
    }

    #[allow(dead_code)]
    pub fn restore(self) -> eyre::Result<()> {
        let shader = self.to_shader()?;
        shader.on(&self.data)
    }

    pub fn from_path_buf(path: PathBuf) -> ShaderInstance {
        ShaderInstance {
            source: ShaderSource::Path(path.clone()),
            instance_path: path,
            data: Default::default(),
        }
    }

    pub fn to_shader(&self) -> Result<Shader, ShaderConversionError> {
        match &self.source {
            ShaderSource::Path(path) => Resolver::with_path(path).resolve().map_err(|source| {
                ShaderConversionError::Resolver {
                    path: ShaderInstance::path_from_instance_path(&self.instance_path),
                    source,
                }
            }),
            ShaderSource::Builtin(name) => Resolver::with_name(&name).resolve().map_err(|source| {
                ShaderConversionError::Resolver {
                    path: ShaderInstance::path_from_instance_path(&self.instance_path),
                    source,
                }
            }),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("reading shader instance from {path:?}")]
enum ReadShaderInstanceError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    SerdeJson {
        path: PathBuf,
        source: serde_json::Error,
    },
}

#[derive(Debug, thiserror::Error)]
#[error("writing shader instance to {path:?}")]
enum WriteShaderInstanceError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    SerdeJson {
        path: PathBuf,
        source: serde_json::Error,
    },
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
#[error("converting from shader instance at {path:?}")]
pub enum ShaderConversionError {
    Resolver {
        path: PathBuf,
        source: resolver::Error,
    },
}

impl From<ShaderInner> for ShaderSource {
    fn from(value: ShaderInner) -> Self {
        match value {
            ShaderInner::Path(path) => ShaderSource::Path(path),
            ShaderInner::Builtin(builtin) => ShaderSource::Builtin(builtin.name().to_owned()),
        }
    }
}
