use crate::{controls::IntoControl, renderer::Render};
use godot::{classes::Control, obj::Gd};

impl<T: Render> Render for Vec<T> {
    fn to_controls(self) -> Vec<Gd<Control>> {
        self.into_iter()
            .flat_map(|child| child.to_controls())
            .collect()
    }

    fn to_json(self) -> String {
        let parts: Vec<String> = self.into_iter().map(|child| child.to_json()).collect();
        format!("[{}]", parts.join(", "))
    }
}

pub trait CollectControl {
    type Control: IntoControl;

    fn collect_control(self) -> Vec<Self::Control>;
}

impl<It, V> CollectControl for It
where
    It: IntoIterator<Item = V>,
    V: IntoControl,
{
    type Control = V;

    fn collect_control(self) -> Vec<Self::Control> {
        self.into_iter().collect()
    }
}
