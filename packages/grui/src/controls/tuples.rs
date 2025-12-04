use crate::renderer::Render;
use frunk::{HCons, HNil};
use godot::{
    builtin::{Callable, Variant},
    classes::Control,
    meta::ToGodot,
    obj::Gd,
};
use std::collections::HashMap;

// impl Render for HNil {
//     fn to_controls(self) -> Vec<Gd<Control>> {
//         EmptyControl {}.to_controls()
//     }

//     fn to_json(self) -> String {
//         EmptyControl {}.to_json()
//     }
// }

// impl<Head, Tail> Render for HCons<Head, Tail>
// where
//     Head: Render,
//     Tail: Render,
// {
//     fn to_controls(self) -> Vec<Gd<Control>> {
//         let head_controls = self.head.to_controls();
//         let tail_controls = self.tail.to_controls();
//         [head_controls, tail_controls].concat()
//     }

//     fn to_json(self) -> String {
//         let head_json = self.head.to_json();
//         let tail_json = self.tail.to_json();
//         format!("[{},{}]", head_json, tail_json)
//     }
// }

pub trait ChildrenGatherer {
    fn gather_controls(self) -> Vec<Gd<Control>>;
    fn gather_json(self) -> Vec<String>;
}

impl ChildrenGatherer for HNil {
    fn gather_controls(self) -> Vec<Gd<Control>> {
        Vec::new()
    }

    fn gather_json(self) -> Vec<String> {
        Vec::new()
    }
}

impl<Head, Tail> ChildrenGatherer for HCons<Head, Tail>
where
    Head: Render,
    Tail: ChildrenGatherer,
{
    fn gather_controls(self) -> Vec<Gd<Control>> {
        let mut vec = self.tail.gather_controls();
        vec.extend(self.head.to_controls());
        vec
    }

    fn gather_json(self) -> Vec<String> {
        let mut vec = self.tail.gather_json();
        vec.push(self.head.to_json());
        vec
    }
}

pub trait PropsGatherer {
    fn gather_props(self) -> HashMap<String, Variant>;
    fn gather_json(self) -> HashMap<String, String>;
}

impl PropsGatherer for HNil {
    fn gather_props(self) -> HashMap<String, Variant> {
        HashMap::new()
    }

    fn gather_json(self) -> HashMap<String, String> {
        HashMap::new()
    }
}

impl<V, Tail> PropsGatherer for HCons<(String, V), Tail>
where
    V: std::fmt::Debug + ToGodot,
    Tail: PropsGatherer,
{
    fn gather_props(self) -> HashMap<String, Variant> {
        let mut map = self.tail.gather_props();
        map.insert(self.head.0.to_string(), self.head.1.to_variant());
        map
    }

    fn gather_json(self) -> HashMap<String, String> {
        let mut map = self.tail.gather_json();
        map.insert(self.head.0.to_string(), format!("{:?}", self.head.1));
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

pub trait CompatibleFn: 'static + FnMut(&[&Variant]) -> Result<Variant, ()> {}

impl<T> CompatibleFn for T where T: 'static + FnMut(&[&Variant]) -> Result<Variant, ()> {}

impl<V, Tail> SignalsGatherer for HCons<(String, V), Tail>
where
    V: CompatibleFn,
    Tail: SignalsGatherer,
{
    fn gather_signals(self) -> HashMap<String, Callable> {
        let mut map = self.tail.gather_signals();
        map.insert(
            self.head.0.to_string(),
            Callable::from_local_fn(&format!("{}_handler", self.head.0), self.head.1),
        );
        map
    }

    fn gather_json(self) -> Vec<String> {
        let mut vec = self.tail.gather_json();
        vec.push(self.head.0.to_string());
        vec
    }
}
