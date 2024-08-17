#![allow(dead_code)]

use std::io;

use phf::phf_map;

use crate::template::{MergeDeep, TemplateData, TemplateDataMap};

#[derive(Debug, Clone)]
pub struct BuiltinShader {
    name: &'static str,
    value: &'static BuiltinShaderValue,
}

type BuiltinShaders = phf::Map<&'static [u8], BuiltinShaderValue>;

#[derive(Debug)]
struct BuiltinShaderValue {
    contents: &'static str,
    is_template: bool,
    metadata: Metadata,
}

#[derive(Debug)]
struct Metadata {
    full_name: &'static str,
    description: &'static str,
    variables: phf::Map<&'static str, Variable>,
}

#[derive(Debug)]
enum Variable {
    Float {
        description: &'static str,
        min: f64,
        max: f64,
        default: f64,
    },
    Enum {
        description: &'static str,
        variants: &'static [&'static str],
        default: &'static str,
    },
    Map(phf::Map<&'static str, Variable>),
}

impl BuiltinShader {
    pub fn get<K>(name: &K) -> Option<BuiltinShader>
    where
        K: AsRef<[u8]> + ?Sized,
    {
        BuiltinShader::_get(name.as_ref())
    }

    fn _get(name: &[u8]) -> Option<BuiltinShader> {
        BUILTIN_SHADERS.get_entry(name).map(|(key, value)| {
            BuiltinShader {
                // SAFETY: All keys are valid UTF-8 strings.
                name: unsafe { std::str::from_utf8_unchecked(key) },
                value,
            }
        })
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn is_template(&self) -> bool {
        self.value.is_template
    }

    pub fn write<W: io::Write>(&self, wr: &mut W) -> io::Result<()> {
        wr.write_all(self.value.contents.as_bytes())
    }

    pub fn render<W: io::Write>(
        &self,
        out_file: &mut W,
        data: &TemplateDataMap,
    ) -> Result<(), RenderError> {
        debug_assert!(self.is_template());

        let template = mustache::compile_str(self.value.contents).map_err(|source| {
            RenderError::MustacheCompile {
                name: self.name().to_owned(),
                source,
            }
        })?;
        let data = {
            let mut self_data = self.data();
            self_data.merge_deep_force(data.clone());
            self_data
        };

        template
            .render(out_file, &data)
            .map_err(|source| RenderError::MustacheRender {
                name: self.name().to_owned(),
                source,
            })
    }

    fn data(&self) -> TemplateDataMap {
        let variables = &self.value.metadata.variables;
        TemplateDataMap::from_iter(
            variables
                .into_iter()
                .map(|(k, v)| (k.to_string(), TemplateData::from(v))),
        )
    }
}

impl PartialEq for BuiltinShader {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.name, other.name) && std::ptr::eq(self.value, other.value)
    }
}
impl Eq for BuiltinShader {}

impl From<&Variable> for TemplateData {
    fn from(value: &Variable) -> Self {
        match value {
            Variable::Float { default, .. } => TemplateData::Float(*default),
            Variable::Enum { default, .. } => {
                TemplateData::Enum(default.to_owned().to_ascii_uppercase())
            }
            Variable::Map(map) => TemplateData::from_iter(
                map.into_iter()
                    .map(|(k, v)| (k.to_string(), TemplateData::from(v))),
            ),
        }
    }
}

