use godot::builtin::{GString, StringName};
use godot::classes::{Button, Control, Label, Node as GodotNode, Object}; // Limited set; remove unsupported classes
use godot::prelude::{Callable, Gd, Variant};
use godot::obj::NewAlloc;
use crate::node::{ElementNode, EventBinding, Node, Property, PropertyValue, TextNode};

/// Reactive renderer: builds a Godot subtree from a virtual UI tree. Performs
/// keyed + positional diffing for direct children and recursively for element
/// children (except Buttons which aggregate text and still rebuild fully).
pub struct Renderer {
    root: Gd<Control>,
    render_fn: Box<dyn FnMut() -> Node>,
    current: Node,
    mounted: Vec<Gd<Control>>,
}

impl Renderer {
    pub fn new(root: Gd<Control>, mut render_fn: impl FnMut() -> Node + 'static) -> Self {
        let initial = render_fn();
        let mut root_clone = root.clone();
        let mounted = mount_into_parent(&mut root_clone, &initial);
        Self { root, render_fn: Box::new(render_fn), current: initial, mounted }
    }

    pub fn process(&mut self, _delta: f64) {
        if crate::reactive::take_dirty() {
            let next = (self.render_fn)();
            self.rerender(next);
            crate::reactive::run_effects();
        }
    }

    pub fn from_component<C>(root: Gd<Control>, component: C) -> Self
    where
        C: crate::control::Component + 'static,
    {
        let mut component = component;
        Self::new(root, move || component.render())
    }

    fn rerender(&mut self, tree: Node) {
        let mut root_clone = self.root.clone();
        self.mounted = diff_mount_into_parent(&mut root_clone, &tree);
        self.current = tree;
    }
}

fn mount_into_parent(parent: &mut Gd<Control>, node: &Node) -> Vec<Gd<Control>> {
    let controls = instantiate_node(node, None);
    let mut parent_node = parent.clone().upcast::<GodotNode>();
    for control in controls.iter() {
        let child_node = control.clone().upcast::<GodotNode>();
        parent_node.add_child(&child_node);
    }
    controls
}

fn diff_mount_into_parent(parent: &mut Gd<Control>, node: &Node) -> Vec<Gd<Control>> {
    // Diff direct children under parent (root control). Virtual children are derived from the root node.
    let virtual_children = collect_virtual_children(node);
    diff_children_on_control(parent, &virtual_children)
}

// A representation of direct children controls this node would produce
#[derive(Clone)]
enum VirtualChild {
    Element(ElementNode),
    Text(TextNode, Option<crate::godot::VNodeKind>),
}

fn collect_virtual_children(node: &Node) -> Vec<VirtualChild> {
    collect_virtual_children_with_parent(node, None)
}

fn collect_virtual_children_with_parent(
    node: &Node,
    parent_kind: Option<crate::godot::VNodeKind>,
) -> Vec<VirtualChild> {
    match node {
        Node::Element(el) => vec![VirtualChild::Element(el.clone())],
        Node::Text(text) => vec![VirtualChild::Text(text.clone(), parent_kind)],
        Node::Fragment(children) => {
            let mut out = Vec::new();
            for child in children {
                out.extend(collect_virtual_children_with_parent(child, parent_kind));
            }
            out
        }
        Node::Empty => Vec::new(),
    }
}

