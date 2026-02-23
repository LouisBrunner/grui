use crate::{controls::IntoControl, core::renderer::Render};
use godot::{classes::Control, obj::Gd};

impl<T: Render> Render for Vec<T> {
    fn mount(self, parent: Gd<Control>) {
        for child in self.into_iter() {
            child.mount(parent.clone());
        }
    }

    fn to_json(self) -> String {
        let parts: Vec<String> = self.into_iter().map(|child| child.to_json()).collect();
        format!("{}", parts.join(", "))
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
