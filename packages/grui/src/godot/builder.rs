use crate::{
    controls::{BuiltinControl, IntoControl},
    godot::ty::GDType,
    prelude::tuples::{ChildrenGatherer, PropsGatherer, SignalsGatherer},
    renderer::IntoRender,
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

impl<Pp, Sg, Ch> GDClass<Pp, Sg, Ch>
where
    Pp: HList + PropsGatherer,
    Sg: HList + SignalsGatherer,
    Ch: HList + ChildrenGatherer,
{
    pub fn build(self) -> impl IntoControl {
        BuiltinControl::new(self.ty, self.props, self.signals, self.children)
    }
}

impl<Pp, Sg, Ch> GDClass<Pp, Sg, Ch>
where
    Pp: HList,
{
    pub fn prop<Value>(
        self,
        name: &str,
        value: Value,
    ) -> GDClass<HCons<(String, Value), Pp>, Sg, Ch>
    where
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
    pub fn on<Fn>(self, name: &str, func: Fn) -> GDClass<Pp, HCons<(String, Fn), Sg>, Ch> {
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
