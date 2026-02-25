use crate::core::render::{Mountable, Render};
use godot::{classes::Control, obj::Gd};

impl<T: Render> Render for Vec<T> {
    type State = StateVec<T::State>;

    fn build(self) -> Self::State {
        StateVec(self.into_iter().map(|child| child.build()).collect())
    }

    fn rebuild(self, state: &mut Self::State) {
        for (child, state) in self.into_iter().zip(state.0.iter_mut()) {
            child.rebuild(state);
        }
    }

    fn to_json(self) -> String {
        let parts: Vec<String> = self.into_iter().map(|child| child.to_json()).collect();
        format!("{}", parts.join(", "))
    }
}

pub struct StateVec<T: Mountable>(pub Vec<T>);

impl<T: Mountable> Mountable for StateVec<T> {
    fn mount(&mut self, parent: &Gd<Control>) {
        for gd in &mut self.0 {
            gd.mount(parent);
        }
    }

    fn unmount(&mut self) {
        for gd in &mut self.0 {
            gd.unmount();
        }
    }
}

// pub trait CollectControl {
//     type Control: IntoControl;

//     fn collect_control(self) -> Vec<Self::Control>;
// }

// impl<It, V> CollectControl for It
// where
//     It: IntoIterator<Item = V>,
//     V: IntoControl,
// {
//     type Control = V;

//     fn collect_control(self) -> Vec<Self::Control> {
//         self.into_iter().collect()
//     }
// }
