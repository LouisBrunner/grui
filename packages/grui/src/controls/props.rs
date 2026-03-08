use crate::{core::render::Node, utils::errors::debug_error};
use frunk::{HCons, HNil};
use godot::meta::ToGodot;
use reactive_graph::effect::RenderEffect;
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

pub trait PropsGatherer {
    fn attach(self, node: Node, properties: &Option<HashSet<String>>) -> Vec<RenderEffect<()>>;
    fn get_debug(&self) -> HashMap<String, String>;
}

impl PropsGatherer for HNil {
    fn attach(self, _node: Node, _properties: &Option<HashSet<String>>) -> Vec<RenderEffect<()>> {
        vec![]
    }

    fn get_debug(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

impl<VF, V, Tail> PropsGatherer for HCons<(String, VF), Tail>
where
    VF: Fn() -> V + 'static,
    V: Debug + ToGodot,
    Tail: PropsGatherer,
{
    fn attach(self, node: Node, properties: &Option<HashSet<String>>) -> Vec<RenderEffect<()>> {
        let mut props = self.tail.attach(node.clone(), properties);
        let key = self.head.0;
        let new_prop = {
            debug_error!(
                match properties {
                    None => true,
                    Some(properties) => properties.contains(&key) || key.contains("/"), // i.e. it's a theme override
                },
                "Godot class {} doesn't support property {}, supported: {}",
                node.get_class(),
                key,
                {
                    match properties {
                        None => "none".to_string(),
                        Some(properties) => {
                            let mut props = properties
                                .iter()
                                .map(|s| format!("\"{}\"", s))
                                .collect::<Vec<_>>();
                            props.sort();
                            props.join(", ")
                        }
                    }
                }
            );

            RenderEffect::new(move |prev| {
                let value = node.set(&key, &self.head.1);
                if prev.is_some() {
                    log::trace!("updating prop {} to {} for {}", key, value, node.get_id(),);
                }
            })
        };
        props.push(new_prop);
        props
    }

    fn get_debug(&self) -> HashMap<String, String> {
        let mut map = self.tail.get_debug();
        map.insert(self.head.0.clone(), serialize_prop(&self.head.1));
        map
    }
}

pub(crate) fn serialize_prop<VF, V>(value: &VF) -> String
where
    VF: Fn() -> V + 'static,
    V: Debug + ToGodot,
{
    let value = value();
    format!("{:?}", value)
}
