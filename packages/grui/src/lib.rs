pub mod classes;
mod control;
pub mod events;
mod godot;
pub mod node;
pub mod renderer;
mod utils;
pub mod reactive;

extern crate grui_macros;

pub mod prelude {
    pub use crate::classes;
    pub use crate::control::Component;
    pub use crate::events;
    pub use crate::node::{
        Children, ElementBuilder, ElementNode, EventDescriptor, IntoChildren, IntoControl,
        IntoNode, Node, PropertyValue,
    };
    pub use crate::reactive::{signal, for_each, ReadSignal, WriteSignal, Effect};
    pub use crate::renderer::Renderer;
    pub use grui_macros::*;

    /// Type alias for Node, used as the return type for components
    pub type Control = Node;
}
