use super::functions::SignalCallable;
use frunk::{HCons, HNil};
use godot::builtin::Callable;
use std::collections::HashMap;

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
