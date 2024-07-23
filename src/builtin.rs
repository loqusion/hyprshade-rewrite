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
    pub fn get<K>(&self, key: &K) -> Option<&BuiltinShader>
    where
        K: AsRef<[u8]> + ?Sized,
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
    b"grayscale" => BuiltinShader {
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
