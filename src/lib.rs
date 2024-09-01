//! **WARNING:** This library crate is not meant to be used directly.
//! Breaking changes may occur in any of the exported items without warning.

#![cfg(feature = "_lib")]
#![allow(dead_code)]

mod builtin;
mod constants;
mod dirs;
mod hyprctl;
mod resolver;
mod shader;
mod template;
mod util;

#[doc(hidden)]
pub mod __private {
    pub use crate::shader::{Shader, ShaderInstance};
}
