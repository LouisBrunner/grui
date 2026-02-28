use super::{
    any::{AnyControl, IntoAny},
    IntoControl,
};
use crate::core::render::{MountPlace, Mountable, Render};
use reactive_graph::effect::RenderEffect;
use std::fmt::Debug;

impl<T, C> Render for T
where
    T: FnMut() -> C + 'static,
    C: IntoControl,
    C::State: 'static,
{
    type State = RenderEffect<C::State>;

    fn build(mut self) -> Self::State {
        RenderEffect::new(move |prev| {
            let value = (self)();
            if let Some(mut state) = prev {
                value.rebuild(&mut state);
                state
            } else {
                value.build()
            }
        })
    }

    fn rebuild(self, state: &mut Self::State) {
        let new = self.build();
        let mut old = std::mem::replace(state, new);
        old.mount_after(state);
        old.unmount();
    }

    fn to_json(mut self) -> String {
        (self)().to_json()
    }
}

impl<T> Mountable for RenderEffect<T>
where
    T: Mountable,
{
    fn mount(&mut self, place: MountPlace) {
        self.with_value_mut(|state| {
            state.mount(place);
        });
    }

    fn mount_after(&mut self, sibling: &mut dyn Mountable) {
        self.with_value_mut(|value| value.mount_after(sibling));
    }

    fn unmount(&mut self) {
        self.with_value_mut(|state| state.unmount());
    }
}

pub struct ControlFn(Box<dyn Fn() -> AnyControl + 'static>);

impl<F, C> From<F> for ControlFn
where
    F: Fn() -> C + 'static,
    C: Render + 'static,
{
    fn from(value: F) -> Self {
        Self(Box::new(move || value().into_any()))
    }
}

impl ControlFn {
    pub fn run(&self) -> AnyControl {
        (self.0)()
    }
}

impl Debug for ControlFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ControlFn")
    }
}
