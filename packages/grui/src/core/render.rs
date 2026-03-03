use crate::controls::signals::SignalCallable;
use godot::{classes::Control, obj::Gd};
use std::collections::HashMap;

pub trait IntoRender {
    type Output;

    fn into_render(self) -> Self::Output;
}

impl<T: Render> IntoRender for T {
    type Output = Self;

    fn into_render(self) -> Self::Output {
        self
    }
}

pub struct TestSnapshot {
    pub json: String,
    pub actions: HashMap<String, SignalCallable>,
}

impl TestSnapshot {
    pub(crate) fn new() -> Self {
        Self {
            json: String::new(),
            actions: HashMap::new(),
        }
    }

    pub(crate) fn prefix_action(self, prefix: &str) -> Self {
        Self {
            actions: self
                .actions
                .into_iter()
                .map(|(key, value)| (format!("{}.{}", prefix, key), value))
                .collect(),
            json: self.json,
        }
    }

    // fn make_test_path(prefix: String, suffix: String) -> String {
    //     if prefix.is_empty() {
    //         suffix.to_string()
    //     } else {
    //         format!("{}.{}", prefix, suffix)
    //     }
    // }

    pub(crate) fn merge_actions(mut self, others: Vec<TestSnapshot>) -> Self {
        for other in others {
            self.actions.extend(other.actions);
        }
        self
    }
}

pub trait Render: Sized {
    type State: Mountable;

    fn build(self) -> Self::State;

    fn rebuild(self, state: &mut Self::State);

    fn get_test_snapshot(&self) -> TestSnapshot;
}

#[derive(Clone)]
pub enum MountPlace {
    AppendToParent(Gd<Control>),
    AfterSibling(Gd<Control>),
}

pub trait Mountable {
    fn mount(&mut self, place: MountPlace);

    fn mount_after(&mut self, sibling: &mut dyn Mountable);

    fn unmount(&mut self);
}
