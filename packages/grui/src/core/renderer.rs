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
    pub fn mount<P, FC, C>(parent: P, control: FC) -> Self
    where
        P: AsArg<Gd<Control>>,
        FC: Fn() -> C,
        C: IntoControl + 'static,
        C: Render,
    {
        let _ = Executor::init_custom_executor(GodotExecutor {});

        let parent = parent.into_arg().to_owned();

        let owner = Owner::new();
        let mounted = owner.with(move || {
            let control = control().into_control();
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
    pub fn mount<C>(control: C) -> Self
    where
        C: IntoControl + 'static,
        C: Render,
    {
        let result = format!("[{}]", control.into_control().to_json());
        TestRenderer { result }
    }

    pub fn snapshot(&self) -> &str {
        &self.result
    }
}
