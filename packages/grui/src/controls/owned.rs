use super::IntoControl;
use crate::core::render::{BuildOptions, Render};
use reactive_graph::owner::Owner;

pub(crate) struct OwnedControl<T> {
    inner: T,
    #[allow(dead_code)]
    owner: Owner,
}

impl<T> OwnedControl<T> {
    pub(crate) fn new_with_owner(control: T, owner: Owner) -> Self
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

    fn build(self, opts: &BuildOptions) -> Self::State {
        self.inner.build(opts)
    }

    fn rebuild(self, state: &mut Self::State, opts: &BuildOptions) {
        self.inner.rebuild(state, opts);
    }
}
