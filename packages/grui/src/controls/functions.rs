use super::IntoControl;
use crate::core::render::{MountPlace, Mountable, Render};
use godot::builtin::{Callable, Variant};
use reactive_graph::effect::RenderEffect;
use std::fmt::Debug;

pub trait CompatibleFn: 'static + FnMut(&[&Variant]) -> () {}

impl<T> CompatibleFn for T where T: 'static + FnMut(&[&Variant]) -> () {}

pub struct SignalCallable {
    func: Box<dyn CompatibleFn>,
}

impl SignalCallable {
    pub fn new<F>(func: F) -> Self
    where
        F: CompatibleFn,
    {
        return Self {
            func: Box::new(func),
        };
    }

    pub fn to_godot(self, label: &str) -> Callable {
        let mut func = self.func;
        Callable::from_fn(&format!("{}_handler", label), move |args| {
            (func)(args);
        })
    }
}

impl Debug for SignalCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SignalCallable")
    }
}

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