const BUILTIN_SHADERS: BuiltinShaders = phf_map! {
    b"blue-light-filter" => BuiltinShaderValue {
        contents: include_str!("shaders/blue-light-filter.glsl.mustache"),
        is_template: true,
        metadata: Metadata {
            full_name: "Blue Light Filter",
            description: "Use warmer colors to make the display easier on your eyes.",
            variables: phf_map! {
                "temperature" => Variable::Float {
                    description: "Color temperature in Kelvin.",
                    min: 1000.0,
                    max: 40000.0,
                    default: 2600.0,
                },
                "strength" => Variable::Float {
                    description: "Strength of the effect.",
                    min: 0.0,
                    max: 1.0,
                    default: 1.0,
                },
            },
        },
    },
    b"color-filter" => BuiltinShaderValue {
        contents: include_str!("shaders/color-filter.glsl.mustache"),
        is_template: true,
        metadata: Metadata {
            full_name: "Color Filter",
            description: "\
                Adjust colors for color vision deficiencies.\n\
                Supports protanopia (red-green), deuteranopia (green-red), and tritanopia (blue-yellow).\
            ",
            variables: phf_map! {
                "type" => Variable::Enum {
                    description: "\
                        Type of color correction.\n\
                        - \"protanopia\": Red-green color blindness.\n\
                        - \"deuteranopia\": Green-red color blindness.\n\
                        - \"tritanopia\": Blue-yellow color blindness.\
                    ",
                    variants: &[
                        "protanopia",
                        "protan",
                        "redgreen",
                        "deuteranopia",
                        "deutan",
                        "greenred",
                        "tritanopia",
                        "tritan",
                        "blueyellow",
                    ],
                    default: "protanopia",
                },
                "strength" => Variable::Float {
                    description: "Strength of the effect.",
                    min: 0.0,
                    max: 1.0,
                    default: 0.2,
                },
            },
        },
    },
    b"grayscale" => BuiltinShaderValue {
        contents: include_str!("shaders/grayscale.glsl.mustache"),
        is_template: true,
        metadata: Metadata {
            full_name: "Grayscale",
            description: "Use grayscale filter",
            variables: phf_map! {
                "type" => Variable::Enum {
                    description: "\
                        Type of grayscale conversion.\n\
                        - \"luminosity\": Use weighted average of RGB values.\n\
                        - \"lightness\": Use average of min and max RGB values.\n\
                        - \"average\": Use average of RGB values.\
                    ",
                    variants: &["luminosity", "lightness", "average"],
                    default: "luminosity",
                },
                "luminosity_type" => Variable::Enum {
                    description: "\
                        Type of luminosity calculation. (Only applies when type = \"luminosity\")\n\
                        - \"pal\": Use PAL/NTSC standard. (Rec. 601)\n\
                        - \"hdtv\": Use HDTV standard. (Rec. 709)\n\
                        - \"hdr\": Use HDR standard. (Rec. 2100)\n\
                    ",
                    variants: &["pal", "hdtv", "hdr"],
                    default: "hdr",
                },
            },
        },
    },
    b"invert-colors" => BuiltinShaderValue {
        contents: include_str!("shaders/invert-colors.glsl"),
        is_template: false,
        metadata: Metadata {
            full_name: "Invert Colors",
            description: "Invert colors so text and content stand out.",
            variables: phf_map! {},
        },
    },
    b"vibrance" => BuiltinShaderValue {
        contents: include_str!("shaders/vibrance.glsl.mustache"),
        is_template: true,
        metadata: Metadata {
            full_name: "Vibrance",
            description: "Enhance color saturation.",
            variables: phf_map! {
                "balance" => Variable::Map(phf_map! {
                    "red" => Variable::Float {
                        description: "Per-channel multiplier to vibrance strength (red).",
                        min: 0.0,
                        max: 10.0,
                        default: 1.0,
                    },
                    "green" => Variable::Float {
                        description: "Per-channel multiplier to vibrance strength (green).",
                        min: 0.0,
                        max: 10.0,
                        default: 1.0,
                    },
                    "blue" => Variable::Float {
                        description: "Per-channel multiplier to vibrance strength (blue).",
                        min: 0.0,
                        max: 10.0,
                        default: 1.0,
                    },
                }),
                "strength" => Variable::Float {
                    description: "Strength of vibrance effect. (Negative values will reduce vibrance.)",
                    min: -1.0,
                    max: 1.0,
                    default: 0.15,
                },
            },
        },
    },
};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RenderError {
    #[error("compiling mustache template for {name}")]
    MustacheCompile {
        name: String,
        source: mustache::Error,
    },
    #[error("rendering mustache template for {name}")]
    MustacheRender {
        name: String,
        source: mustache::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_shader_eq() {
        let entry_1 = BuiltinShader::get("blue-light-filter").unwrap();
        let entry_2 = BuiltinShader::get("blue-light-filter").unwrap();
        let entry_3 = BuiltinShader::get("vibrance").unwrap();

        assert_eq!(entry_1, entry_2);
        assert_ne!(entry_1, entry_3);
        assert_ne!(entry_2, entry_3);
    }
}
