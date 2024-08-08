use std::{
    fs::{self, File},
    io,
    path::PathBuf,
};

use crate::{
    constants::HYPRSHADE_RUNTIME_DIR,
    template::{MergeDeep, TemplateData, TemplateDataMap},
};
use phf::phf_map;

#[derive(Debug, Clone)]
pub struct BuiltinShader {
    name: &'static str,
    value: &'static BuiltinShaderValue,
}

pub struct BuiltinShaders {
    inner: phf::Map<&'static [u8], BuiltinShaderValue>,
}

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

impl BuiltinShaders {
    pub fn get<K>(&'static self, key: &K) -> Option<BuiltinShader>
    where
        K: AsRef<[u8]> + ?Sized,
    {
        self.inner
            .get_entry(key.as_ref())
            .map(|(key, value)| BuiltinShader {
                // SAFETY: All keys are valid UTF-8 strings.
                name: unsafe { std::str::from_utf8_unchecked(key) },
                value,
            })
    }

    const fn new(inner: phf::Map<&'static [u8], BuiltinShaderValue>) -> Self {
        Self { inner }
    }
}

impl BuiltinShader {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn is_template(&self) -> bool {
        self.value.is_template
    }

    pub fn write(&self) -> io::Result<PathBuf> {
        let out_path = HYPRSHADE_RUNTIME_DIR
            .to_owned()
            .join(format!("{}.glsl", self.name()));
        fs::create_dir_all(out_path.parent().unwrap())?;
        fs::write(&out_path, self.value.contents)?;
        Ok(out_path)
    }

    pub fn render(&self, data: &TemplateDataMap) -> eyre::Result<PathBuf> {
        debug_assert!(self.is_template());

        let template = mustache::compile_str(self.value.contents)?;
        let data = {
            let mut self_data = self.data();
            self_data.merge_deep_force(data.clone());
            self_data
        };
        let out_path = HYPRSHADE_RUNTIME_DIR
            .to_owned()
            .join(format!("{}.glsl", self.name()));
        fs::create_dir_all(out_path.parent().unwrap())?;
        let mut out_file = File::create(&out_path)?;
        template.render(&mut out_file, &data)?;

        Ok(out_path)
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

pub const BUILTIN_SHADERS: BuiltinShaders = BuiltinShaders::new(phf_map! {
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
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_pointer_equality() {
        let raw_entry = BUILTIN_SHADERS
            .inner
            .get_entry(b"blue-light-filter")
            .unwrap();
        let wrapper_entry = BUILTIN_SHADERS.get("blue-light-filter").unwrap();
        assert!(std::ptr::eq(
            *raw_entry.0,
            wrapper_entry.name as *const str as *const [u8]
        ));
        assert!(std::ptr::eq(raw_entry.1, wrapper_entry.value));
    }
}
