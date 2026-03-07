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
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

#[derive(Clone)]
pub(super) struct TestHandle {
    id: uuid::Uuid,
    graph: TestGraphHandle,
}

impl TestHandle {
    pub(crate) fn new<S>(ty: S, graph: &TestGraphHandle) -> TestHandle
    where
        S: Into<String>,
    {
        graph.add(uuid::Uuid::new_v4(), ty.into())
    }

    pub(crate) fn get_id(&self) -> String {
        format!("{}#{:?}", self.ty, self.id)
    }

    pub(crate) fn get_type(&self) -> String {
        self.ty.clone()
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
            current = self.graph.get(current.children[index])?;
        }
        Some(current)
    }

    pub fn call_signal(&mut self, name: &str, args: &[&Variant]) {
        if let Some(signal) = self.signals.get_mut(name) {
            signal.call(args);
        }
    }

    fn serialize(&self) -> SerializableTestNode {
        SerializableTestNode {
            ty: self.ty.clone(),
            props: self.props.clone(),
            signals: self.signals.keys().cloned().collect(),
            children: self
                .children
                .iter()
                .filter_map(|c| Some(self.graph.get(*c)?.serialize()))
                .collect(),
        }
    }

    pub fn snapshot(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self.serialize())
    }

    pub(crate) fn set_prop(&mut self, key: String, value: String) {
        self.props.insert(key, value);
    }

    pub(crate) fn add_signal(&mut self, key: String, func: SignalCallable) {
        self.signals.insert(key, func);
    }

    pub(crate) fn add_child(&mut self, child: &mut TestHandle) {
        self.children.push(child.id);
        child.parent = Some(self.id);
    }

    pub(crate) fn add_sibling(&mut self, sibling: &mut TestHandle) {
        let Some(parent) = self.parent else {
            return;
        };
        let Some(parent) = self.graph.get_mut(parent) else {
            return;
        };
        let index = parent
            .children
            .iter()
            .position(|c| *c == self.id)
            .map(|p| p + 1)
            .unwrap_or_else(|| parent.children.len());
        parent.children.insert(index, sibling.id);
        sibling.parent = Some(parent.id);
    }

    pub(crate) fn unmount(&mut self) {
        let Some(parent) = self.parent else {
            return;
        };
        let Some(parent) = self.graph.get_mut(parent) else {
            return;
        };
        parent.children.retain(|child| *child != self.id);
        self.graph.remove(self.id);
    }
}

struct TestNode {
    id: uuid::Uuid,
    ty: String,
    parent: Option<uuid::Uuid>,
    props: HashMap<String, String>,
    signals: HashMap<String, SignalCallable>,
    children: Vec<uuid::Uuid>,
    graph: TestGraphHandle,
}

#[derive(Serialize)]
pub struct SerializableTestNode {
    pub ty: String,
    pub props: HashMap<String, String>,
    pub signals: HashSet<String>,
    pub children: Vec<SerializableTestNode>,
}

#[derive(Clone)]
pub(super) struct TestGraphHandle(Rc<RefCell<TestGraph>>);

struct TestGraph {
    root: uuid::Uuid,
    nodes: HashMap<uuid::Uuid, TestNode>,
}

impl TestGraphHandle {
    fn new() -> (Self, TestHandle) {
        let root_id = uuid::Uuid::new_v4();
        let graph = Self(Rc::new(RefCell::new(TestGraph {
            root: root_id,
            nodes: HashMap::new(),
        })));
        let root = graph.add(root_id, "Root".to_string());
        (graph, root)
    }

    fn add(&self, id: uuid::Uuid, ty: String) -> TestHandle {
        let mut graph = self.0.borrow_mut();
        graph.nodes.insert(
            id,
            TestNode {
                id,
                ty,
                parent: None,
                props: HashMap::new(),
                signals: HashMap::new(),
                children: Vec::new(),
                graph: self.clone(),
            },
        );
        TestHandle {
            id,
            graph: self.clone(),
        }
    }

    fn get(&self, id: uuid::Uuid) -> Option<&TestNode> {
        let graph = self.0.borrow();
        graph.nodes.get(&id)
    }

    fn get_mut(&self, id: uuid::Uuid) -> Option<&mut TestNode> {
        let mut graph = self.0.borrow_mut();
        graph.nodes.get_mut(&id)
    }

    fn remove(&self, id: uuid::Uuid) {
        let mut graph = self.0.borrow_mut();
        if graph.root == id {
            return;
        }
        graph.nodes.remove(&id);
    }

    fn get_root(&self) -> TestHandle {
        let graph = self.0.borrow();
        TestHandle {
            id: graph.root.clone(),
            graph: self.clone(),
        }
    }
}

pub struct TestRenderer {
    mounted: AnyState,
    #[allow(dead_code)] // FIXME: remove later
    owner: Owner,
    graph: TestGraphHandle,
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

        let (graph, root) = TestGraphHandle::new();
        let (owner, mounted) = mount(Node::Test(root), control, &BuildOptions { test: true });
        let renderer = Self {
            mounted: AnyState::new::<C, C::State>(mounted),
            owner,
            graph,
        };
        actions(&renderer);
    }

    pub fn get_root(&self) -> TestHandle {
        self.graph.get_root()
    }
}
