use serde::{Deserialize, Serialize};
use toml::value::Time;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "compat", serde(from = "CompatConfig"))]
pub struct Config {
    #[serde(default)]
    pub shader: Vec<Shader>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Shader {
    pub name: String,
    pub start_time: Option<Time>,
    pub end_time: Option<Time>,
    #[serde(default)]
    pub default: bool,
    // TODO: Add config field when TemplateData is implemented
    // #[serde(default)]
    // pub config: TemplateData,
}

#[cfg(feature = "compat")]
#[derive(Debug, Deserialize)]
pub struct CompatConfig {
    #[serde(alias = "shades", alias = "shaders", default)]
    pub shader: Vec<Shader>,
}

#[cfg(feature = "compat")]
impl From<CompatConfig> for Config {
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
        let config: Config = toml::from_str(
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
                },
                Shader {
                    name: "wow".to_owned(),
                    start_time: Some(Datetime::from_str("14:00:00").unwrap().time.unwrap()),
                    end_time: None,
                    default: true
                }
            ]
        );
    }
}
