use crate::controls::{any::IntoAny, IntoControl};
use grui_macros::component;
use reactive_graph::{computed::ArcMemo, traits::Get};

#[component]
pub fn Show<W, F, FC, C>(when: W, fallback: F, children: C) -> impl IntoControl
where
    W: Fn() -> bool + Send + Sync + 'static,
    F: Fn() -> FC,
    FC: IntoControl + 'static,
    C: IntoControl + 'static,
{
    let memoized_when = ArcMemo::new(move |_| when());
    let children = children.into_control();

    move || match memoized_when.get() {
        true => children.into_any(),
        false => fallback().into_any(),
    }
}
