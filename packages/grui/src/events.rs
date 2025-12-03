use godot::prelude::Callable;

use crate::node::{EventBinding, EventDescriptor};

pub const PRESSED: EventDescriptor = EventDescriptor::new("pressed");
pub const CLICK: EventDescriptor = EventDescriptor::new("pressed");
pub const BUTTON_UP: EventDescriptor = EventDescriptor::new("button_up");
pub const BUTTON_DOWN: EventDescriptor = EventDescriptor::new("button_down");

pub fn descriptor(name: &'static str) -> EventDescriptor {
    EventDescriptor::new(name)
}

pub fn binding(descriptor: EventDescriptor, handler: Callable) -> EventBinding {
    EventBinding::new(descriptor, handler)
}
