use super::{children::ChildrenGatherer, props::PropsGatherer, signals::SignalsGatherer};
use crate::{
    core::render::{Mountable, Render},
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
    type State = StateGD<Ch::State>;

    fn build(self) -> Self::State {
        let mut gd = self.ty.create_instance();
        let props = self.props.attach(gd.clone());
        let signals = self.signals.gather_signals();
        for (signal, method) in &signals {
            gd.connect(signal, method);
        }
        let mut children = self.children.build();
        children.mount(&gd);
        StateGD {
            node: gd,
            props,
            children,
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        self.children.rebuild(&mut state.children);
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

pub struct StateGD<Ch> {
    node: Gd<Control>,
    props: Vec<RenderEffect<()>>,
    children: Ch,
}

impl<Ch> Mountable for StateGD<Ch> {
    fn mount(&mut self, parent: &Gd<Control>) {
        parent.clone().add_child(&self.node);
    }

    fn unmount(&mut self) {
        self.node.queue_free();
    }
}
