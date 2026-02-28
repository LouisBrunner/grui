use frunk::{HCons, HNil};
use godot::builtin::{Callable, Variant};
use std::{collections::HashMap, fmt::Debug};

pub trait SignalsGatherer {
    fn gather_signals(self) -> HashMap<String, Callable>;
    fn gather_json(&self) -> Vec<String>;
}

impl SignalsGatherer for HNil {
    fn gather_signals(self) -> HashMap<String, Callable> {
        HashMap::new()
    }

    fn gather_json(&self) -> Vec<String> {
        Vec::new()
    }
}

impl<Tail> SignalsGatherer for HCons<(String, SignalCallable), Tail>
where
    Tail: SignalsGatherer,
{
    fn gather_signals(self) -> HashMap<String, Callable> {
        let mut map = self.tail.gather_signals();
        map.insert(self.head.0.to_string(), self.head.1.to_godot(&self.head.0));
        map
    }

    fn gather_json(&self) -> Vec<String> {
        let mut vec = self.tail.gather_json();
        vec.push(self.head.0.to_string());
        vec
    }
}

pub trait CompatibleFn: 'static + FnMut(&[&Variant]) -> () {}

impl<T> CompatibleFn for T where T: 'static + FnMut(&[&Variant]) -> () {}

pub struct SignalCallable(Box<dyn CompatibleFn>);

impl SignalCallable {
    pub fn new<F>(func: F) -> Self
    where
        F: CompatibleFn,
    {
        Self(Box::new(func))
    }

    pub fn to_godot(self, label: &str) -> Callable {
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
