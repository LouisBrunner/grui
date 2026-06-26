use godot_grui::prelude::*;

#[component]
pub fn Show<W>(
    when: W,
    #[prop(optional, into)] fallback: ControlFn,
    children: ChildrenFn,
) -> impl IntoControl
where
    W: Fn() -> bool + Send + Sync + 'static,
{
    let memoized_when = ArcMemo::new(move |_| when());

    move || match memoized_when.get() {
        true => children().into_any(),
        false => match &fallback {
            Some(fallback) => fallback.run(),
            None => ().into_any(),
        },
    }
}
