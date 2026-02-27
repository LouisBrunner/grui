pub(crate) mod components;
pub(crate) mod controls;
pub(crate) mod core;
pub(crate) mod godot;
mod tests;

extern crate self as grui;

pub mod prelude {
    pub use crate::components::*;
    pub use crate::controls::*;
    pub use crate::core::*;
    pub use crate::godot::classes::*;
    pub use grui_macros::*;
}

#[doc(hidden)]
pub mod internal {
    pub use typed_builder;
}

pub use crate::core::renderer::TestRenderer;
