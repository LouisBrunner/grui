use crate::{
    controls::{owned::OwnedControl, IntoControl},
    core::render::{MountPlace, Mountable, Render},
};
use grui_macros::component;
use indexmap::IndexSet;
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
    C: Render,
    T: Send,
{
    let parent = Owner::current().expect("no reactive owner");
    let children = move |_, child| {
        let owner = parent.with(Owner::new);
        let control = owner.with(|| children(child));
        (|_| {}, OwnedControl::new_with_owner(control, owner))
    };
    move || ForControl::new(each(), key.clone(), children.clone())
}

#[component]
pub fn ForEnumerate<EF, E, KF, K, CF, C, T>(each: EF, key: KF, children: CF) -> impl IntoControl
where
    EF: Fn() -> E,
    E: IntoIterator<Item = T>,
    KF: Fn(&T) -> K + Clone,
    K: Eq + Hash,
    CF: Fn(ReadSignal<usize>, T) -> C + Clone,
    C: Render,
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
    move || ForControl::new(each(), key.clone(), children.clone())
}

struct ForControl<E, KF, K, CF, C, CIF, T>
where
    E: IntoIterator<Item = T>,
    KF: Fn(&T) -> K + Clone,
    K: Eq + Hash,
    CF: Fn(usize, T) -> (CIF, C),
    C: Render,
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
    C: Render,
    CIF: Fn(usize),
    T: Send,
{
    type State = KeysState<K, CIF, C::State>;

    fn build(self) -> Self::State {
        let items = self.each.into_iter();
        let (capacity, _) = items.size_hint();
        let mut hashed_items = IndexSet::with_capacity(capacity);
        let mut rendered_items = Vec::with_capacity(capacity);
        for (index, item) in items.enumerate() {
            hashed_items.insert((self.key)(&item));
            let (set_index, view) = (self.children)(index, item);
            rendered_items.push((set_index, view.build()));
        }
        KeysState {
            hashed_items,
            rendered_items,
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        let KeysState {
            hashed_items,
            ref mut rendered_items,
        } = state;
        let new_items = self.each.into_iter();
        let (capacity, _) = new_items.size_hint();
        let mut new_hashed_items = IndexSet::with_capacity(capacity);

        let mut items = Vec::new();
        for item in new_items {
            new_hashed_items.insert((self.key)(&item));
            items.push(item);
        }

        // TODO: apply diff of hashed_items -> new_hashed_items to new_items
        for (_, item) in rendered_items.iter_mut() {
            item.unmount();
        }
        rendered_items.clear();
        for (index, item) in items.into_iter().enumerate() {
            let (set_index, view) = (self.children)(index, item);
            rendered_items.push((set_index, view.build()));
        }

        *hashed_items = new_hashed_items;
    }

    fn to_json(self) -> String {
        self.each
            .into_iter()
            .enumerate()
            .map(|(i, child)| {
                let (_, child) = (self.children)(i, child);
                child
            })
            .collect::<Vec<_>>()
            .to_json()
    }
}

impl<E, KF, K, CF, C, CIF, T> ForControl<E, KF, K, CF, C, CIF, T>
where
    E: IntoIterator<Item = T>,
    KF: Fn(&T) -> K + Clone,
    K: Eq + Hash,
    CF: Fn(usize, T) -> (CIF, C),
    C: Render,
    CIF: Fn(usize),
    T: Send,
{
    fn new(it: E, key: KF, children: CF) -> Self {
        ForControl {
            each: it,
            key,
            children,
        }
    }
}

struct KeysState<K, CIF, M>
where
    K: Eq + Hash,
    CIF: Fn(usize),
    M: Mountable,
{
    hashed_items: IndexSet<K>,
    rendered_items: Vec<(CIF, M)>,
}

impl<K, CIF, M> Mountable for KeysState<K, CIF, M>
where
    K: Eq + Hash,
    CIF: Fn(usize),
    M: Mountable,
{
    fn mount(&mut self, place: MountPlace) {
        match &place {
            MountPlace::AppendToParent(_) => {
                for (_, item) in self.rendered_items.iter_mut() {
                    item.mount(place.clone());
                }
            }
            MountPlace::AfterSibling(_) => {
                for (_, item) in self.rendered_items.iter_mut().rev() {
                    item.mount(place.clone());
                }
            }
        }
    }

    fn mount_after(&mut self, sibling: &mut dyn Mountable) {
        for (_, item) in self.rendered_items.iter_mut().rev() {
            item.mount_after(sibling);
        }
    }

    fn unmount(&mut self) {
        for (_, child) in self.rendered_items.iter_mut() {
            child.unmount();
        }
    }
}
