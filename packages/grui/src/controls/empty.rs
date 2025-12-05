use crate::{controls::IntoControl, renderer::Render};
use godot::{classes::Control, obj::Gd};

pub(crate) struct Empty;

pub fn empty() -> impl IntoControl {
    Empty {}
}

impl Render for Empty {
    fn to_controls(self) -> Vec<Gd<Control>> {
        Vec::new()
    }

    fn to_json(self) -> String {
        "null".to_string()
    }
}
