use crate::controls::IntoControl;
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
    fn to_controls(self) -> Vec<Gd<Control>>;
    fn to_json(self) -> String;
}

pub struct Renderer {
    parent: Gd<Control>,
    root: Vec<Gd<Control>>,
}

impl Renderer {
    pub fn mount<P, C, T>(mut parent: Gd<Control>, component: C, props: P) -> Self
    where
        C: FnOnce(P) -> T,
        T: IntoControl,
        T: Render,
    {
        let controls = component(props).into_control().to_controls();
        for control in &controls {
            parent.add_child(control);
        }
        Renderer {
            parent,
            root: controls,
        }
    }
}

pub struct TestRenderer {
    result: String,
}

impl TestRenderer {
    pub fn mount<P, C, T>(component: C, props: P) -> Self
    where
        C: FnOnce(P) -> T,
        T: IntoControl,
        T: Render,
    {
        let result = component(props).into_control().to_json();
        TestRenderer { result }
    }

    pub fn snapshot(&self) -> &str {
        &self.result
    }
}
