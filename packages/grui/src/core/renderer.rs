use super::{
    reactive::GodotExecutor,
    render::{BuildOptions, MountPlace, Mountable, Node, Render},
};
use crate::controls::{any::AnyState, IntoControl};
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

pub(crate) fn mount<C, M>(parent: Node, control: C, opts: &BuildOptions) -> (Owner, M)
where
    C: IntoControl + 'static,
    C: Render<State = M>,
    M: Mountable,
{
    let owner = Owner::new();
    let mounted = owner.with(move || {
        let control = control.into_control();
        let mut mountable = control.build(opts);
        mountable.mount(MountPlace::AppendToParent(parent));
        mountable
    });
    (owner, mounted)
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

        let (owner, mounted) = mount(
            Node::Godot(parent),
            control(),
            &BuildOptions {
                ..Default::default()
            },
        );

        Renderer {
            mounted: AnyState::new::<C, C::State>(mounted),
            owner,
        }
    }
}
