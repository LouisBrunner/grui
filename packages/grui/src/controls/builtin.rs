use super::{
    any::AnyState, children::ChildrenGatherer, props::PropsGatherer, signals::SignalsGatherer,
};
use crate::{
    core::render::{BuildOptions, MountPlace, Mountable, Node, Render},
    godot::ty::GDType,
};
use frunk::hlist::HList;
use reactive_graph::effect::RenderEffect;

pub(crate) struct Builtin<Pp, Sg, Ch> {
    ty: GDType,
    props: Pp,
    signals: Sg,
    children: Ch,
}

impl<Pp, Sg, Ch> Builtin<Pp, Sg, Ch> {
    pub(crate) fn new(ty: GDType, props: Pp, signals: Sg, children: Ch) -> Self {
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

    fn build(self, opts: &BuildOptions) -> Self::State {
        let gd = Node::new(self.ty.clone(), opts.test);
        log::trace!(
            "instancing {} ({:?} / {:?}) = {}",
            self.ty,
            self.props.get_debug(),
            self.signals.get_debug(),
            gd.get_id()
        );
        let props = self.props.attach(gd.clone(), &gd.get_properties());
        self.signals.attach(gd.clone());
        let mut children = self.children.gather().build(opts);
        children.mount(MountPlace::AppendToParent(gd.clone()));
        StateGD {
            node: gd,
            props,
            children,
        }
    }

    fn rebuild(self, state: &mut Self::State, opts: &BuildOptions) {
        log::trace!(
            "rebuilding {} ({:?} / {:?}) = {}",
            self.ty,
            self.props.get_debug(),
            self.signals.get_debug(),
            state.node.get_id()
        );
        self.children.gather().rebuild(&mut state.children, opts);
    }
}

pub struct StateGD {
    pub(super) node: Node,
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