fn diff_children_on_control(parent: &mut Gd<Control>, virtual_children: &[VirtualChild]) -> Vec<Gd<Control>> {
    let mut parent_node = parent.clone().upcast::<GodotNode>();

    // Capture existing children
    let mut existing_keyed: std::collections::HashMap<String, Gd<Control>> = std::collections::HashMap::new();
    let mut existing_unkeyed: Vec<Gd<Control>> = Vec::new();
    let child_count = parent_node.get_child_count();
    for i in 0..child_count {
        if let Some(child) = parent_node.get_child(i) {
            if let Ok(control) = child.clone().try_cast::<Control>() {
                let object = control.clone().upcast::<Object>();
                let key_variant = object.get_meta(&StringName::from("grui:key"));
                let key_str: String = key_variant.to_string().to_string();
                if !key_variant.is_nil() && !key_str.is_empty() {
                    existing_keyed.insert(key_str, control);
                } else {
                    existing_unkeyed.push(control);
                }
            }
        }
    }

    // Detach all existing (we will re-add in new order and free leftovers)
    for i in (0..child_count).rev() {
        if let Some(child) = parent_node.get_child(i) { parent_node.remove_child(&child); }
    }

    let mut new_controls: Vec<Gd<Control>> = Vec::new();
    let mut unkeyed_reuse_index = 0usize;

    for vchild in virtual_children.iter() {
        match vchild.clone() {
            VirtualChild::Element(element) => {
                if let Some(ref key) = element.key {
                    if let Some(mut reused) = existing_keyed.remove(key) {
                        patch_element_on_control(&mut reused, &element);
                        let reused_node = reused.clone().upcast::<GodotNode>();
                        parent_node.add_child(&reused_node);
                        new_controls.push(reused);
                        continue;
                    }
                } else if unkeyed_reuse_index < existing_unkeyed.len() {
                    let mut reused = existing_unkeyed[unkeyed_reuse_index].clone();
                    unkeyed_reuse_index += 1;
                    patch_element_on_control(&mut reused, &element);
                    let reused_node = reused.clone().upcast::<GodotNode>();
                    parent_node.add_child(&reused_node);
                    new_controls.push(reused);
                    continue;
                }
                // Create new
                let mut instance = instantiate_element(&element);
                let instance_node = instance.clone().upcast::<GodotNode>();
                parent_node.add_child(&instance_node);
                new_controls.push(instance);
            }
            VirtualChild::Text(text, parent_kind) => {
                // Text nodes are never reused (no key). Positional reuse would require adding meta; skip for now.
                let controls = instantiate_text_node(&text, parent_kind);
                for c in controls.into_iter() {
                    let node = c.clone().upcast::<GodotNode>();
                    parent_node.add_child(&node);
                    new_controls.push(c);
                }
            }
        }
    }

    // Free leftover keyed
    for (_k, leftover) in existing_keyed.into_iter() {
        let mut node = leftover.clone().upcast::<GodotNode>();
        node.queue_free();
    }
    // Free leftover unkeyed not reused
    for leftover in existing_unkeyed.into_iter().skip(unkeyed_reuse_index) {
        let mut node = leftover.clone().upcast::<GodotNode>();
        node.queue_free();
    }

    new_controls
}

fn instantiate_node(node: &Node, parent_kind: Option<crate::godot::VNodeKind>) -> Vec<Gd<Control>> {
    match node {
        Node::Element(element) => { vec![instantiate_element(element)] }
        Node::Text(text) => instantiate_text_node(text, parent_kind),
        Node::Fragment(children) => children.iter().flat_map(|child| instantiate_node(child, parent_kind)).collect(),
        Node::Empty => Vec::new(),
    }
}

fn instantiate_element(element: &ElementNode) -> Gd<Control> {
    let mut instance = element.kind.create_instance();
    if let Some(ref key) = element.key {
        let mut object = instance.clone().upcast::<Object>();
        object.set_meta(&StringName::from("grui:key"), &Variant::from(GString::from(key.clone())));
    }
    apply_properties(&mut instance, element);
    apply_events_initial(&mut instance, &element.events);
    mount_children(&mut instance, element);
    instance
}

fn patch_element_on_control(instance: &mut Gd<Control>, element: &ElementNode) {
    if let Some(ref key) = element.key {
        let mut object = instance.clone().upcast::<Object>();
        object.set_meta(&StringName::from("grui:key"), &Variant::from(GString::from(key.clone())));
    }
    // Patch properties (diffed internally)
    apply_properties(instance, element);
    // Patch events (add new, disconnect removed)
    patch_events(instance, &element.events);

    match element.kind {
        crate::godot::VNodeKind::Button => {
            // Button still rebuilds all children due to aggregated text behavior
            clear_all_children(instance);
            mount_button_children(instance, &element.children);
        }
        _ => {
            // Diff children recursively
            let virtual_children: Vec<VirtualChild> = element
                .children
                .iter()
                .flat_map(|c| collect_virtual_children_with_parent(c, Some(element.kind)))
                .collect();
            diff_children_on_control(instance, &virtual_children);
        }
    }
}

