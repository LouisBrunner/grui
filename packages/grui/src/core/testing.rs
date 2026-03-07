use super::{
    render::{Mountable, Node, Render},
    renderer::mount,
};
use crate::{
    controls::signals::SignalCallable,
    core::render::BuildOptions,
    prelude::{any::AnyState, IntoControl},
};
use godot::builtin::Variant;
use reactive_graph::owner::Owner;
use serde::Serialize;
use std::{
    collections::HashMap,
    rc::{Rc, Weak},
};

#[derive(Serialize)]
pub struct TestNode {
    pub ty: String,
    #[serde(skip_serializing)]
    parent: Option<Weak<TestNode>>,
    pub props: HashMap<String, String>,
    pub signals: TestSignals,
    pub children: Vec<Rc<TestNode>>,
}

impl TestNode {
    pub(crate) fn new<S>(ty: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            ty: ty.into(),
            parent: None,
            props: HashMap::new(),
            signals: TestSignals::new(),
            children: Vec::new(),
        }
    }

    pub fn select_by_indices<S: Into<String>>(&self, path: S) -> Option<&TestNode> {
        let path = path.into();
        let parts = path.split('.').collect::<Vec<_>>();
        let mut current = self;
        for part in parts {
            let index = part.parse::<usize>().ok()?;
            if index >= current.children.len() {
                return None;
            }
            current = &current.children[index];
        }
        Some(current)
    }

    pub fn call_signal(&mut self, name: &str, args: &[&Variant]) {
        if let Some(signal) = self.signals.0.get_mut(name) {
            signal.call(args);
        }
    }

    pub fn snapshot(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    pub(crate) fn set_prop(node: &mut Rc<TestNode>, key: String, value: String) {
        if let Some(node) = TestNode::get_mut(node) {
            node.props.insert(key, value);
        }
    }

    pub(crate) fn add_signal(node: &mut Rc<TestNode>, key: String, func: SignalCallable) {
        if let Some(node) = TestNode::get_mut(node) {
            node.signals.0.insert(key, func);
        }
    }

    pub(crate) fn add_child(parent: &mut Rc<TestNode>, child: &mut Rc<TestNode>) {
        if let Some(parent) = TestNode::get_mut(parent) {
            parent.children.push(child.clone());
        }
        if let Some(child) = TestNode::get_mut(child) {
            child.parent = Some(Rc::downgrade(parent));
        }
    }

    pub(crate) fn add_sibling(sibling: &mut Rc<TestNode>, child: &mut Rc<TestNode>) {
        {
            let Some(parent) = sibling.parent.as_ref() else {
                return;
            };
            let Some(mut parent) = parent.upgrade() else {
                return;
            };
            let Some(parent) = TestNode::get_mut(&mut parent) else {
                return;
            };
            let index = sibling
                .children
                .iter()
                .position(|c| Rc::as_ptr(c) == Rc::as_ptr(&child))
                .map(|p| p + 1)
                .unwrap_or_else(|| parent.children.len());
            parent.children.insert(index, child.clone());
        }

        if let Some(child) = TestNode::get_mut(child) {
            child.parent = sibling.parent.clone();
        }
    }

    pub(crate) fn unmount(node: &mut Rc<TestNode>) {
        let Some(parent) = node.parent.as_ref() else {
            return;
        };
        let Some(mut parent) = parent.upgrade() else {
            return;
        };
        let Some(parent) = TestNode::get_mut(&mut parent) else {
            return;
        };
        parent
            .children
            .retain(|child| Rc::as_ptr(child) != Rc::as_ptr(node));
    }

    pub(crate) fn get_mut(node: &mut Rc<TestNode>) -> Option<&mut TestNode> {
        let id = TestNode::get_id(node);
        let res = Rc::get_mut(node);
        debug_assert!(
            res.is_some(),
            "Failed to get mutable reference to node {:?}",
            id
        );
        res
    }

    pub(crate) fn get_id(node: &Rc<TestNode>) -> String {
        format!("{}#{:?}", node.ty, Rc::as_ptr(node))
    }
}

pub struct TestSignals(HashMap<String, SignalCallable>);

impl TestSignals {
    fn new() -> Self {
        Self(HashMap::new())
    }
}

impl Serialize for TestSignals {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

pub struct TestRenderer {
    mounted: AnyState,
    #[allow(dead_code)] // FIXME: remove later
    owner: Owner,
    root: Rc<TestNode>,
}

impl Drop for TestRenderer {
    fn drop(&mut self) {
        self.mounted.unmount();
    }
}

impl TestRenderer {
    pub fn mount<C, F>(control: C, actions: F)
    where
        C: IntoControl + 'static,
        C: Render,
        F: Fn(&Self),
    {
        // let _ = Executor::init_local_custom_executor(TestExecutor {});
        let root = Rc::new(TestNode::new("Root"));
        let (owner, mounted) = mount(
            Node::Test(root.clone()),
            control,
            &BuildOptions { test: true },
        );
        let renderer = Self {
            mounted: AnyState::new::<C, C::State>(mounted),
            owner,
            root,
        };
        actions(&renderer);
    }

    pub fn get_root(&self) -> &TestNode {
        &self.root
    }
}
