//! **WARNING:** This library crate is not meant to be used directly.
//! Breaking changes may occur in any of the exported items without warning.
#![doc(hidden)]
#![allow(dead_code)]

mod builtin;
mod constants;
mod hyprctl;
mod shader;
mod template;
mod util;

pub use shader::Shader;
