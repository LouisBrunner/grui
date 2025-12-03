use crate::node::Node;

/// A trait implemented by stateful components that can render a virtual UI tree.
pub trait Component {
    fn render(&mut self) -> Node;
}

impl<T> Component for T
where
    T: FnMut() -> Node,
{
    fn render(&mut self) -> Node {
        self()
    }
}
