use super::any::{AnyControl, IntoAny};
use crate::core::render::Render;
use frunk::{HCons, HNil};

pub trait ChildrenGatherer {
    fn gather(self) -> Vec<AnyControl>;
    fn gather_json(self) -> Vec<String>;
}

impl ChildrenGatherer for HNil {
    fn gather(self) -> Vec<AnyControl> {
        Vec::new()
    }

    fn gather_json(self) -> Vec<String> {
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

    fn gather_json(self) -> Vec<String> {
        let mut vec = self.tail.gather_json();
        vec.push(self.head.to_json());
        vec
    }
}
