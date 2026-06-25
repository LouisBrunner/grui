use crate::core::render::Node;
use frunk::{HCons, HNil};
use godot::builtin::{Callable, Variant};
use std::fmt::Debug;

pub trait SignalsGatherer {
    fn attach(self, node: Node);
    fn get_debug(&self) -> Vec<String>;
}

impl SignalsGatherer for HNil {
    fn attach(self, _node: Node) {}

    fn get_debug(&self) -> Vec<String> {
        vec![]
    }
}

impl<Tail> SignalsGatherer for HCons<(String, SignalCallable), Tail>
where
    Tail: SignalsGatherer,
{
    fn attach(self, node: Node) {
        node.connect(self.head.0, self.head.1);
        self.tail.attach(node);
    }

    fn get_debug(&self) -> Vec<String> {
        let mut vec = self.tail.get_debug();
        vec.push(self.head.0.clone());
        vec
    }
}

pub trait CompatibleFn: 'static + FnMut(&[&Variant]) {}

impl<T> CompatibleFn for T where T: 'static + FnMut(&[&Variant]) {}

pub struct SignalCallable(Box<dyn CompatibleFn>);

impl SignalCallable {
    pub fn new<F>(func: F) -> Self
    where
        F: CompatibleFn,
    {
        Self(Box::new(func))
    }

    #[cfg(feature = "testing")]
    pub(crate) fn call(&mut self, args: &[&Variant]) {
        (self.0)(args);
    }

    pub(crate) fn into_godot(self, label: &str) -> Callable {
        let mut func = self.0;
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