fn clear_all_children(parent: &mut Gd<Control>) {
    let mut parent_node = parent.clone().upcast::<GodotNode>();
    let count = parent_node.get_child_count();
    for i in (0..count).rev() {
        if let Some(mut child) = parent_node.get_child(i) {
            parent_node.remove_child(&child);
            child.queue_free();
        }
    }
}

fn mount_children(parent: &mut Gd<Control>, element: &ElementNode) {
    match element.kind {
        crate::godot::VNodeKind::Button => mount_button_children(parent, &element.children),
        _ => {
            for child in &element.children {
                let controls = instantiate_node(child, Some(element.kind));
                for control in controls { attach_child(parent, &control); }
            }
        }
    }
}

fn mount_button_children(button: &mut Gd<Control>, children: &[Node]) {
    let mut text_value = String::new();
    for child in children {
        match child {
            Node::Text(text) => text_value.push_str(&text.value),
            Node::Fragment(fragment_children) => mount_button_children(button, fragment_children),
            Node::Element(element) => {
                let controls = instantiate_node(&Node::Element(element.clone()), Some(crate::godot::VNodeKind::Button));
                for control in controls { attach_child(button, &control); }
            }
            Node::Empty => {}
        }
    }
    if !text_value.is_empty() {
        if let Ok(mut button_control) = button.clone().try_cast::<Button>() {
            button_control.set_text(&GString::from(text_value));
        }
    }
}

fn attach_child(parent: &mut Gd<Control>, child: &Gd<Control>) {
    let mut parent_node = parent.clone().upcast::<GodotNode>();
    let child_node = child.clone().upcast::<GodotNode>();
    parent_node.add_child(&child_node);
}

fn instantiate_text_node(text: &TextNode, parent_kind: Option<crate::godot::VNodeKind>) -> Vec<Gd<Control>> {
    match parent_kind {
        Some(crate::godot::VNodeKind::Button) => Vec::new(),
        _ => {
            let mut label = Label::new_alloc();
            label.set_text(&text.as_gstring());
            vec![label.upcast()]
        }
    }
}

fn apply_properties(instance: &mut Gd<Control>, element: &ElementNode) {
    // Build new property map (name -> string serialization) for diffing
    let mut new_prop_map: std::collections::HashMap<&str, String> = std::collections::HashMap::new();
    for property in &element.props {
        let serialized = match &property.value {
            PropertyValue::Bool(v) => v.to_string(),
            PropertyValue::I64(v) => v.to_string(),
            PropertyValue::F64(v) => v.to_string(),
            PropertyValue::String(s) => s.to_string(),
            PropertyValue::Variant(v) => format!("{:?}", v),
        };
        new_prop_map.insert(property.name, serialized);
    }

    let mut object = instance.clone().upcast::<Object>();
    let existing_variant = object.get_meta(&StringName::from("grui:props"));
    let existing_map: std::collections::HashMap<String, String> = if existing_variant.is_nil() {
        std::collections::HashMap::new()
    } else {
        let raw = existing_variant.to_string().to_string();
        raw.split(';').filter(|s| !s.is_empty()).filter_map(|pair| {
            if let Some((k,v)) = pair.split_once('=') { Some((k.to_string(), v.to_string())) } else { None }
        }).collect()
    };

    // Apply changes and new properties
    for property in &element.props {
        let prev = existing_map.get(property.name);
        let curr = new_prop_map.get(property.name).unwrap();
        if prev.map(|p| p != curr).unwrap_or(true) {
            match property.name {
                "name" => apply_name(instance, property),
                "text" => apply_text_property(instance, element.kind, property),
                _ => { eprintln!("[grui] warn: property `{}` not yet supported for {:?}", property.name, element.kind); }
            }
        }
    }

    // Removal handling currently skipped (could reset defaults per kind)

    // Persist new map
    let serialized: String = new_prop_map.iter().map(|(k,v)| format!("{}={}", k, v.replace(';', "/;").replace('=', "/="))).collect::<Vec<String>>().join(";");
    object.set_meta(&StringName::from("grui:props"), &Variant::from(GString::from(serialized)));
}

