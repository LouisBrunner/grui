use crate::{controls::IntoControl, renderer::Render};
use godot::{classes::Control, obj::Gd};

pub(crate) struct EmptyControl;

pub fn empty() -> impl IntoControl {
    EmptyControl {}
}

impl Render for EmptyControl {
    fn to_controls(self) -> Vec<Gd<Control>> {
        Vec::new()
    }

    fn to_json(self) -> String {
        "null".to_string()
    }
}
