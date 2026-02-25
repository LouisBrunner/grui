use godot::{classes::Control, obj::Gd};

pub trait IntoRender {
    type Output;

    fn into_render(self) -> Self::Output;
}

impl<T: Render> IntoRender for T {
    type Output = Self;

    fn into_render(self) -> Self::Output {
        self
    }
}

pub trait Render: Sized {
    type State: Mountable;

    fn build(self) -> Self::State;

    fn rebuild(self, state: &mut Self::State);

    fn to_json(self) -> String;
}

pub trait Mountable {
    fn mount(&mut self, parent: &Gd<Control>);

    fn unmount(&mut self);
}
