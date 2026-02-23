use crate::{core::renderer::Render, prelude::SignalCallable};
use frunk::{HCons, HNil};
use godot::{builtin::Callable, classes::Control, meta::ToGodot, obj::Gd};
use reactive_graph::effect::Effect;
use std::{collections::HashMap, fmt::Debug};

pub trait ChildrenGatherer {
    fn mount(self, parent: Gd<Control>);
    fn gather_json(self) -> Vec<String>;
}

impl ChildrenGatherer for HNil {
    fn mount(self, _parent: Gd<Control>) {}

    fn gather_json(self) -> Vec<String> {
        Vec::new()
    }
}

impl<Head, Tail> ChildrenGatherer for HCons<Head, Tail>
where
    Head: Render,
    Tail: ChildrenGatherer,
{
    fn mount(self, parent: Gd<Control>) {
        self.tail.mount(parent.clone());
        self.head.mount(parent);
    }

    fn gather_json(self) -> Vec<String> {
        let mut vec = self.tail.gather_json();
        vec.push(self.head.to_json());
        vec
    }
}

pub trait PropsGatherer {
    fn set_props(self, gd: Gd<Control>);
    fn gather_json(self) -> HashMap<String, String>;
}

impl PropsGatherer for HNil {
    fn set_props(self, _gd: Gd<Control>) {}

    fn gather_json(self) -> HashMap<String, String> {
        HashMap::new()
    }
}

impl<VF, V, Tail> PropsGatherer for HCons<(String, VF), Tail>
where
    VF: Fn() -> V + 'static,
    V: Debug + ToGodot,
    Tail: PropsGatherer,
{
    fn set_props(self, gd: Gd<Control>) {
        {
            let mut gd = gd.clone();
            Effect::new(move || {
                let value = (self.head.1)().to_variant();
                gd.set(&self.head.0, &value);
            });
        }
        self.tail.set_props(gd);
    }

    fn gather_json(self) -> HashMap<String, String> {
        let mut map = self.tail.gather_json();
        map.insert(self.head.0.to_string(), format!("{:?}", self.head.1()));
        map
    }
}

pub trait SignalsGatherer {
    fn gather_signals(self) -> HashMap<String, Callable>;
    fn gather_json(self) -> Vec<String>;
}

impl SignalsGatherer for HNil {
    fn gather_signals(self) -> HashMap<String, Callable> {
        HashMap::new()
    }

    fn gather_json(self) -> Vec<String> {
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

    fn gather_json(self) -> Vec<String> {
        let mut vec = self.tail.gather_json();
        vec.push(self.head.0.to_string());
        vec
    }
}
