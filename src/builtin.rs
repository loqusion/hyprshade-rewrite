use phf::phf_map;

#[derive(Debug)]
pub struct BuiltinShader {
    pub contents: &'static str,
    pub is_template: bool,
}

pub const BUILTIN_SHADERS: phf::Map<&'static str, BuiltinShader> = phf_map! {
    "blue-light-filter" => BuiltinShader {
        contents: include_str!("shaders/blue-light-filter.glsl.mustache"),
        is_template: true,
    },
    "color-filter" => BuiltinShader {
        contents: include_str!("shaders/color-filter.glsl.mustache"),
        is_template: true,
    },
    "grayscale" => BuiltinShader {
        contents: include_str!("shaders/grayscale.glsl.mustache"),
        is_template: true,
    },
    "invert-colors" => BuiltinShader {
        contents: include_str!("shaders/invert-colors.glsl"),
        is_template: false,
    },
    "vibrance" => BuiltinShader {
        contents: include_str!("shaders/vibrance.glsl.mustache"),
        is_template: true,
    },
};
