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
    cell::{Ref, RefCell, RefMut},
    collections::{HashMap, HashSet},
    rc::Rc,
};

#[derive(Clone)]
pub struct TestHandle {
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
        let node = self.graph.get(self.id);
        format!("{}#{:?}", node.ty, self.id)
    }

    pub(crate) fn get_type(&self) -> String {
        let node = self.graph.get(self.id);
        node.ty.clone()
    }

    pub fn select_by_indices<S: Into<String>>(&self, path: S) -> Option<TestHandle> {
        let path = path.into();
        let parts = path.split('.').collect::<Vec<_>>();
        let mut current = self.clone();
        for part in parts {
            let node = self.graph.get(current.id);
            let index = part.parse::<usize>().ok()?;
            if index >= node.children.len() {
                return None;
            }
            current = self.graph.get(node.children[index]).handle();
        }
        Some(current)
    }

    pub fn call_signal(&mut self, name: &str, args: &[&Variant]) {
        let mut node = self.graph.get_mut(self.id);
        if let Some(signal) = node.signals.get_mut(name) {
            signal.call(args);
        }
    }

    pub fn snapshot(&self) -> serde_json::Result<String> {
        let node = self.graph.get(self.id);
        serde_json::to_string(&node.serialize())
    }

    pub(crate) fn set_prop(&mut self, key: String, value: String) {
        let mut node = self.graph.get_mut(self.id);
        node.props.insert(key, value);
    }

    pub(crate) fn add_signal(&mut self, key: String, func: SignalCallable) {
        let mut node = self.graph.get_mut(self.id);
        node.signals.insert(key, func);
    }

    pub(crate) fn add_child(&mut self, child: &mut TestHandle) {
        {
            let mut node = self.graph.get_mut(self.id);
            node.children.push(child.id);
        }
        {
            let mut child = self.graph.get_mut(child.id);
            child.parent = Some(self.id);
        }
    }

    pub(crate) fn add_sibling(&mut self, sibling: &mut TestHandle) {
        let parent_id = {
            let parent = { self.graph.get_mut(self.id).parent };
            let Some(parent) = parent else {
                return;
            };
            let mut parent = self.graph.get_mut(parent);
            let index = parent
                .children
                .iter()
                .position(|c| *c == self.id)
                .map(|p| p + 1)
                .unwrap_or_else(|| parent.children.len());
            parent.children.insert(index, sibling.id);
            parent.id
        };
        {
            let mut sibling = self.graph.get_mut(sibling.id);
            sibling.parent = Some(parent_id);
        }
    }

    pub(crate) fn unmount(&mut self) {
        {
            let parent = { self.graph.get_mut(self.id).parent };
            if let Some(parent) = parent {
                let mut parent = self.graph.get_mut(parent);
                parent.children.retain(|child| *child != self.id);
            }
        }
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

impl TestNode {
    fn handle(&self) -> TestHandle {
        TestHandle {
            id: self.id,
            graph: self.graph.clone(),
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
                .map(|c| self.graph.get(*c).serialize())
                .collect(),
        }
    }
}

#[derive(Serialize)]
pub struct SerializableTestNode {
    pub ty: String,
    pub props: HashMap<String, String>,
    pub signals: HashSet<String>,
    pub children: Vec<SerializableTestNode>,
}

#[derive(Clone)]
pub(crate) struct TestGraphHandle(Rc<RefCell<TestGraph>>);

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

    fn get(&self, id: uuid::Uuid) -> Ref<'_, TestNode> {
        let graph = self.0.borrow();
        Ref::map(graph, |graph| graph.nodes.get(&id).expect("node not found"))
    }

    fn get_mut(&self, id: uuid::Uuid) -> RefMut<'_, TestNode> {
        let graph = self.0.borrow_mut();
        RefMut::map(graph, |graph| {
            graph.nodes.get_mut(&id).expect("node not found")
        })
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
        let (owner, mounted) = mount(
            Node::Test(root),
            control,
            &BuildOptions {
                graph: Some(graph.clone()),
            },
        );
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
