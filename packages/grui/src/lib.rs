pub(crate) mod components;
pub(crate) mod controls;
pub(crate) mod core;
pub(crate) mod godot;
mod tests;
pub(crate) mod utils;

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
    pub use crate::utils::logger;
    pub use typed_builder;
}

#[cfg(feature = "testing")]
pub mod testing {
    pub use crate::core::testing::TestRenderer;
}
