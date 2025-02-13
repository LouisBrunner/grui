use std::collections::HashMap;

use godot::{classes::Control, prelude::*};

use crate::godot::VNodeKind;

#[derive(Clone, Debug)]
struct VNodeMeta {
    key: u64,      // unique identifier
    hash: u64,     // has it changed
    own_hash: u64, // has its properties changed
    class: VNodeKind,
    // TODO: class properties
    children_hash: u64, // have its children changed
}

#[derive(Debug)]
struct VNode {
    meta: VNodeMeta,
    children: Vec<VNode>,
}

#[derive(Debug, Clone)]
struct RNode {
    meta: VNodeMeta,
    instance: Gd<Control>,
    children: Vec<RNode>,
}

pub struct Renderer {
    root: Gd<Control>,
    app: Node,
    rtree: RNode,
}

impl Renderer {
    pub fn new(root: Gd<Control>, app: Node) -> Self {
        let vtree = Self::render(&app);
        let rtree = Self::create(&vtree);

        let mut root = root;
        root.add_child(&rtree.instance);

        Self { root, rtree, app }
    }

    pub fn process(&mut self, _delta: f64) {
        // TODO: where is state stored?
        let new_tree = Self::render(&self.app);
        self.apply(new_tree);
    }

    fn apply(&mut self, vtree: VNode) {
        Self::patch(&mut self.root, &mut self.rtree, &vtree);
    }

    fn patch(parent: &mut Gd<Control>, old: &mut RNode, new: &VNode) {
        if old.meta.hash == new.meta.hash {
            return;
        }

        if old.meta.own_hash != new.meta.own_hash {
            if old.meta.class != new.meta.class {
                let new_instance = new.meta.class.create_instance();
                parent.add_child(&new_instance);
                parent.remove_child(&old.instance);
                let old_instance = std::mem::replace(&mut old.instance, new_instance);
                old_instance.free();
            }
            // TODO: patch properties
        }

        if old.meta.children_hash != new.meta.children_hash {
            let mut old_children = std::mem::replace(&mut old.children, vec![]);
            let mut keys = old_children
                .iter_mut()
                .map(|old_child| (old_child.meta.key, old_child))
                .collect::<HashMap<_, _>>();
            old.children = new
                .children
                .iter()
                .map(|new_child| {
                    if let Some(old_child) = keys.remove(&new_child.meta.key) {
                        Self::patch(&mut old.instance, old_child, new_child);
                        // FIXME: better way to reorder?
                        old.instance.remove_child(&old_child.instance);
                        old.instance.add_child(&old_child.instance);
                        old_child.clone()
                    } else {
                        let new_instance = new_child.meta.class.create_instance();
                        old.instance.add_child(&new_instance);
                        RNode {
                            meta: new_child.meta.clone(),
                            instance: new_instance,
                            children: vec![],
                        }
                    }
                })
                .collect();
            for (_, old_child) in keys {
                old.instance.remove_child(&old_child.instance);
                old_child.instance.clone().free();
            }
        }
    }
    fn create(vnode: &VNode) -> RNode {
        let instance = vnode.meta.class.create_instance();
        let children = vnode.children.iter().map(Self::create).collect();
        RNode {
            meta: vnode.meta.clone(),
            instance,
            children,
        }
    }

    fn render(app: &Node) -> VNode {
        todo!()
    }
}
