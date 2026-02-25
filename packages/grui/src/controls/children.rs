use crate::core::render::Render;
use frunk::{HCons, HNil};

pub trait ChildrenGatherer: Render {
    fn gather_json(self) -> Vec<String>;
}

impl Render for HNil {
    type State = ();

    fn build(self) -> Self::State {}

    fn rebuild(self, state: &mut Self::State) {}

    fn to_json(self) -> String {
        todo!()
    }
}

impl ChildrenGatherer for HNil {
    fn gather_json(self) -> Vec<String> {
        Vec::new()
    }
}

impl<Head, Tail> ChildrenGatherer for HCons<Head, Tail>
where
    Head: Render,
    Tail: ChildrenGatherer,
{
    // fn build(self) -> Vec<AnyState> {
    //     let mut children = self.tail.build();
    //     children.extend(self.head.into_any().build());
    //     children
    // }

    fn gather_json(self) -> Vec<String> {
        let mut vec = self.tail.gather_json();
        vec.push(self.head.to_json());
        vec
    }
}
