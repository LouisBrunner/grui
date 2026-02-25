use super::IntoControl;
use crate::core::render::Render;
use reactive_graph::owner::Owner;

pub struct OwnedControl<T> {
    inner: T,
    #[allow(dead_code)]
    owner: Owner,
}

impl<T> OwnedControl<T> {
    pub fn new_with_owner(control: T, owner: Owner) -> Self
    where
        T: IntoControl,
    {
        OwnedControl {
            inner: control,
            owner,
        }
    }
}

impl<T> Render for OwnedControl<T>
where
    T: Render,
{
    type State = T::State;

    fn build(self) -> Self::State {
        self.inner.build()
    }

    fn rebuild(self, state: &mut Self::State) {
        self.inner.rebuild(state);
    }

    fn to_json(self) -> String {
        self.inner.to_json()
    }
}
