use godot::builtin::GString;

use crate::godot::VNodeKind;
use crate::node::{ElementBuilder, PropertyValue};

pub fn control() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::Control)
}

pub fn color_rect() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::ColorRect)
}

pub fn graph_edit() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::GraphEdit)
}

pub fn item_list() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::ItemList)
}

pub fn label() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::Label)
}

pub fn line_edit() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::LineEdit)
}

pub fn menu_bar() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::MenuBar)
}

pub fn nine_patch_rect() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::NinePatchRect)
}

pub fn panel() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::Panel)
}

pub fn reference_rect() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::ReferenceRect)
}

pub fn rich_text_label() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::RichTextLabel)
}

pub fn tab_bar() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::TabBar)
}

pub fn text_edit() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::TextEdit)
}

pub fn texture_rect() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::TextureRect)
}

pub fn tree() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::Tree)
}

pub fn video_stream_player() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::VideoStreamPlayer)
}

pub fn h_separator() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::HSeparator)
}

pub fn v_separator() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::VSeparator)
}

pub fn progress_bar() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::ProgressBar)
}

pub fn spin_box() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::SpinBox)
}

pub fn texture_progress_bar() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::TextureProgressBar)
}

pub fn h_slider() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::HSlider)
}

pub fn v_slider() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::VSlider)
}

pub fn h_scroll_bar() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::HScrollBar)
}

pub fn v_scroll_bar() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::VScrollbar)
}

pub fn button() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::Button)
}

pub fn link_button() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::LinkButton)
}

pub fn texture_button() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::TextureButton)
}

pub fn check_box() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::CheckBox)
}

pub fn check_box_button() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::CheckBoxButton)
}

pub fn color_picker_button() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::ColorPickerButton)
}

pub fn menu_button() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::MenuButton)
}

pub fn option_button() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::OptionButton)
}

pub fn container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::Container)
}

pub fn aspect_ratio_container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::AspectRatioContainer)
}

pub fn box_container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::BoxContainer)
}

pub fn vbox_container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::VBoxContainer)
}

pub fn hbox_container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::HBoxContainer)
}

pub fn color_picker() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::ColorPicker)
}

pub fn center_container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::CenterContainer)
}

pub fn editor_property() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::EditorProperty)
}

pub fn flow_container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::FlowContainer)
}

pub fn h_flow_container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::HFlowContainer)
}

pub fn v_flow_container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::VFlowContainer)
}

pub fn graph_element() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::GraphElement)
}

pub fn graph_frame() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::GraphFrame)
}

pub fn graph_node() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::GraphNode)
}

pub fn grid_container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::GridContainer)
}

pub fn margin_container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::MarginContainer)
}

pub fn panel_container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::PanelContainer)
}

pub fn scroll_container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::ScrollContainer)
}

pub fn split_container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::SplitContainer)
}

pub fn h_split_container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::HSplitContainer)
}

pub fn v_split_container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::VSplitContainer)
}

pub fn sub_viewport_container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::SubViewportContainer)
}

pub fn tab_container() -> ElementBuilder {
    ElementBuilder::new(VNodeKind::TabContainer)
}

pub fn text<T>(value: T) -> PropertyValue
where
    T: Into<GString>,
{
    PropertyValue::from(value.into())
}
