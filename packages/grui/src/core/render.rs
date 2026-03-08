#[cfg(feature = "testing")]
use super::testing::{TestGraphHandle, TestHandle};
use crate::{controls::signals::SignalCallable, godot::ty::GDType};
use godot::{
    classes::Control,
    global::PropertyUsageFlags,
    meta::ToGodot,
    obj::{EngineBitfield, Gd},
};
use std::{collections::HashSet, fmt::Debug};

pub trait IntoRender {
    type Output;

    fn into_render(self) -> Self::Output;
}

impl<T: Render> IntoRender for T {
    type Output = Self;

    fn into_render(self) -> Self::Output {
        self
    }
}

#[derive(Clone, Default)]
pub struct BuildOptions {
    #[cfg(feature = "testing")]
    pub(crate) graph: Option<TestGraphHandle>,
}

pub trait Render: Sized {
    type State: Mountable;

    fn build(self, opts: &BuildOptions) -> Self::State;

    fn rebuild(self, state: &mut Self::State, opts: &BuildOptions);
}

#[derive(Clone)]
pub(crate) enum MountPlace {
    AppendToParent(Node),
    AfterSibling(Node),
}

pub trait Mountable {
    fn mount(&mut self, place: MountPlace);

    fn mount_after(&mut self, sibling: &mut dyn Mountable);

    fn unmount(&mut self);
}

#[derive(Clone)]
pub enum Node {
    Godot(Gd<Control>),
    #[cfg(feature = "testing")]
    Test(TestHandle),
}

impl Node {
    pub(crate) fn new(ty: GDType, _opts: &BuildOptions) -> Self {
        #[cfg(feature = "testing")]
        match &_opts.graph {
            Some(graph) => Self::Test(TestHandle::new(ty.to_string(), graph)),
            None => Self::Godot(ty.create_instance()),
        }
        #[cfg(not(feature = "testing"))]
        {
            Self::Godot(ty.create_instance())
        }
    }

    pub(crate) fn get_id(&self) -> String {
        match &self {
            Node::Godot(node) => {
                let mut instance_id = "unknown".to_string();
                if node.is_instance_valid() {
                    instance_id = node.instance_id().to_string();
                }
                let prefix = format!("{}#{}", node.get_class(), instance_id);
                if !node.get_name().is_empty() {
                    format!("{}+{}", prefix, node.get_name())
                } else {
                    prefix
                }
            }
            #[cfg(feature = "testing")]
            Node::Test(node) => node.get_id(),
        }
    }

    pub(crate) fn get_class(&self) -> String {
        match &self {
            Node::Godot(node) => node.get_class().to_string(),
            #[cfg(feature = "testing")]
            Node::Test(node) => node.get_type(),
        }
    }

    pub(crate) fn get_properties(&self) -> Option<HashSet<String>> {
        #[allow(irrefutable_let_patterns)] // due to testing feature
        let Node::Godot(node) = self
        else {
            return None;
        };
        let properties = node
            .get_property_list()
            .iter_shared()
            .filter_map(|property| {
                let name = property.get("name")?;
                let usage = property.get("usage")?.try_to::<PropertyUsageFlags>().ok()?;
                if !usage.is_set(PropertyUsageFlags::STORAGE) {
                    return None;
                }
                Some(name.to_string())
            })
            .collect::<HashSet<_>>();
        Some(properties)
    }

    pub(crate) fn set<VF, V>(&self, key: &str, value: &VF) -> String
    where
        VF: Fn() -> V + 'static,
        V: Debug + ToGodot,
    {
        match self.clone() {
            Node::Godot(mut node) => {
                let variant = value().to_variant();
                node.set(key, &variant);
                variant.to_string()
            }
            #[cfg(feature = "testing")]
            Node::Test(mut node) => {
                use crate::controls::props::serialize_prop;

                let text = serialize_prop(value);
                node.set_prop(key.to_string(), text.clone());
                text
            }
        }
    }

    pub(crate) fn connect(&self, key: String, func: SignalCallable) {
        match self.clone() {
            Node::Godot(mut node) => {
                node.connect(&key, &func.to_godot(&key));
            }
            #[cfg(feature = "testing")]
            Node::Test(mut node) => {
                node.add_signal(key, func);
            }
        }
    }
}

impl Mountable for Node {
    fn mount(&mut self, place: MountPlace) {
        match place {
            MountPlace::AppendToParent(mut parent) => {
                log::trace!("mounting {} to parent {}", self.get_id(), parent.get_id());
                match self {
                    Node::Godot(node) => {
                        #[allow(irrefutable_let_patterns)] // due to testing feature
                        let Node::Godot(parent) = &mut parent
                        else {
                            debug_assert!(
                              false,
                              "Node and parent need to be the same type: Godot({:?}) vs Other({:?})",
                              self.get_id(),
                              parent.get_id()
                            );
                            return;
                        };
                        parent.add_child(&node.clone())
                    }
                    #[cfg(feature = "testing")]
                    Node::Test(node) => {
                        let Node::Test(parent) = &mut parent else {
                            debug_assert!(
                            false,
                            "Node and parent need to be the same type: Test({:?}) vs Other({:?})",
                            self.get_id(),
                            parent.get_id()
                          );
                            return;
                        };
                        parent.add_child(node)
                    }
                }
            }
            MountPlace::AfterSibling(mut sibling) => {
                log::trace!(
                    "mounting {} after sibling {}",
                    self.get_id(),
                    sibling.get_id()
                );
                match self {
                    Node::Godot(node) => {
                        let sibling_id = sibling.get_id();
                        #[allow(irrefutable_let_patterns)] // due to testing feature
                        let Node::Godot(sibling) = &mut sibling
                        else {
                            debug_assert!(
                              false,
                              "Node and sibling need to be the same type: Godot({:?}) vs Other({:?})",
                              self.get_id(),
                              sibling_id,
                            );
                            return;
                        };
                        debug_assert!(
                            sibling.get_parent().is_some(),
                            "Cannot mount {} after sibling {} without sibling",
                            self.get_id(),
                            sibling_id
                        );
                        sibling.add_sibling(&node.clone());
                    }
                    #[cfg(feature = "testing")]
                    Node::Test(node) => {
                        let Node::Test(sibling) = &mut sibling else {
                            debug_assert!(
                            false,
                            "Node and sibling need to be the same type: Test({:?}) vs Other({:?})",
                            self.get_id(),
                            sibling.get_id()
                          );
                            return;
                        };
                        sibling.add_sibling(node)
                    }
                }
            }
        }
    }

    fn mount_after(&mut self, sibling: &mut dyn Mountable) {
        log::trace!("mounting after sibling {}", self.get_id());
        sibling.mount(MountPlace::AfterSibling(self.clone()));
    }

    fn unmount(&mut self) {
        log::trace!("unmounting {}", self.get_id());
        match self {
            Node::Godot(node) => node.queue_free(),
            #[cfg(feature = "testing")]
            Node::Test(node) => node.unmount(),
        }
    }
}
