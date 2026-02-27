use super::{
    reactive::GodotExecutor,
    render::{Mountable, Render},
};
use crate::{
    controls::{any::AnyState, IntoControl},
    core::render::MountPlace,
};
use any_spawner::Executor;
use godot::{classes::Control, meta::AsArg, obj::Gd};
use reactive_graph::owner::Owner;

pub struct Renderer {
    mounted: AnyState,
    #[allow(dead_code)] // FIXME: remove later
    owner: Owner,
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.mounted.unmount();
    }
}

impl Renderer {
    pub fn mount<P, C>(parent: P, control: C) -> Self
    where
        P: AsArg<Gd<Control>>,
        C: IntoControl + 'static,
        C: Render,
    {
        let _ = Executor::init_custom_executor(GodotExecutor {});

        let parent = parent.into_arg().to_owned();

        let owner = Owner::new();
        let mounted = owner.with(move || {
            let control = control.into_control();
            let mut mountable = control.build();
            mountable.mount(MountPlace::AppendToParent(parent));
            mountable
        });

        Renderer {
            mounted: AnyState::new::<C, C::State>(mounted),
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
