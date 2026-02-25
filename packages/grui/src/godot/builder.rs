use crate::{
    controls::{
        builtin::Builtin, children::ChildrenGatherer, props::PropsGatherer,
        signals::SignalsGatherer, IntoControl,
    },
    core::render::IntoRender,
    godot::ty::GDType,
};
use frunk::{hlist::HList, HCons, HNil};
use godot::meta::ToGodot;

pub struct GDClass<Pp, Sg, Ch> {
    ty: GDType,
    props: Pp,
    signals: Sg,
    children: Ch,
}

pub type GDClassBuilder = GDClass<HNil, HNil, HNil>;

impl GDClassBuilder {
    pub fn new(ty: GDType) -> Self {
        GDClass {
            ty,
            props: HNil,
            signals: HNil,
            children: HNil,
        }
    }
}

impl<Pp, Sg, Ch> GDClass<Pp, Sg, Ch> {
    pub fn build(self) -> impl IntoControl
    where
        Pp: HList + PropsGatherer,
        Sg: HList + SignalsGatherer,
        Ch: HList + ChildrenGatherer,
    {
        Builtin::new(self.ty, self.props, self.signals, self.children)
    }
}

impl<Pp, Sg, Ch> GDClass<Pp, Sg, Ch>
where
    Pp: HList,
{
    pub fn prop<FValue, Value>(
        self,
        name: &str,
        value: FValue,
    ) -> GDClass<HCons<(String, FValue), Pp>, Sg, Ch>
    where
        FValue: Fn() -> Value,
        Value: ToGodot,
    {
        GDClass {
            ty: self.ty,
            props: HCons {
                head: (name.to_string(), value),
                tail: self.props,
            },
            signals: self.signals,
            children: self.children,
        }
    }
}

impl<Pp, Sg, Ch> GDClass<Pp, Sg, Ch>
where
    Sg: HList,
{
    pub fn on<F>(self, name: &str, func: F) -> GDClass<Pp, HCons<(String, F), Sg>, Ch> {
        GDClass {
            ty: self.ty,
            props: self.props,
            signals: HCons {
                head: (name.to_string(), func),
                tail: self.signals,
            },
            children: self.children,
        }
    }
}

impl<Pp, Sg, Ch> GDClass<Pp, Sg, Ch>
where
    Ch: HList,
{
    pub fn child<NewChild>(self, child: NewChild) -> GDClass<Pp, Sg, HCons<NewChild::Output, Ch>>
    where
        NewChild: IntoRender,
    {
        GDClass {
            ty: self.ty,
            props: self.props,
            signals: self.signals,
            children: HCons {
                head: child.into_render(),
                tail: self.children,
            },
        }
    }
}
