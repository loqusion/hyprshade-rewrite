use std::{
    collections::HashMap,
    fs::{self, File},
    io,
    path::PathBuf,
};

use crate::constants::HYPRSHADE_RUNTIME_DIR;
use phf::phf_map;

pub struct BuiltinShaders(phf::Map<&'static [u8], BuiltinShaderValue>);

#[derive(Debug)]
pub struct BuiltinShader<'a>(&'static str, &'a BuiltinShaderValue);

#[derive(Debug)]
pub struct BuiltinShaderValue {
    contents: &'static str,
    is_template: bool,
    metadata: Metadata,
}

#[derive(Debug)]
pub struct Metadata {
    full_name: &'static str,
    description: &'static str,
    variables: phf::Map<&'static str, Variable>,
}

#[derive(Debug)]
pub enum Variable {
    Float {
        description: &'static str,
        min: f32,
        max: f32,
        default: f32,
    },
    Enum {
        description: &'static str,
        variants: &'static [&'static str],
        default: &'static str,
    },
    Dict(phf::Map<&'static str, Variable>),
}

impl BuiltinShaders {
    pub fn get_entry<K>(&self, key: &K) -> Option<BuiltinShader>
    where
        K: AsRef<[u8]> + ?Sized,
    {
        self.0.get_entry(key.as_ref()).map(|(key, value)|
                // SAFETY: All keys are valid UTF-8 strings.
                unsafe { BuiltinShader(std::str::from_utf8_unchecked(key), value) })
    }
}

impl BuiltinShader<'_> {
    pub fn write(&self) -> io::Result<PathBuf> {
        let out_path = HYPRSHADE_RUNTIME_DIR
            .to_owned()
            .join(format!("{}.glsl", self.0));
        fs::create_dir_all(out_path.parent().unwrap())?;
        fs::write(&out_path, self.1.contents)?;
        Ok(out_path)
    }
}

impl BuiltinShader<'_> {
    pub fn is_template(&self) -> bool {
        self.1.is_template
    }

    pub fn render_data(&self, data: &mustache::Data) -> eyre::Result<PathBuf> {
        debug_assert!(self.is_template());

        let template = mustache::compile_str(self.1.contents)?;
        let data = self.merge_data(data)?;
        let out_path = HYPRSHADE_RUNTIME_DIR
            .to_owned()
            .join(format!("{}.glsl", self.0));
        fs::create_dir_all(out_path.parent().unwrap())?;
        let mut out_file = File::create(&out_path)?;
        template.render_data(&mut out_file, &data)?;

        Ok(out_path)
    }

    fn merge_data(&self, data: &mustache::Data) -> eyre::Result<mustache::Data> {
        debug_assert!(matches!(data, mustache::Data::Map(_)));

        let mut merged_data = self.data();

        todo!();

        Ok(merged_data)
    }

    fn data(&self) -> mustache::Data {
        let variables = &self.1.metadata.variables;
        let map = variables.into_iter().fold(
            HashMap::with_capacity(variables.len()),
            |mut map, (key, value)| {
                map.insert(key.to_string(), mustache::Data::from(value));
                map
            },
        );
        mustache::Data::Map(map)
    }
}

impl PartialEq for BuiltinShader<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0) && std::ptr::eq(self.1, other.1)
    }
}
impl Eq for BuiltinShader<'_> {}

impl From<&Variable> for mustache::Data {
    fn from(value: &Variable) -> Self {
        match value {
            // TODO: does f32::to_string() render differently based on locale?
            Variable::Float { default, .. } => mustache::Data::String(default.to_string()),
            Variable::Enum { default, .. } => {
                mustache::Data::String(default.to_string().to_uppercase())
            }
            Variable::Dict(map) => mustache::Data::Map(map.into_iter().fold(
                HashMap::with_capacity(map.len()),
                |mut map, (key, value)| {
                    map.insert(key.to_string(), mustache::Data::from(value));
                    map
                },
            )),
        }
    }
}

pub const BUILTIN_SHADERS: BuiltinShaders = BuiltinShaders(phf_map! {
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
                "balance" => Variable::Dict(phf_map! {
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
    fn get_entry_pointer_equality() {
        let expected = BUILTIN_SHADERS.0.get_entry(b"blue-light-filter").unwrap();
        let actual = BUILTIN_SHADERS.get_entry("blue-light-filter").unwrap();
        assert!(std::ptr::eq(
            *expected.0,
            actual.0 as *const str as *const [u8]
        ));
        assert!(std::ptr::eq(expected.1, actual.1));
    }
}
