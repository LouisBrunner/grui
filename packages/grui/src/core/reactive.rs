use any_spawner::CustomExecutor;
pub use reactive_graph::prelude::*;
pub use reactive_graph::{
    actions::*,
    computed::*,
    effect::*,
    graph::untrack,
    owner::*,
    signal::*,
    wrappers::{read::*, write::*},
};

pub(crate) struct GodotExecutor {}

impl CustomExecutor for GodotExecutor {
    fn spawn(&self, fut: any_spawner::PinnedFuture<()>) {
        godot::task::spawn(async move {
            fut.await;
        });
    }

    fn spawn_local(&self, fut: any_spawner::PinnedLocalFuture<()>) {
        godot::task::spawn(async move {
            fut.await;
        });
    }

    fn poll_local(&self) {}
}
