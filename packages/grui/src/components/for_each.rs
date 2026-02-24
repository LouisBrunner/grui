use crate::{
    core::renderer::Render,
    prelude::{IntoControl, OwnedControl},
};
use godot::{classes::Control, obj::Gd};
use grui_macros::component;
use reactive_graph::{
    owner::Owner,
    signal::{ArcRwSignal, ReadSignal},
    traits::Set,
};
use std::hash::Hash;

#[component]
pub fn For<EF, E, KF, K, CF, C, T>(each: EF, key: KF, children: CF) -> impl IntoControl
where
    EF: Fn() -> E,
    E: IntoIterator<Item = T>,
    KF: Fn(&T) -> K + Clone,
    K: Eq + Hash,
    CF: Fn(T) -> C + Clone,
    C: IntoControl,
    T: Send,
{
    let parent = Owner::current().expect("no reactive owner");
    let children = move |_, child| {
        let owner = parent.with(Owner::new);
        let control = owner.with(|| children(child));
        (|_| {}, OwnedControl::new_with_owner(control, owner))
    };
    move || for_control(each(), key.clone(), children.clone())
}

#[component]
pub fn ForEnumerate<EF, E, KF, K, CF, C, T>(each: EF, key: KF, children: CF) -> impl IntoControl
where
    EF: Fn() -> E,
    E: IntoIterator<Item = T>,
    KF: Fn(&T) -> K + Clone,
    K: Eq + Hash,
    CF: Fn(ReadSignal<usize>, T) -> C + Clone,
    C: IntoControl,
    T: Send,
{
    let parent = Owner::current().expect("no reactive owner");
    let children = move |index, child| {
        let owner = parent.with(Owner::new);
        let (index, set_index) = ArcRwSignal::new(index).split();
        let control = owner.with(|| children(index.into(), child));
        (
            move |index| set_index.set(index),
            OwnedControl::new_with_owner(control, owner),
        )
    };
    move || for_control(each(), key.clone(), children.clone())
}

struct ForControl<E, KF, K, CF, C, CIF, T>
where
    E: IntoIterator<Item = T>,
    KF: Fn(&T) -> K + Clone,
    K: Eq + Hash,
    CF: Fn(usize, T) -> (CIF, C),
    C: IntoControl,
    CIF: Fn(usize),
    T: Send,
{
    each: E,
    #[allow(dead_code)]
    key: KF,
    children: CF,
}

impl<E, KF, K, CF, C, CIF, T> Render for ForControl<E, KF, K, CF, C, CIF, T>
where
    E: IntoIterator<Item = T>,
    KF: Fn(&T) -> K + Clone,
    K: Eq + Hash,
    CF: Fn(usize, T) -> (CIF, C),
    C: IntoControl,
    CIF: Fn(usize),
    T: Send,
{
    fn mount(self, parent: Gd<Control>) {
        self.as_controls().mount(parent)
    }

    fn to_json(self) -> String {
        self.as_controls().to_json()
    }
}

impl<E, KF, K, CF, C, CIF, T> ForControl<E, KF, K, CF, C, CIF, T>
where
    E: IntoIterator<Item = T>,
    KF: Fn(&T) -> K + Clone,
    K: Eq + Hash,
    CF: Fn(usize, T) -> (CIF, C),
    C: IntoControl,
    CIF: Fn(usize),
    T: Send,
{
    fn as_controls(self) -> impl IntoControl {
        self.each
            .into_iter()
            .enumerate()
            .map(|(i, child)| {
                let (set_index, child) = (self.children)(i, child);
                set_index(i);
                child
            })
            .collect::<Vec<_>>()
    }
}

fn for_control<E, KF, K, CF, C, CIF, T>(it: E, key: KF, children: CF) -> impl IntoControl
where
    E: IntoIterator<Item = T>,
    KF: Fn(&T) -> K + Clone,
    K: Eq + Hash,
    CF: Fn(usize, T) -> (CIF, C),
    C: IntoControl,
    CIF: Fn(usize),
    T: Send,
{
    ForControl {
        each: it,
        key,
        children,
    }
}
