use super::{
    reactive::GodotExecutor,
    render::{Mountable, Render},
};
use crate::controls::IntoControl;
use any_spawner::Executor;
use godot::{classes::Control, meta::AsArg, obj::Gd};
use reactive_graph::owner::Owner;

pub struct Renderer<M: Mountable> {
    #[allow(dead_code)] // FIXME: remove later
    mounted: M,
    #[allow(dead_code)] // FIXME: remove later
    owner: Owner,
}

impl<M> Drop for Renderer<M>
where
    M: Mountable,
{
    fn drop(&mut self) {
        self.mounted.unmount();
    }
}

impl<M> Renderer<M>
where
    M: Mountable,
{
    pub fn mount<N, P, C, T>(parent: N, component: C, props: P) -> Self
    where
        N: AsArg<Gd<Control>>,
        C: FnOnce(P) -> T,
        T: IntoControl,
        T: Render<State = M>,
    {
        let _ = Executor::init_custom_executor(GodotExecutor {});

        let parent = parent.into_arg().to_owned();

        let owner = Owner::new();
        let mounted = owner.with(move || {
            let control = component(props).into_control();
            let mut mountable = control.build();
            mountable.mount(&parent);
            mountable
        });

        Renderer { mounted, owner }
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
