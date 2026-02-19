use crate::{controls::IntoControl, prelude::GodotExecutor};
use any_spawner::Executor;
use godot::{classes::Control, meta::AsArg, obj::Gd};
use reactive_graph::owner::Owner;

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
    #[allow(dead_code)] // FIXME: remove later
    root: Vec<Gd<Control>>,
    #[allow(dead_code)] // FIXME: remove later
    owner: Owner,
}

impl Renderer {
    pub fn mount<N, P, C, T>(parent: N, component: C, props: P) -> Self
    where
        N: AsArg<Gd<Control>>,
        C: FnOnce(P) -> T,
        T: IntoControl,
        T: Render,
    {
        let _ = Executor::init_custom_executor(GodotExecutor {});

        let mut parent = parent.into_arg().to_owned();

        let owner = Owner::new();
        let mounted = owner.with(move || {
            let controls = component(props).into_control().to_controls();
            for control in &controls {
                parent.add_child(control);
            }
            controls
        });

        Renderer {
            root: mounted,
            owner,
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
        let result = format!("[{}]", component(props).into_control().to_json());
        TestRenderer { result }
    }

    pub fn snapshot(&self) -> &str {
        &self.result
    }
}
