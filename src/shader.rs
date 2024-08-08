use std::{
    fs::{self, File},
    os::unix::ffi::OsStrExt,
    path::PathBuf,
};

use crate::{
    builtin::BuiltinShader,
    constants::HYPRSHADE_RUNTIME_DIR,
    hyprctl,
    template::TemplateDataMap,
    util::{rsplit_file_at_dot, PathExt},
};

const TEMPLATE_EXTENSION: &str = "mustache";

#[derive(Debug)]
pub struct Shader(ShaderInner);

#[derive(Debug)]
enum ShaderInner {
    Path(PathBuf),
    Builtin(BuiltinShader),
}

impl Shader {
    pub fn from_path_buf(path_buf: PathBuf) -> Self {
        Self(ShaderInner::Path(path_buf))
    }

    pub fn from_builtin(builtin_shader: BuiltinShader) -> Self {
        Self(ShaderInner::Builtin(builtin_shader))
    }

    pub fn current() -> eyre::Result<Option<Self>> {
        match hyprctl::shader::get()? {
            Some(path) => {
                // FIXME: This is incorrect, since it doesn't take into account template shader
                // instances. The `Shader` instance should point to the template file (i.e.
                // *.glsl.mustache), not the template instance (i.e. *.glsl).
                Ok(Some(Self(ShaderInner::Path(path))))
            }
            None => Ok(None),
        }
    }

    pub fn off() -> eyre::Result<()> {
        hyprctl::shader::clear()
    }

    pub fn on(&self, data: &TemplateDataMap) -> eyre::Result<()> {
        let path = match &self.0 {
            ShaderInner::Path(path) => match path.file_name().map(rsplit_file_at_dot) {
                Some((Some(prefix), Some(extension))) if extension == TEMPLATE_EXTENSION => {
                    let template = mustache::compile_path(path)?;
                    let out_path = HYPRSHADE_RUNTIME_DIR.to_owned().join(prefix);
                    fs::create_dir_all(out_path.parent().unwrap())?;
                    let mut out_file = File::create(&out_path)?;
                    template.render(&mut out_file, data)?;
                    out_path
                }
                _ => path.clone(),
            },
            ShaderInner::Builtin(builtin_shader) => {
                if builtin_shader.is_template() {
                    builtin_shader.render(data)?
                } else {
                    builtin_shader.write()?
                }
            }
        };
        hyprctl::shader::set(&path)
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
