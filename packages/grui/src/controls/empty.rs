use crate::{controls::IntoControl, core::renderer::Render};
use godot::{classes::Control, obj::Gd};

pub(crate) struct Empty;

pub fn empty() -> impl IntoControl {
    Empty {}
}

impl Render for Empty {
    fn mount(self, _parent: Gd<Control>) {}

    fn to_json(self) -> String {
        "null".to_string()
    }
}
