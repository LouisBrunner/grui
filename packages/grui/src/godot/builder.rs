use crate::{
    controls::{BuiltinControl, HasChild, IntoControl},
    renderer::{IntoRender, Render},
};
use godot::{builtin::Callable, meta::ToGodot};

pub struct ElementBuilder;

impl ElementBuilder {
    pub fn prop<T>(mut self, _name: &str, _value: T) -> Self
    where
        T: ToGodot,
    {
        // TODO: store properties
        self
    }

    pub fn on(mut self, _event: &str, _handler: Callable) -> Self {
        // TODO: store signals
        self
    }

    pub fn build(&self) -> impl IntoControl {
        BuiltinControl {}
    }
}

impl<NewChild> HasChild<NewChild> for ElementBuilder
where
    NewChild: IntoRender,
    NewChild::Output: Render,
{
    type Output = ElementBuilder;

    fn child(self, _child: NewChild) -> Self::Output {
        // TODO: store child
        self
    }
}
