use crate::controls::{any::IntoAny, IntoControl};
use grui_macros::component;
use reactive_graph::{computed::ArcMemo, traits::Get};

#[component]
pub fn Show<W, F, FC, C, CC>(
    when: W,
    #[prop(optional)] fallback: F,
    children: C,
) -> impl IntoControl
where
    W: Fn() -> bool + Send + Sync + 'static,
    F: Fn() -> FC + 'static,
    FC: IntoControl + 'static,
    C: Fn() -> CC + 'static,
    CC: IntoControl + 'static,
{
    let memoized_when = ArcMemo::new(move |_| when());

    move || match memoized_when.get() {
        true => children().into_any(),
        false => match &fallback {
            Some(fallback) => fallback().into_any(),
            None => ().into_any(),
        },
    }
}
