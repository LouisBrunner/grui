use crate::{
    godot::ty::GDType,
    prelude::tuples::{ChildrenGatherer, PropsGatherer, SignalsGatherer},
    renderer::Render,
};
use frunk::hlist::HList;
use godot::{classes::Control, obj::Gd};

pub(crate) struct BuiltinControl<Pp, Sg, Ch> {
    ty: GDType,
    props: Pp,
    signals: Sg,
    children: Ch,
}

impl<Pp, Sg, Ch> BuiltinControl<Pp, Sg, Ch> {
    pub fn new(ty: GDType, props: Pp, signals: Sg, children: Ch) -> Self {
        BuiltinControl {
            ty,
            props,
            signals,
            children,
        }
    }
}

impl<Pp, Sg, Ch> Render for BuiltinControl<Pp, Sg, Ch>
where
    Pp: HList + PropsGatherer,
    Sg: HList + SignalsGatherer,
    Ch: HList + ChildrenGatherer,
{
    fn to_controls(self) -> Vec<Gd<Control>> {
        let mut gd = self.ty.create_instance();
        let props = self.props.gather_props();
        for (key, value) in &props {
            gd.set(key, value);
        }
        let signals = self.signals.gather_signals();
        for (signal, method) in &signals {
            gd.connect(signal, method);
        }
        let children = self.children.gather_controls();
        for child in &children {
            gd.add_child(child);
        }
        vec![gd]
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
            json.push_str(&format!(r#", "children": {}"#, children.join(",")));
        }
        json.push('}');
        json
    }
}
