use crate::{
    controls::owned::OwnedControl,
    core::render::{BuildOptions, MountPlace, Mountable, Node, Render},
    godot::ty::GDType,
};
use grui::prelude::*;
use indexmap::IndexSet;
use std::hash::Hash;

// FIXME: I don't think we are using set_index correctly

#[component]
pub fn For<EF, E, KF, K, CF, C, T>(each: EF, key: KF, children: CF) -> impl IntoControl
where
    EF: Fn() -> E + 'static,
    E: IntoIterator<Item = T>,
    KF: Fn(&T) -> K + Clone + 'static,
    K: Eq + Hash + Ord + 'static,
    CF: Fn(T) -> C + Clone + 'static,
    C: Render + 'static,
    T: Send + 'static,
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
    EF: Fn() -> E + 'static,
    E: IntoIterator<Item = T>,
    KF: Fn(&T) -> K + Clone + 'static,
    K: Eq + Hash + Ord + 'static,
    CF: Fn(ReadSignal<usize>, T) -> C + Clone + 'static,
    C: Render + 'static,
    T: Send + 'static,
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
    K: Hash + Ord,
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
    K: Hash + Ord,
    CF: Fn(usize, T) -> (CIF, C),
    C: Render,
    CIF: Fn(usize),
    T: Send,
{
    type State = ForState<K, CIF, C::State>;

    fn build(self, opts: &BuildOptions) -> Self::State {
        let items = self.each.into_iter();
        let (capacity, _) = items.size_hint();
        let mut hashed_items = IndexSet::with_capacity(capacity);
        let mut rendered_items = Vec::with_capacity(capacity);
        for (index, item) in items.enumerate() {
            hashed_items.insert((self.key)(&item));
            let (set_index, view) = (self.children)(index, item);
            rendered_items.push(Some((set_index, view.build(opts))));
        }
        ForState {
            placeholder: Node::new(GDType::Control, opts),
            keys: hashed_items,
            items: rendered_items,
        }
    }

    fn rebuild(self, state: &mut Self::State, opts: &BuildOptions) {
        let new_items = self.each.into_iter();
        let (capacity, _) = new_items.size_hint();
        let mut new_keys = IndexSet::with_capacity(capacity);
        let mut items = Vec::new();
        for item in new_items {
            new_keys.insert((self.key)(&item));
            items.push(Some(item));
        }

        state.do_diff(items, new_keys, self.children, opts);
    }
}

fn take_from_vec<T>(v: &mut Vec<Option<T>>, i: usize) -> Option<T> {
    let mut item = None;
    std::mem::swap(&mut v[i], &mut item);
    item
}

impl<E, KF, K, CF, C, CIF, T> ForControl<E, KF, K, CF, C, CIF, T>
where
    E: IntoIterator<Item = T>,
    KF: Fn(&T) -> K + Clone,
    K: Hash + Ord,
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

struct ForState<K, CIF, M>
where
    K: Hash + Ord,
    CIF: Fn(usize),
    M: Mountable,
{
    placeholder: Node,
    keys: IndexSet<K>,
    items: Vec<Option<(CIF, M)>>,
}

impl<K, CIF, M> Mountable for ForState<K, CIF, M>
where
    K: Hash + Ord,
    CIF: Fn(usize),
    M: Mountable,
{
    fn mount(&mut self, place: MountPlace) {
        self.placeholder.mount(place);
        for (_, item) in self.items.iter_mut().flatten() {
            item.mount(MountPlace::AfterSibling(self.placeholder.clone()));
        }
    }

    fn mount_after(&mut self, sibling: &mut dyn Mountable) {
        self.placeholder.mount_after(sibling);
        for (_, item) in self.items.iter_mut().flatten() {
            item.mount(MountPlace::AfterSibling(self.placeholder.clone()));
        }
    }

    fn unmount(&mut self) {
        self.placeholder.unmount();
        for (_, child) in self.items.iter_mut().flatten() {
            child.unmount();
        }
    }
}

impl<K, CIF, M> ForState<K, CIF, M>
where
    K: Hash + Ord,
    CIF: Fn(usize),
    M: Mountable,
{
    fn do_diff<CF, C, T>(
        &mut self,
        mut new_children: Vec<Option<T>>,
        new_keys: IndexSet<K>,
        make_child: CF,
        opts: &BuildOptions,
    ) where
        CF: Fn(usize, T) -> (CIF, C),
        C: Render<State = M>,
        T: Send,
    {
        let old = self.keys.iter().collect::<Vec<_>>();
        let new = new_keys.iter().collect::<Vec<_>>();
        let diff = similar::capture_diff_slices(similar::Algorithm::Myers, &old, &new);

        let mut prev = None;
        let mut new_items = Vec::with_capacity(new_children.len());
        for op in diff {
            match op {
                similar::DiffOp::Equal {
                    old_index,
                    new_index,
                    len,
                } => {
                    for i in 0..len {
                        let old = old_index + i;
                        let new = new_index + i;
                        let Some(item) = take_from_vec(&mut new_children, new) else {
                            log::warn!("Item {} was not found in the iterator items", new);
                            continue;
                        };
                        let Some((_, mut state)) = take_from_vec(&mut self.items, old) else {
                            log::warn!("Item {} was not found in the rendered items", old);
                            continue;
                        };
                        let (set_index, child) = (make_child)(new, item);
                        child.rebuild(&mut state, opts);
                        prev = Some(new);
                        new_items.push(Some((set_index, state)));
                    }
                }
                similar::DiffOp::Delete {
                    old_index,
                    old_len,
                    new_index: _,
                } => {
                    for i in old_index..old_index + old_len {
                        let Some((_, mut state)) = take_from_vec(&mut self.items, i) else {
                            log::warn!("Item {} was not found in the rendered items", i);
                            continue;
                        };
                        state.unmount();
                    }
                }
                similar::DiffOp::Insert {
                    old_index: _,
                    new_index,
                    new_len,
                } => {
                    for i in new_index..new_index + new_len {
                        let Some(item) = take_from_vec(&mut new_children, i) else {
                            log::warn!("Item {} was not found in the iterator items", i);
                            continue;
                        };
                        let (set_index, child) = (make_child)(i, item);
                        let mut state = child.build(opts);
                        match prev {
                            Some(prev) => match new_items[prev].as_mut() {
                                Some(prev) => prev.1.mount_after(&mut state),
                                None => self.placeholder.mount_after(&mut state),
                            },
                            None => self.placeholder.mount_after(&mut state),
                        }
                        new_items.push(Some((set_index, state)));
                    }
                }
                similar::DiffOp::Replace {
                    old_index,
                    old_len: _,
                    new_index,
                    new_len: _,
                } => {
                    panic!("Replacing {} with {}", old_index, new_index)
                }
            }
        }

        self.items = new_items;
        self.keys = new_keys;
    }
}
