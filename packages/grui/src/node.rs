use std::fmt;

use godot::builtin::GString;
use godot::prelude::{Callable, Variant};

use crate::godot::VNodeKind;

/// A virtual node produced by the `control!` macro.
#[derive(Clone)]
pub enum Node {
    Element(ElementNode),
    Text(TextNode),
    Fragment(Vec<Node>),
    Empty,
}

impl Node {
    pub fn element(node: ElementNode) -> Self {
        Self::Element(node)
    }

    pub fn text<T>(value: T) -> Self
    where
        T: Into<String>,
    {
        Self::Text(TextNode::new(value))
    }

    pub fn fragment(children: Vec<Node>) -> Self {
        Self::Fragment(children)
    }

    pub fn empty() -> Self {
        Self::Empty
    }
}

impl std::iter::FromIterator<Node> for Node {
    fn from_iter<T: IntoIterator<Item = Node>>(iter: T) -> Self {
        let children: Vec<Node> = iter.into_iter().collect();
        Node::fragment(children)
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Element(element) => f
                .debug_struct("Element")
                .field("kind", &element.kind)
                .field("props", &element.props)
                .field("events", &element.events)
                .field("children", &element.children)
                .finish(),
            Node::Text(text) => f.debug_tuple("Text").field(text).finish(),
            Node::Fragment(children) => f.debug_tuple("Fragment").field(children).finish(),
            Node::Empty => f.debug_tuple("Empty").finish(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ElementNode {
    pub kind: VNodeKind,
    pub key: Option<String>,
    pub props: Vec<Property>,
    pub events: Vec<EventBinding>,
    pub children: Vec<Node>,
}

impl ElementNode {
    pub fn new(kind: VNodeKind) -> Self {
        Self {
            kind,
            key: None,
            props: Vec::new(),
            events: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn with_key(mut self, key: Option<String>) -> Self {
        self.key = key;
        self
    }

    pub fn with_props(mut self, props: Vec<Property>) -> Self {
        self.props = props;
        self
    }

    pub fn with_events(mut self, events: Vec<EventBinding>) -> Self {
        self.events = events;
        self
    }

    pub fn with_children(mut self, children: Vec<Node>) -> Self {
        self.children = children;
        self
    }
}

#[derive(Clone, Debug)]
pub struct TextNode {
    pub value: String,
}

impl TextNode {
    pub fn new<T>(value: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            value: value.into(),
        }
    }

    pub fn as_gstring(&self) -> GString {
        GString::from(self.value.clone())
    }
}

#[derive(Clone, Debug)]
pub struct Property {
    pub name: &'static str,
    pub value: PropertyValue,
}

impl Property {
    pub fn new(name: &'static str, value: PropertyValue) -> Self {
        Self { name, value }
    }
}

#[derive(Clone, Debug)]
pub enum PropertyValue {
    Bool(bool),
    I64(i64),
    F64(f64),
    String(GString),
    Variant(Variant),
}

impl From<bool> for PropertyValue {
    fn from(value: bool) -> Self {
        PropertyValue::Bool(value)
    }
}

impl From<i32> for PropertyValue {
    fn from(value: i32) -> Self {
        PropertyValue::I64(value as i64)
    }
}

impl From<i64> for PropertyValue {
    fn from(value: i64) -> Self {
        PropertyValue::I64(value)
    }
}

impl From<f32> for PropertyValue {
    fn from(value: f32) -> Self {
        PropertyValue::F64(value as f64)
    }
}

impl From<f64> for PropertyValue {
    fn from(value: f64) -> Self {
        PropertyValue::F64(value)
    }
}

impl From<String> for PropertyValue {
    fn from(value: String) -> Self {
        PropertyValue::String(GString::from(value))
    }
}

impl From<&str> for PropertyValue {
    fn from(value: &str) -> Self {
        PropertyValue::String(GString::from(value))
    }
}

impl From<GString> for PropertyValue {
    fn from(value: GString) -> Self {
        PropertyValue::String(value)
    }
}

impl From<Variant> for PropertyValue {
    fn from(value: Variant) -> Self {
        PropertyValue::Variant(value)
    }
}

impl PropertyValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PropertyValue::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            PropertyValue::I64(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            PropertyValue::F64(value) => Some(*value),
            PropertyValue::I64(value) => Some(*value as f64),
            _ => None,
        }
    }

    pub fn as_gstring(&self) -> Option<GString> {
        match self {
            PropertyValue::String(value) => Some(value.clone()),
            _ => None,
        }
    }

    pub fn into_variant(self) -> Variant {
        match self {
            PropertyValue::Bool(value) => Variant::from(value),
            PropertyValue::I64(value) => Variant::from(value),
            PropertyValue::F64(value) => Variant::from(value),
            PropertyValue::String(value) => Variant::from(value),
            PropertyValue::Variant(value) => value,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EventBinding {
    pub descriptor: EventDescriptor,
    pub callable: Callable,
}

impl EventBinding {
    pub fn new(descriptor: EventDescriptor, callable: Callable) -> Self {
        Self {
            descriptor,
            callable,
        }
    }
}

/// A descriptor for a Godot signal that components may listen to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventDescriptor {
    pub name: &'static str,
}

impl EventDescriptor {
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }
}

/// Helper trait to convert a value into a [`Node`].
pub trait IntoNode {
    fn into_node(self) -> Node;
}

impl IntoNode for Node {
    fn into_node(self) -> Node {
        self
    }
}

impl IntoNode for ElementNode {
    fn into_node(self) -> Node {
        Node::element(self)
    }
}

impl IntoNode for ElementBuilder {
    fn into_node(self) -> Node {
        self.build()
    }
}

impl IntoNode for TextNode {
    fn into_node(self) -> Node {
        Node::Text(self)
    }
}

impl IntoNode for &str {
    fn into_node(self) -> Node {
        Node::text(self)
    }
}

impl IntoNode for String {
    fn into_node(self) -> Node {
        Node::text(self)
    }
}

impl IntoNode for GString {
    fn into_node(self) -> Node {
        Node::text(self.to_string())
    }
}

// Alias used for generated component props. Keep as an alias to Node so existing
// builder behavior remains the same but macros can refer to `Children`.
pub type Children = Node;

/// Trait used as the return type of functional components.
pub trait IntoControl {
    fn into_control(self) -> Node;
}

impl<T> IntoControl for T
where
    T: IntoNode,
{
    fn into_control(self) -> Node {
        self.into_node()
    }
}

/// Helper trait to convert values into `Vec<Node>` for the `.children(...)`
/// builder API.
pub trait IntoChildren {
    fn into_children(self) -> Vec<Node>;
}

impl IntoChildren for () {
    fn into_children(self) -> Vec<Node> {
        Vec::new()
    }
}

impl<T> IntoChildren for T
where
    T: IntoNode,
{
    fn into_children(self) -> Vec<Node> {
        vec![self.into_node()]
    }
}

macro_rules! impl_into_children_tuple {
    ($($ty:ident),+ $(,)?) => {
        impl<$($ty),+> IntoChildren for ($($ty,)+)
        where
            $($ty: IntoNode,)+
        {
            fn into_children(self) -> Vec<Node> {
                let ($($ty,)+) = self;
                let mut children = Vec::new();
                $(
                    children.push($ty.into_node());
                )+
                children
            }
        }
    };
}

impl_into_children_tuple!(A, B);
impl_into_children_tuple!(A, B, C);
impl_into_children_tuple!(A, B, C, D);
impl_into_children_tuple!(A, B, C, D, E);
impl_into_children_tuple!(A, B, C, D, E, F);
impl_into_children_tuple!(A, B, C, D, E, F, G);
impl_into_children_tuple!(A, B, C, D, E, F, G, H);

/// A builder for [`ElementNode`]s, exposed through the `grui::classes::*`
/// helpers.
#[derive(Clone, Debug)]
pub struct ElementBuilder {
    kind: VNodeKind,
    key: Option<String>,
    props: Vec<Property>,
    events: Vec<EventBinding>,
}

impl ElementBuilder {
    pub fn new(kind: VNodeKind) -> Self {
        Self {
            kind,
            key: None,
            props: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn prop(mut self, name: &'static str, value: impl Into<PropertyValue>) -> Self {
        self.props.push(Property::new(name, value.into()));
        self
    }

    pub fn event(mut self, binding: EventBinding) -> Self {
        self.events.push(binding);
        self
    }

    pub fn on(mut self, descriptor: EventDescriptor, handler: Callable) -> Self {
        self.events.push(EventBinding::new(descriptor, handler));
        self
    }

    pub fn build(self) -> Node {
        Node::element(
            ElementNode::new(self.kind)
                .with_key(self.key)
                .with_props(self.props)
                .with_events(self.events),
        )
    }

    pub fn children(self, children: impl IntoChildren) -> Node {
        Node::element(
            ElementNode::new(self.kind)
                .with_key(self.key)
                .with_props(self.props)
                .with_events(self.events)
                .with_children(children.into_children()),
        )
    }
}
