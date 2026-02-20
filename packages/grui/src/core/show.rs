use crate::prelude::{IntoAny, IntoControl};
use reactive_graph::{computed::ArcMemo, traits::Get};

pub fn show<W, F, C>(when: W, fallback: F, children: C) -> impl IntoControl
where
    W: Fn() -> bool + Send + Sync + 'static,
    F: Fn() -> C,
    C: IntoControl + 'static,
{
    let memoized_when = ArcMemo::new(move |_| when());
    let children = children.into_control();

    move || match memoized_when.get() {
        true => children.into_any(),
        false => fallback().into_any(),
    }
}
