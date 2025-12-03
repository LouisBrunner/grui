use crate::control::IntoControl;
use godot::{classes::Control, obj::Gd};

pub struct Renderer {
    parent: Gd<Control>,
    root: Vec<Gd<Control>>,
}

impl Renderer {
    pub fn mount<P, C, T>(mut parent: Gd<Control>, component: C, props: P) -> Self
    where
        C: FnOnce(P) -> T,
        T: IntoControl,
    {
        let controls = component(props).to_controls();
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
    {
        let result = component(props).to_json();
        TestRenderer { result }
    }

    pub fn snapshot(&self) -> &str {
        &self.result
    }
}
