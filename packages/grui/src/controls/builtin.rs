use super::{
    any::AnyState, children::ChildrenGatherer, props::PropsGatherer, signals::SignalsGatherer,
};
use crate::{
    core::render::{MountPlace, Mountable, Render},
    godot::ty::GDType,
};
use frunk::hlist::HList;
use godot::{classes::Control, obj::Gd};
use reactive_graph::effect::RenderEffect;

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
        log::trace!("instancing {}", self.ty);
        let mut gd = self.ty.create_instance();
        let props = self.props.attach(gd.clone());
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
    node: Gd<Control>,
    #[allow(dead_code)]
    props: Vec<RenderEffect<()>>,
    children: Vec<AnyState>,
}

impl Mountable for StateGD {
    fn mount(&mut self, place: MountPlace) {
        match place {
            MountPlace::AppendToParent(mut parent) => {
                parent.add_child(&self.node);
            }
            MountPlace::AfterSibling(mut sibling) => {
                sibling.add_sibling(&self.node);
            }
        }
    }

    fn mount_after(&mut self, sibling: &mut dyn Mountable) {
        sibling.mount(MountPlace::AfterSibling(self.node.clone()));
    }

    // FIXME: needed?
    // MountRelative::Before => {
    //     let index = sibling.clone().get_index();
    //     sibling.add_sibling(&self.node);
    //     let Some(mut parent) = sibling.get_parent() else {
    //         return; // FIXME: !!!
    //     };
    //     parent.move_child(&self.node, index);
    // }

    fn unmount(&mut self) {
        self.node.queue_free();
        for child in &mut self.children {
            child.unmount();
        }
    }
}
