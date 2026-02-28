use super::{
    any::AnyState, children::ChildrenGatherer, props::PropsGatherer, signals::SignalsGatherer,
};
use crate::{
    core::render::{MountPlace, Mountable, Render},
    godot::ty::GDType,
};
use frunk::hlist::HList;
use godot::{
    classes::Control,
    global::PropertyUsageFlags,
    obj::{EngineBitfield, Gd},
};
use reactive_graph::effect::RenderEffect;
use std::collections::HashSet;

pub struct Builtin<Pp, Sg, Ch> {
    ty: GDType,
    props: Pp,
    signals: Sg,
    children: Ch,
}

impl<Pp, Sg, Ch> Builtin<Pp, Sg, Ch> {
    pub fn new(ty: GDType, props: Pp, signals: Sg, children: Ch) -> Self {
        Builtin {
            ty,
            props,
            signals,
            children,
        }
    }
}

impl<Pp, Sg, Ch> Render for Builtin<Pp, Sg, Ch>
where
    Pp: HList + PropsGatherer,
    Sg: HList + SignalsGatherer,
    Ch: HList + ChildrenGatherer,
{
    type State = StateGD;

    fn build(self) -> Self::State {
        let mut gd = self.ty.create_instance();
        log::trace!(
            "instancing {} ({:?} / {:?}) = {}",
            self.ty,
            self.props.gather_json(),
            self.signals.gather_json(),
            get_id_for_gd(&gd)
        );
        let props = self.props.attach(gd.clone(), &get_properties_for(&gd));
        let signals = self.signals.gather_signals();
        for (signal, method) in &signals {
            gd.connect(signal, method);
        }
        let mut children = self.children.gather().build();
        children.mount(MountPlace::AppendToParent(gd.clone()));
        StateGD {
            node: gd,
            props,
            children,
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        log::trace!(
            "rebuilding {} ({:?} / {:?}) = {}",
            self.ty,
            self.props.gather_json(),
            self.signals.gather_json(),
            get_id_for_gd(&state.node)
        );
        self.children.gather().rebuild(&mut state.children);
    }

    fn to_json(self) -> String {
        let mut json = format!(r#"{{"type": "{}""#, self.ty);
        let props = self.props.gather_json();
        if !props.is_empty() {
            json.push_str(&format!(
                r#", "props": {{{}}}"#,
                props
                    .iter()
                    .map(|(k, v)| format!(r#""{}": {}"#, k, v))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        let signals = self.signals.gather_json();
        if !signals.is_empty() {
            json.push_str(&format!(
                r#", "signals": [{}]"#,
                signals
                    .iter()
                    .map(|k| format!(r#""{}""#, k))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        let children = self.children.gather_json();
        if !children.is_empty() {
            json.push_str(&format!(r#", "children": [{}]"#, children.join(", ")));
        }
        json.push('}');
        json
    }
}

pub struct StateGD {
    pub(super) node: Gd<Control>,
    #[allow(dead_code)]
    pub(super) props: Vec<RenderEffect<()>>,
    pub(super) children: Vec<AnyState>,
}

impl Mountable for StateGD {
    fn mount(&mut self, place: MountPlace) {
        self.node.mount(place);
    }

    fn mount_after(&mut self, sibling: &mut dyn Mountable) {
        self.node.mount_after(sibling);
    }

    fn unmount(&mut self) {
        self.node.unmount();
        for child in &mut self.children {
            child.unmount();
        }
    }
}

impl Mountable for Gd<Control> {
    fn mount(&mut self, place: MountPlace) {
        match place {
            MountPlace::AppendToParent(mut parent) => {
                log::trace!(
                    "mounting {} to parent {}",
                    get_id_for_gd(self),
                    get_id_for_gd(&parent)
                );
                parent.add_child(&self.clone());
            }
            MountPlace::AfterSibling(mut sibling) => {
                log::trace!(
                    "mounting {} after sibling {}",
                    get_id_for_gd(self),
                    get_id_for_gd(&sibling)
                );
                if sibling.get_parent().is_none() {
                    log::error!(
                        "Cannot mount {} after sibling {} without parent",
                        get_id_for_gd(self),
                        get_id_for_gd(&sibling)
                    );
                }
                sibling.add_sibling(&self.clone());
            }
        }
    }

    fn mount_after(&mut self, sibling: &mut dyn Mountable) {
        log::trace!("mounting after sibling {}", get_id_for_gd(self));
        sibling.mount(MountPlace::AfterSibling(self.clone()));
    }

    fn unmount(&mut self) {
        log::trace!("unmounting {}", get_id_for_gd(self));
        self.queue_free();
    }
}

pub(crate) fn get_id_for_gd(node: &Gd<Control>) -> String {
    let mut instance_id = "unknown".to_string();
    if node.is_instance_valid() {
        instance_id = node.instance_id().to_string();
    }
    let prefix = format!("{}#{}", node.get_class(), instance_id);
    if !node.get_name().is_empty() {
        format!("{}+{}", prefix, node.get_name())
    } else {
        prefix
    }
}

fn get_properties_for(node: &Gd<Control>) -> HashSet<String> {
    node.get_property_list()
        .iter_shared()
        .filter_map(|property| {
            let name = property.get("name")?;
            let usage = property.get("usage")?.try_to::<PropertyUsageFlags>().ok()?;
            if !usage.is_set(PropertyUsageFlags::STORAGE) {
                return None;
            }
            Some(name.to_string())
        })
        .collect::<HashSet<_>>()
}
