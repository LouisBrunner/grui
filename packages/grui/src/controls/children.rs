use super::any::{AnyControl, IntoAny};
use crate::core::render::Render;
use frunk::{HCons, HNil};
use std::{fmt::Debug, ops::Deref, sync::Arc};

pub trait ChildrenGatherer {
    fn gather(self) -> Vec<AnyControl>;
}

impl ChildrenGatherer for HNil {
    fn gather(self) -> Vec<AnyControl> {
        Vec::new()
    }
}

impl<Head, Tail> ChildrenGatherer for HCons<Head, Tail>
where
    Head: Render + 'static,
    Tail: ChildrenGatherer,
{
    fn gather(self) -> Vec<AnyControl> {
        let mut children = self.tail.gather();
        children.push(self.head.into_any());
        children
    }
}

pub trait ToChildren<F> {
    fn to_children(f: F) -> Self;
}

pub struct ChildrenFn(Arc<dyn Fn() -> AnyControl>);

impl<F, C> ToChildren<F> for ChildrenFn
where
    F: Fn() -> C + 'static,
    C: Render + 'static,
{
    fn to_children(f: F) -> Self {
        ChildrenFn(Arc::new(move || f().into_any()))
    }
}

impl Deref for ChildrenFn {
    type Target = Arc<dyn Fn() -> AnyControl>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for ChildrenFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChildrenFn")
    }
}
