use super::{builtin::StateGD, IntoControl};
use crate::{
    core::render::{BuildOptions, MountPlace, Mountable, Node, Render},
    godot::ty::GDType,
};

impl Render for () {
    type State = StateGD;

    fn build(self, opts: &BuildOptions) -> Self::State {
        StateGD {
            node: Node::new(GDType::Control, &opts.graph),
            props: vec![],
            children: vec![],
        }
    }

    fn rebuild(self, _state: &mut Self::State, _opts: &BuildOptions) {}
}

impl Mountable for () {
    fn mount(&mut self, _place: MountPlace) {}

    fn mount_after(&mut self, _sibling: &mut dyn Mountable) {}

    fn unmount(&mut self) {}
}

impl<T: Render> Render for Vec<T> {
    type State = Vec<T::State>;

    fn build(self, opts: &BuildOptions) -> Self::State {
        self.into_iter()
            .map(|child| child.build(opts))
            .collect::<Vec<_>>()
    }

    fn rebuild(self, state: &mut Self::State, opts: &BuildOptions) {
        for (child, state) in self.into_iter().zip(state.iter_mut()) {
            child.rebuild(state, opts);
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
