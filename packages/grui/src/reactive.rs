use std::cell::{Cell, RefCell};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::node::Node;

thread_local! {
    // Global dirty flag set whenever any signal is written.
    static DIRTY: Cell<bool> = const { Cell::new(true) }; // start dirty so first process renders
}

/// Marks the UI as needing re-render.
pub fn mark_dirty() {
    DIRTY.with(|d| d.set(true));
}

/// Returns whether a render is required. Resets flag to false if it was true.
pub fn take_dirty() -> bool {
    DIRTY.with(|d| {
        if d.get() {
            d.set(false);
            true
        } else {
            false
        }
    })
}

/// A readable handle to a signal.
pub struct ReadSignal<T: Clone> {
    inner: Rc<RefCell<T>>,
}

impl<T: Clone> ReadSignal<T> {
    pub fn get(&self) -> T { self.inner.borrow().clone() }
}

/// A writable handle to a signal.
pub struct WriteSignal<T: Clone> {
    inner: Rc<RefCell<T>>,
}

impl<T: Clone> WriteSignal<T> {
    pub fn set(&self, value: T) {
        *self.inner.borrow_mut() = value;
        mark_dirty();
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let mut borrow = self.inner.borrow_mut();
        f(&mut borrow);
        mark_dirty();
    }
}

/// Creates a reactive signal (Leptos-like). Returns (read, write) handles.
pub fn signal<T: Clone + 'static>(value: T) -> (ReadSignal<T>, WriteSignal<T>) {
    let inner = Rc::new(RefCell::new(value));
    (
        ReadSignal { inner: inner.clone() },
        WriteSignal { inner },
    )
}

/// Simple reactive effect: runs immediately and again after any signal write.
/// For now it re-runs each time the global dirty flag triggers a renderer pass;
/// the renderer calls all registered effects after its render function runs.
pub struct Effect(usize);

thread_local! {
    static EFFECTS: RefCell<Vec<Box<dyn FnMut()>>> = RefCell::new(Vec::new());
}

impl Effect {
    pub fn new(f: impl FnMut() + 'static) -> Self {
        let mut boxed: Box<dyn FnMut()> = Box::new(f);
        // run once immediately
        boxed.as_mut()();
        let id = EFFECTS.with(|effects| {
            let mut effects = effects.borrow_mut();
            effects.push(boxed);
            effects.len() - 1
        });
        Self(id)
    }
}

/// Runs all registered effects. Called by the renderer after a successful render.
pub fn run_effects() {
    EFFECTS.with(|effects| {
        for effect in effects.borrow_mut().iter_mut() {
            effect();
        }
    });
}

/// for_each builds a fragment by mapping an iterator (provided by a closure) to nodes.
/// Rough Leptos <For /> equivalent without diffing.
pub fn for_each<I, F, K, T, KH>(each: I, key: K, f: F) -> Node
where
    I: IntoIterator<Item = T>,
    F: Fn(&T) -> Node,
    K: Fn(&T) -> KH,
    KH: Hash,{
    let mut idx = 0usize;
    let children: Vec<Node> = each
        .into_iter()
        .map(|item| {
            let node = f(&item);
            // Try to inject a stable key into element nodes if present.
            let hashed_key = {
                let mut hasher = DefaultHasher::new();
                key(&item).hash(&mut hasher);
                hasher.finish().to_string()
            };
            let maybe_keyed = match node.clone() {
                Node::Element(mut el) => {
                    if el.key.is_none() {
                        el.key = Some(hashed_key);
                    }
                    Node::Element(el)
                }
                _ => node,
            };
            idx += 1;
            maybe_keyed
        })
        .collect();
    Node::fragment(children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Node as VNode;

    #[test]
    fn signal_set_and_get() {
        let (count, set_count) = signal(0);
        assert_eq!(count.get(), 0);
        set_count.set(1);
        assert_eq!(count.get(), 1);
    }

    #[test]
    fn signal_update() {
        let (text, set_text) = signal(String::from("A"));
        set_text.update(|s| s.push('B'));
        assert_eq!(text.get(), "AB");
    }

    #[test]
    fn for_each_builds_fragment() {
        let list = for_each(0..3, |i| *i, |i| VNode::text(format!("Item {i}")));
        match list {
            VNode::Fragment(children) => {
                assert_eq!(children.len(), 3);
            }
            _ => panic!("expected fragment"),
        }
    }
}
