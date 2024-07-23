use phf::phf_map;

pub struct BuiltinShaders(phf::Map<&'static [u8], BuiltinShader>);

#[derive(Debug)]
pub struct BuiltinShader {
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
    Float { default: f32, min: f32, max: f32 },
    Enum(&'static [&'static str]),
    Dict(phf::Map<&'static str, Variable>),
}

impl BuiltinShaders {
    pub fn get<K>(&self, key: K) -> Option<&BuiltinShader>
    where
        K: AsRef<[u8]>,
    {
        self.0.get(key.as_ref())
    }
}

impl BuiltinShader {
    pub fn is_template(&self) -> bool {
        self.is_template
    }
}

pub const BUILTIN_SHADERS: BuiltinShaders = BuiltinShaders(phf_map! {
    b"blue-light-filter" => BuiltinShader {
        contents: include_str!("shaders/blue-light-filter.glsl.mustache"),
        is_template: true,
        metadata: Metadata {
            full_name: "Blue Light Filter",
            description: "Use warmer colors to make the display easier on your eyes.",
            variables: phf_map! {
                "temperature" => Variable::Float {
                    default: 2600.0,
                    min: 1000.0,
                    max: 40000.0,
                },
                "strength" => Variable::Float {
                    default: 1.0,
                    min: 0.0,
                    max: 1.0,
                },
            },
        },
    },
    b"color-filter" => BuiltinShader {
        contents: include_str!("shaders/color-filter.glsl.mustache"),
        is_template: true,
        metadata: Metadata {
            full_name: "Color Filter",
            description: "\
                Adjust colors for color vision deficiencies.\n\
                Supports protanopia (red-green), deuteranopia (green-red), and tritanopia (blue-yellow).\
            ",
            variables: phf_map! {
                "type" => Variable::Enum(&[
                    "protanopia",
                    "protan",
                    "redgreen",
                    "deuteranopia",
                    "deutan",
                    "greenred",
                    "tritanopia",
                    "tritan",
                    "blueyellow",
                ]),
                "strength" => Variable::Float {
                    default: 0.2,
                    min: 0.0,
                    max: 1.0,
                },
            },
        },
    },
    b"grayscale" => BuiltinShader {
        contents: include_str!("shaders/grayscale.glsl.mustache"),
        is_template: true,
        metadata: Metadata {
            full_name: "Grayscale",
            description: "Use grayscale filter",
            variables: phf_map! {
                "type" => Variable::Enum(&["luminosity", "lightness", "average"]),
                "luminosity_type" => Variable::Enum(&["pal", "hdtv", "hdr"]),
            },
        },
    },
    b"invert-colors" => BuiltinShader {
        contents: include_str!("shaders/invert-colors.glsl"),
        is_template: false,
        metadata: Metadata {
            full_name: "Invert Colors",
            description: "Invert colors so text and content stand out.",
            variables: phf_map! {},
        },
    },
    b"vibrance" => BuiltinShader {
        contents: include_str!("shaders/vibrance.glsl.mustache"),
        is_template: true,
        metadata: Metadata {
            full_name: "Vibrance",
            description: "Enhance color saturation.",
            variables: phf_map! {
                "balance" => Variable::Dict(phf_map! {
                    "red" => Variable::Float {
                        default: 1.0,
                        min: 0.0,
                        max: 10.0,
                    },
                    "green" => Variable::Float {
                        default: 1.0,
                        min: 0.0,
                        max: 10.0,
                    },
                    "blue" => Variable::Float {
                        default: 1.0,
                        min: 0.0,
                        max: 10.0,
                    },
                }),
                "strength" => Variable::Float {
                    default: 0.15,
                    min: -1.0,
                    max: 1.0,
                },
            },
        },
    },
});
