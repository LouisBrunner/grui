// FIXME: retain for refactor
// /*!
// Property descriptor helpers.

// This module provides a small, easily-extensible descriptor of commonly-used
// properties and basic signal descriptors per `VNodeKind`. The intent is to give
// the renderer and tests a central, static source of truth about which properties
// and signals are expected for a given virtual node kind.

// This is intentionally conservative — the long-term plan is to generate a
// complete descriptor from Godot's class metadata (codegen). For now this
// descriptor reduces hardcoding in the renderer and makes tests predictable.
// */
// use crate::godot::VNodeKind;

// /// Returns true if the given `VNodeKind` is expected to support a property
// /// with the provided `name`.
// ///
// /// The check is conservative: unknown kinds fall back to a small set of generic
// /// properties. Extend the static arrays below as needed.
// pub fn supports_property(kind: VNodeKind, name: &str) -> bool {
//     match kind {
//         VNodeKind::Control | VNodeKind::Omni => CONTROL_PROPS.contains(&name),
//         VNodeKind::Button | VNodeKind::LinkButton | VNodeKind::TextureButton => {
//             BUTTON_PROPS.contains(&name)
//         }
//         VNodeKind::Label | VNodeKind::RichTextLabel => LABEL_PROPS.contains(&name),
//         VNodeKind::LineEdit => LINE_EDIT_PROPS.contains(&name),
//         VNodeKind::VBoxContainer | VNodeKind::HBoxContainer | VNodeKind::BoxContainer => {
//             BOX_CONTAINER_PROPS.contains(&name)
//         }
//         VNodeKind::MarginContainer => MARGIN_CONTAINER_PROPS.contains(&name),
//         VNodeKind::TextureRect | VNodeKind::NinePatchRect | VNodeKind::TextureButton => {
//             TEXTURE_PROPS.contains(&name)
//         }
//         VNodeKind::ScrollContainer | VNodeKind::PanelContainer | VNodeKind::Container => {
//             CONTAINER_PROPS.contains(&name)
//         }
//         // Fallback to a small set of generic properties for other kinds.
//         _ => GENERIC_PROPS.contains(&name),
//     }
// }

// /// Returns true if the given `VNodeKind` is expected to emit a signal with the
// /// provided `name`.
// ///
// /// This is a minimal signal descriptor to support renderer event wiring and
// /// tests. Expand or generate from Godot metadata later.
// pub fn supports_signal(kind: VNodeKind, name: &str) -> bool {
//     match kind {
//         VNodeKind::Button | VNodeKind::CheckBox | VNodeKind::TextureButton => {
//             BUTTON_SIGNALS.contains(&name)
//         }
//         VNodeKind::LineEdit => LINE_EDIT_SIGNALS.contains(&name),
//         VNodeKind::Tree => TREE_SIGNALS.contains(&name),
//         VNodeKind::Control | VNodeKind::Container | VNodeKind::Omni => {
//             CONTROL_SIGNALS.contains(&name)
//         }
//         _ => GENERIC_SIGNALS.contains(&name),
//     }
// }

// static CONTROL_PROPS: &[&str] = &[
//     "name",
//     "visible",
//     "disabled",
//     "rect_min_size",
//     "rect_size",
//     "rect_position",
//     "mouse_filter",
// ];

// static BUTTON_PROPS: &[&str] = &[
//     "text",
//     "disabled",
//     "flat",
//     "toggle_mode",
//     "pressed",
//     "toggle_mode",
//     "icon",
// ];

// static LABEL_PROPS: &[&str] = &["text", "align", "valign", "autowrap", "clips_text"];

// static LINE_EDIT_PROPS: &[&str] = &[
//     "text",
//     "placeholder_text",
//     "readonly",
//     "max_length",
//     "secret",
//     "clear_button_enabled",
// ];

// static BOX_CONTAINER_PROPS: &[&str] = &["separation", "alignment", "custom_constants"];

// static MARGIN_CONTAINER_PROPS: &[&str] =
//     &["margin_left", "margin_top", "margin_right", "margin_bottom"];

// static TEXTURE_PROPS: &[&str] = &["texture", "expand", "stretch_mode", "flip_h", "flip_v"];

// static CONTAINER_PROPS: &[&str] = &["custom_minimum_size", "clip_contents", "mouse_filter"];

// static GENERIC_PROPS: &[&str] = &["name", "visible", "custom_minimum_size"];

// //
// // Basic signal lists (extend as needed)
// //
// static BUTTON_SIGNALS: &[&str] = &["pressed", "toggled", "mouse_entered", "mouse_exited"];
// static LINE_EDIT_SIGNALS: &[&str] = &["text_changed", "text_entered", "request_completion"];
// static TREE_SIGNALS: &[&str] = &["item_activated", "cell_selected"];
// static CONTROL_SIGNALS: &[&str] = &["mouse_entered", "mouse_exited", "ready"];
// static GENERIC_SIGNALS: &[&str] = &["ready"];

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::godot::VNodeKind;

//     #[test]
//     fn known_props_supported() {
//         assert!(supports_property(VNodeKind::Button, "text"));
//         assert!(supports_property(VNodeKind::Label, "text"));
//         assert!(supports_property(VNodeKind::Control, "name"));
//         assert!(supports_property(VNodeKind::Omni, "name"));
//         assert!(supports_property(VNodeKind::VBoxContainer, "separation"));
//         assert!(supports_property(VNodeKind::MarginContainer, "margin_left"));
//         // newly added props
//         assert!(supports_property(VNodeKind::TextureRect, "texture"));
//         assert!(supports_property(VNodeKind::Container, "clip_contents"));
//     }

//     #[test]
//     fn unknown_props_rejected_by_default() {
//         assert!(!supports_property(VNodeKind::Button, "i_do_not_exist"));
//         assert!(!supports_property(VNodeKind::Label, "not_a_prop"));
//     }

//     #[test]
//     fn generic_fallback_allows_basic_props() {
//         // Pick a kind not explicitly enumerated above and verify generic props pass.
//         // Use TabContainer as a representative example (it falls into the `_` arm).
//         assert!(supports_property(VNodeKind::TabContainer, "name"));
//         assert!(!supports_property(
//             VNodeKind::TabContainer,
//             "completely_unknown"
//         ));
//     }

//     #[test]
//     fn basic_signals_supported() {
//         assert!(supports_signal(VNodeKind::Button, "pressed"));
//         assert!(supports_signal(VNodeKind::LineEdit, "text_changed"));
//         assert!(supports_signal(VNodeKind::Control, "ready"));
//         assert!(!supports_signal(
//             VNodeKind::Button,
//             "this_signal_does_not_exist"
//         ));
//     }
// }
