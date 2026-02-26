use super::IntoControl;
use crate::core::render::{MountPlace, Mountable, Render};

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

    fn to_json(self) -> String {
        let parts: Vec<String> = self.into_iter().map(|child| child.to_json()).collect();
        format!("{}", parts.join(", "))
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
