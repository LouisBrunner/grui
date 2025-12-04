use crate::renderer::Render;
use godot::{classes::Control, obj::Gd};

pub(crate) struct BuiltinControl;

impl Render for BuiltinControl {
    fn to_controls(self) -> Vec<Gd<Control>> {
        panic!("TODO: finish")
    }

    fn to_json(self) -> String {
        panic!("TODO: finish")
    }
}