fn apply_name(instance: &mut Gd<Control>, property: &Property) {
    if let Some(value) = property.value.as_gstring() {
        let mut node = instance.clone().upcast::<GodotNode>();
        node.set_name(&value);
    } else {
        eprintln!("[grui] error: expected string for `name` property");
    }
}

fn apply_text_property(instance: &mut Gd<Control>, kind: crate::godot::VNodeKind, property: &Property) {
    if let Some(value) = property.value.as_gstring() {
        match kind {
            crate::godot::VNodeKind::Button => { if let Ok(mut button) = instance.clone().try_cast::<Button>() { button.set_text(&value); } }
            crate::godot::VNodeKind::Label => { if let Ok(mut label) = instance.clone().try_cast::<Label>() { label.set_text(&value); } }
            _ => { eprintln!("[grui] warn: `text` property not handled for {:?}", kind); }
        }
    } else {
        eprintln!("[grui] error: expected string value for text property");
    }
}

fn apply_events_initial(instance: &mut Gd<Control>, events: &[EventBinding]) {
    if events.is_empty() { return; }
    let mut object = instance.clone().upcast::<Object>();
    let mut names: Vec<&str> = Vec::new();
    for event in events {
        let signal = StringName::from(event.descriptor.name);
        let result = object.connect(&signal, &event.callable);
        if result != godot::global::Error::OK { eprintln!("[grui] warn: failed to connect signal `{}`: {:?}", event.descriptor.name, result); }
        let meta_key = format!("grui:event:{}", event.descriptor.name);
        object.set_meta(&StringName::from(meta_key), &Variant::from(event.callable.clone()));
        names.push(event.descriptor.name);
    }
    object.set_meta(&StringName::from("grui:events"), &Variant::from(GString::from(names.join(","))));
}

fn patch_events(instance: &mut Gd<Control>, events: &[EventBinding]) {
    let mut object = instance.clone().upcast::<Object>();
    let existing_variant = object.get_meta(&StringName::from("grui:events"));
    let existing_list = if existing_variant.is_nil() { Vec::new() } else { existing_variant.to_string().to_string().split(',').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect::<Vec<String>>() };
    let existing_set: std::collections::HashSet<&str> = existing_list.iter().map(|s| s.as_str()).collect();
    let new_set: std::collections::HashSet<&str> = events.iter().map(|e| e.descriptor.name).collect();

    // Disconnect removed events
    for removed in existing_list.iter().filter(|n| !new_set.contains(n.as_str())) {
        let meta_key = format!("grui:event:{}", removed);
        let callable_variant = object.get_meta(&StringName::from(meta_key.clone()));
        if !callable_variant.is_nil() {
            if let Ok(callable) = callable_variant.try_to::<Callable>() {
                object.disconnect(&StringName::from(removed.as_str()), &callable);
            }
        }
        object.set_meta(&StringName::from(meta_key), &Variant::nil());
    }

    // Add newly added events
    for event in events {
        if !existing_set.contains(event.descriptor.name) {
            let signal = StringName::from(event.descriptor.name);
            let result = object.connect(&signal, &event.callable);
            if result != godot::global::Error::OK { eprintln!("[grui] warn: failed to connect (patch) signal `{}`: {:?}", event.descriptor.name, result); }
            let meta_key = format!("grui:event:{}", event.descriptor.name);
            object.set_meta(&StringName::from(meta_key), &Variant::from(event.callable.clone()));
        } else {
            // Could detect changed handler identity; skipped for now.
        }
    }

    // Rebuild meta list from new events
    let final_names: Vec<&str> = events.iter().map(|e| e.descriptor.name).collect();
    object.set_meta(&StringName::from("grui:events"), &Variant::from(GString::from(final_names.join(","))));
}
