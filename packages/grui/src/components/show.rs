use crate::controls::{any::IntoAny, functions::ControlFn, IntoControl};
use grui_macros::component;
use reactive_graph::{computed::ArcMemo, traits::Get};

#[component]
pub fn Show<W, C, CC>(
    when: W,
    #[prop(optional, into)] fallback: ControlFn,
    children: C,
) -> impl IntoControl
where
    W: Fn() -> bool + Send + Sync + 'static,
    C: Fn() -> CC + 'static,
    CC: IntoControl + 'static,
{
    let memoized_when = ArcMemo::new(move |_| when());

    move || {
        log::info!("Show component rendered: {}", memoized_when.get());
        match memoized_when.get() {
            true => children().into_any(),
            false => match &fallback {
                Some(fallback) => fallback.run(),
                None => ().into_any(),
            },
        }
    }
}
