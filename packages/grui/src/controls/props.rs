use frunk::{HCons, HNil};
use godot::{classes::Control, meta::ToGodot, obj::Gd};
use reactive_graph::effect::RenderEffect;
use std::{collections::HashMap, fmt::Debug};

use crate::prelude::builtin::get_id_for_gd;

pub trait PropsGatherer {
    fn attach(self, gd: Gd<Control>) -> Vec<RenderEffect<()>>;
    fn gather_json(&self) -> HashMap<String, String>;
}

impl PropsGatherer for HNil {
    fn attach(self, _gd: Gd<Control>) -> Vec<RenderEffect<()>> {
        vec![]
    }

    fn gather_json(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

impl<VF, V, Tail> PropsGatherer for HCons<(String, VF), Tail>
where
    VF: Fn() -> V + 'static,
    V: Debug + ToGodot,
    Tail: PropsGatherer,
{
    fn attach(self, mut gd: Gd<Control>) -> Vec<RenderEffect<()>> {
        let mut props = self.tail.attach(gd.clone());
        let new_prop = {
            RenderEffect::new(move |prev| {
                let value = (self.head.1)().to_variant();
                if prev.is_some() {
                    log::trace!(
                        "updating prop {} to {} for {}",
                        self.head.0,
                        value,
                        get_id_for_gd(&gd)
                    );
                }
                gd.set(&self.head.0, &value);
            })
        };
        props.push(new_prop);
        props
    }

    fn gather_json(&self) -> HashMap<String, String> {
        let mut map = self.tail.gather_json();
        map.insert(self.head.0.to_string(), format!("{:?}", self.head.1()));
        map
    }
}
