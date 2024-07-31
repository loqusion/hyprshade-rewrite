use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use toml::value::Time;

use crate::template::TemplateDataMap;

pub struct Config {
    config: ConfigDocument,
    path: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "compat", serde(from = "CompatConfig"))]
pub struct ConfigDocument {
    #[serde(default)]
    pub shader: Vec<Shader>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Shader {
    pub name: String,
    pub start_time: Option<Time>,
    pub end_time: Option<Time>,
    #[serde(default)]
    pub default: bool,
    #[serde(default)]
    pub config: TemplateDataMap,
}

impl Config {
    pub fn read<P: AsRef<Path>>(path: P) -> eyre::Result<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)?;
        Ok(Self {
            config: ConfigDocument::from_str(&contents)?,
            path: path.to_owned(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn all_shaders(&self) -> &[Shader] {
        &self.config.shader
    }

    pub fn shader(&self, name: &str) -> Option<&Shader> {
        self.config.shader.iter().find(|shader| shader.name == name)
    }

    pub fn data(&self, name: &str) -> Option<&TemplateDataMap> {
        self.shader(name).map(|s| &s.config)
    }

    pub fn default_shader(&self) -> Option<&Shader> {
        self.config.shader.iter().find(|shader| shader.default)
    }
}

impl FromStr for ConfigDocument {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

#[cfg(feature = "compat")]
#[derive(Debug, Deserialize)]
pub struct CompatConfig {
    #[serde(alias = "shades", alias = "shaders", default)]
    pub shader: Vec<Shader>,
}

#[cfg(feature = "compat")]
impl From<CompatConfig> for ConfigDocument {
    fn from(value: CompatConfig) -> Self {
        let CompatConfig { shader } = value;
        Self { shader }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use toml::value::Datetime;

    #[test]
    #[cfg_attr(not(feature = "compat"), ignore)]
    fn compat() {
        let config: ConfigDocument = toml::from_str(
            r#"
                [[shades]]
                name = "hello"
                start_time = 12:00:00

                [[shades]]
                name = "wow"
                start_time = 14:00:00
                default = true
            "#,
        )
        .unwrap();

        assert_eq!(
            config.shader,
            [
                Shader {
                    name: "hello".to_owned(),
                    start_time: Some(Datetime::from_str("12:00:00").unwrap().time.unwrap()),
                    end_time: None,
                    default: false,
                    config: Default::default()
                },
                Shader {
                    name: "wow".to_owned(),
                    start_time: Some(Datetime::from_str("14:00:00").unwrap().time.unwrap()),
                    end_time: None,
                    default: true,
                    config: Default::default()
                },
            ]
        );
    }
}
