use std::collections::HashMap;

use godot::{classes::Control, obj::NewAlloc};

use super::{builtin::StateGD, IntoControl};
use crate::core::render::{MountPlace, Mountable, Render, TestSnapshot};

impl Render for () {
    type State = StateGD;

    fn build(self) -> Self::State {
        StateGD {
            node: Control::new_alloc(),
            props: vec![],
            children: vec![],
        }
    }

    fn rebuild(self, _state: &mut Self::State) {}

    fn get_test_snapshot(&self) -> TestSnapshot {
        TestSnapshot {
            json: "null".to_string(),
            actions: HashMap::new(),
        }
    }
}

impl Mountable for () {
    fn mount(&mut self, _place: MountPlace) {}

    fn mount_after(&mut self, _sibling: &mut dyn Mountable) {}

    fn unmount(&mut self) {}
}

impl<T: Render> Render for Vec<T> {
    type State = Vec<T::State>;

    fn build(self) -> Self::State {
        self.into_iter()
            .map(|child| child.build())
            .collect::<Vec<_>>()
    }

    fn rebuild(self, state: &mut Self::State) {
        for (child, state) in self.into_iter().zip(state.iter_mut()) {
            child.rebuild(state);
        }
    }

    fn get_test_snapshot(&self) -> TestSnapshot {
        let parts: Vec<TestSnapshot> = self
            .iter()
            .enumerate()
            .map(|(i, child)| child.get_test_snapshot().prefix_action(&i.to_string()))
            .collect();
        TestSnapshot {
            json: format!(
                "{}",
                parts
                    .iter()
                    .map(|s| s.json.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            actions: TestSnapshot::new().merge_actions(parts).actions,
        }
    }
}

impl<T: Mountable> Mountable for Vec<T> {
    fn mount(&mut self, place: MountPlace) {
        match &place {
            MountPlace::AppendToParent(_) => {
                for gd in self {
                    gd.mount(place.clone());
                }
            }
            MountPlace::AfterSibling(_) => {
                for gd in self.into_iter().rev() {
                    gd.mount(place.clone());
                }
            }
        }
    }

    fn mount_after(&mut self, sibling: &mut dyn Mountable) {
        for gd in self.into_iter().rev() {
            gd.mount_after(sibling);
        }
    }

    fn unmount(&mut self) {
        for gd in self {
            gd.unmount();
        }
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
