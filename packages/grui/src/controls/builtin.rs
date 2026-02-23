use crate::{
    core::renderer::Render,
    godot::ty::GDType,
    prelude::visitors::{ChildrenGatherer, PropsGatherer, SignalsGatherer},
};
use frunk::hlist::HList;
use godot::{classes::Control, obj::Gd};

pub(crate) struct Builtin<Pp, Sg, Ch> {
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
    fn mount(self, mut parent: Gd<Control>) {
        let mut gd = self.ty.create_instance();
        self.props.set_props(gd.clone());
        let signals = self.signals.gather_signals();
        for (signal, method) in &signals {
            gd.connect(signal, method);
        }
        self.children.mount(gd.clone());
        parent.add_child(&gd);
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
