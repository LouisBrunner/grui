use super::IntoControl;
use crate::core::render::Render;
use godot::builtin::{Callable, Variant};
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
    T: FnOnce() -> C,
    C: IntoControl,
{
    type State = C::State;

    fn build(self) -> Self::State {
        (self)().build()
    }

    fn rebuild(self, state: &mut Self::State) {
        (self)().rebuild(state);
    }

    fn to_json(self) -> String {
        (self)().to_json()
    }
}
