use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use rstml::node::{
    KVAttributeValue, KeyedAttribute, KeyedAttributeValue, Node as HtmlNode, NodeAttribute,
    NodeElement, NodeName,
};
use rstml::parse2;
use syn::{spanned::Spanned, Error, Ident, LitStr, Result};

type HtmlElement = NodeElement<rstml::node::Infallible>;

pub fn transform(input: TokenStream) -> Result<TokenStream> {
    let nodes = parse2(input)?;

    let tokens = match nodes.as_slice() {
        [single] => transform_node(single)?,
        slice => transform_fragment(slice)?,
    };

    Ok(tokens)
}

fn transform_node(node: &HtmlNode) -> Result<TokenStream> {
    match node {
        HtmlNode::Element(element) => transform_element(element),
        HtmlNode::Fragment(fragment) => transform_fragment(&fragment.children),
        HtmlNode::Text(text) => {
            let value = text.value.value();
            Ok(quote! { #value })
        }
        HtmlNode::RawText(raw) => {
            let value = raw.to_token_stream_string();
            Ok(quote! { #value })
        }
        HtmlNode::Block(block) => Ok(quote! { #block }),
        HtmlNode::Comment(_) => Ok(quote! { grui::prelude::empty() }),
        HtmlNode::Doctype(_) => Err(Error::new(
            Span::call_site(),
            "doctype nodes are not supported in grui templates",
        )),
        HtmlNode::Custom(_) => Err(Error::new(
            Span::call_site(),
            "custom rstml nodes are not supported",
        )),
    }
}

fn transform_fragment(children: &[HtmlNode]) -> Result<TokenStream> {
    let children = transform_children(children)?;
    Ok(make_fragment(children))
}

fn make_fragment(children: Vec<TokenStream>) -> TokenStream {
    quote! {
      ::grui::prelude::fragment()
        #(.child( #children ))*
    }
}

fn make_children(children: Vec<TokenStream>) -> Option<TokenStream> {
    match children.len() {
        0 => None,
        1 => {
            let child = &children[0];
            Some(quote! { #child })
        }
        _ => Some(make_fragment(children)),
    }
}

fn add_children(mut builder: TokenStream, children: Vec<TokenStream>) -> TokenStream {
    for child in &children {
        // FIXME: might need chunking
        builder = quote! { #builder.child( #child ) };
    }
    builder
}

fn transform_element(element: &HtmlElement) -> Result<TokenStream> {
    if is_for_tag(element.name())? {
        transform_for(element)
    } else if is_component_tag(element.name())? {
        transform_component(element)
    } else {
        transform_builtin(element)
    }
}

fn transform_builtin(element: &HtmlElement) -> Result<TokenStream> {
    let builder = builtin_builder(element.name())?;
    let mut builder = apply_attributes(builder, element)?;
    let children = transform_children(&element.children)?;
    builder = add_children(builder, children);
    Ok(quote! { #builder.build() })
}

fn transform_component(element: &HtmlElement) -> Result<TokenStream> {
    let (component_path, props_path) = component_paths(element.name())?;
    let mut fields = Vec::new();

    for attribute in element.open_tag.attributes.iter() {
        if let NodeAttribute::Attribute(attr) = attribute {
            let key = attribute_name(attr)?;
            if key == "children" {
                return Err(Error::new(
                    attr.span(),
                    "children must be provided as element contents, not as an attribute",
                ));
            }
            let field_ident = attribute_to_ident(&key);
            let value = attribute_value(attr, true)?;
            fields.push(quote! { #field_ident: #value });
        } else {
            return Err(Error::new(
                attribute.span(),
                "attribute blocks are not supported on components",
            ));
        }
    }

    let children = transform_children(&element.children)?;
    let children_expr = make_children(children);

    if let Some(children_expr) = children_expr {
        fields.push(quote! { children: #children_expr });
    }

    let props_literal = quote! {
        #props_path {
            #(#fields,)*
        }
    };

    Ok(quote! { #component_path(#props_literal) })
}

fn apply_attributes(mut builder: TokenStream, element: &HtmlElement) -> Result<TokenStream> {
    for attribute in element.open_tag.attributes.iter() {
        match attribute {
            NodeAttribute::Attribute(attr) => {
                let key = attribute_name(attr)?;
                if let Some(event) = key.strip_prefix("on:") {
                    let handler = attribute_value(attr, false)?;
                    let event_lit = LitStr::new(event, attr.key.span());
                    builder = quote! { #builder.on(#event_lit, #handler) };
                } else {
                    let value = attribute_value(attr, true)?;
                    let key_lit = LitStr::new(&key, attr.key.span());
                    builder = quote! { #builder.prop(#key_lit, #value) };
                }
            }
            NodeAttribute::Block(_) => {
                return Err(Error::new(
                    attribute.span(),
                    "attribute blocks are not supported for builtin controls",
                ));
            }
        }
    }

    Ok(builder)
}

fn transform_children(nodes: &[HtmlNode]) -> Result<Vec<TokenStream>> {
    nodes.iter().map(transform_node).collect()
}

fn is_for_tag(name: &NodeName) -> Result<bool> {
    const FOR_TAG: &str = "For";

    match name {
        NodeName::Path(path) => Ok(path_to_string(path) == FOR_TAG),
        NodeName::Punctuated(_) => Ok(false),
        NodeName::Block(_) => Ok(false),
    }
}

fn transform_for(element: &HtmlElement) -> Result<TokenStream> {
    let mut each_expr: Option<TokenStream> = None;
    let mut key_expr: Option<TokenStream> = None;
    let mut pattern_tokens: Option<TokenStream> = None;

    for attribute in element.open_tag.attributes.iter() {
        match attribute {
            NodeAttribute::Attribute(attr) => {
                let key = attribute_name(attr)?;
                match key.as_str() {
                    "each" => {
                        let expr = attribute_value(attr, false)?;
                        // require zero-arg closure; call it at runtime
                        each_expr = Some(quote! { (#expr)() });
                    }
                    "key" => {
                        let expr = attribute_value(attr, false)?;
                        key_expr = Some(quote! { #expr });
                    }
                    "let" => match &attr.possible_value {
                        KeyedAttributeValue::Binding(binding) => {
                            let tokens = quote! { #binding };
                            pattern_tokens = Some(tokens);
                        }
                        KeyedAttributeValue::Value(value_expr) => match &value_expr.value {
                            KVAttributeValue::Expr(expr) => {
                                pattern_tokens = Some(quote! { #expr });
                            }
                            KVAttributeValue::InvalidBraced(_) => {
                                return Err(Error::new(attr.span(), "invalid let pattern"));
                            }
                        },
                        KeyedAttributeValue::None => {
                            return Err(Error::new(attr.span(), "let(...) requires a pattern"));
                        }
                    },
                    other => {
                        return Err(Error::new(
                            attr.span(),
                            format!("unsupported attribute `{}` on <For>", other),
                        ));
                    }
                }
            }
            NodeAttribute::Block(_) => {
                return Err(Error::new(
                    attribute.span(),
                    "attribute blocks are not supported on <For>",
                ));
            }
        }
    }

    let each_expr = each_expr
        .ok_or_else(|| Error::new(element.name().span(), "<For> requires `each` attribute"))?;
    let key_expr = key_expr.unwrap_or_else(|| quote! { |_| () });
    let pattern = pattern_tokens.unwrap_or_else(|| quote! { __item });

    let children = transform_children(&element.children)?;
    let body = make_children(children).unwrap_or(quote! { ::grui::prelude::empty() });

    let output = quote! {
        ::grui::prelude::for_each(
            #each_expr,
            #key_expr,
            |#pattern| { #body }
        )
    };

    Ok(output)
}

fn is_component_tag(name: &NodeName) -> Result<bool> {
    match name {
        NodeName::Path(path) => {
            if let Some(segment) = path.path.segments.last() {
                Ok(segment
                    .ident
                    .to_string()
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false))
            } else {
                Ok(false)
            }
        }
        NodeName::Punctuated(_) => Ok(false),
        NodeName::Block(_) => Ok(false),
    }
}

fn builtin_builder(name: &NodeName) -> Result<TokenStream> {
    let lookup = match name {
        NodeName::Path(path) => path_to_string(path),
        NodeName::Punctuated(punct) => {
            return Err(Error::new(
                punct.span(),
                "punctuated tag names are not supported",
            ))
        }
        NodeName::Block(block) => {
            return Err(Error::new(
                block.span(),
                "dynamic tag names are not supported",
            ))
        }
    };

    let builder = match lookup.as_str() {
        "control" => quote! { ::grui::prelude::control() },
        "colorrect" | "color_rect" => quote! { ::grui::prelude::color_rect() },
        "itemlist" | "item_list" => quote! { ::grui::prelude::item_list() },
        "label" => quote! { ::grui::prelude::label() },
        "lineedit" | "line_edit" => quote! { ::grui::prelude::line_edit() },
        "menubar" | "menu_bar" => quote! { ::grui::prelude::menu_bar() },
        "ninepatchrect" | "nine_patch_rect" => quote! { ::grui::prelude::nine_patch_rect() },
        "panel" => quote! { ::grui::prelude::panel() },
        "referencerect" | "reference_rect" => quote! { ::grui::prelude::reference_rect() },
        "richtextlabel" | "rich_text_label" => quote! { ::grui::prelude::rich_text_label() },
        "tabbar" | "tab_bar" => quote! { ::grui::prelude::tab_bar() },
        "textedit" | "text_edit" => quote! { ::grui::prelude::text_edit() },
        "texturerect" | "texture_rect" => quote! { ::grui::prelude::texture_rect() },
        "tree" => quote! { ::grui::prelude::tree() },
        "videostreamplayer" | "video_stream_player" => {
            quote! { ::grui::prelude::video_stream_player() }
        }
        "hseparator" | "h_separator" => quote! { ::grui::prelude::h_separator() },
        "vseparator" | "v_separator" => quote! { ::grui::prelude::v_separator() },
        "progressbar" | "progress_bar" => quote! { ::grui::prelude::progress_bar() },
        "spinbox" | "spin_box" => quote! { ::grui::prelude::spin_box() },
        "textureprogressbar" | "texture_progress_bar" => {
            quote! { ::grui::prelude::texture_progress_bar() }
        }
        "hslider" | "h_slider" => quote! { ::grui::prelude::h_slider() },
        "vslider" | "v_slider" => quote! { ::grui::prelude::v_slider() },
        "hscrollbar" | "h_scroll_bar" => quote! { ::grui::prelude::h_scroll_bar() },
        "vscrollbar" | "v_scroll_bar" => quote! { ::grui::prelude::v_scroll_bar() },
        "button" => quote! { ::grui::prelude::button() },
        "linkbutton" | "link_button" => quote! { ::grui::prelude::link_button() },
        "texturebutton" | "texture_button" => quote! { ::grui::prelude::texture_button() },
        "checkbox" | "check_box" => quote! { ::grui::prelude::check_box() },
        "checkbutton" | "check_button" => quote! { ::grui::prelude::check_button() },
        "colorpickerbutton" | "color_picker_button" => {
            quote! { ::grui::prelude::color_picker_button() }
        }
        "menubutton" | "menu_button" => quote! { ::grui::prelude::menu_button() },
        "optionbutton" | "option_button" => quote! { ::grui::prelude::option_button() },
        "container" => quote! { ::grui::prelude::container() },
        "aspectratiocontainer" | "aspect_ratio_container" => {
            quote! { ::grui::prelude::aspect_ratio_container() }
        }
        "boxcontainer" | "box_container" => quote! { ::grui::prelude::box_container() },
        "vboxcontainer" | "v_box_container" => quote! { ::grui::prelude::v_box_container() },
        "hboxcontainer" | "h_box_container" => quote! { ::grui::prelude::h_box_container() },
        "colorpicker" | "color_picker" => quote! { ::grui::prelude::color_picker() },
        "centercontainer" | "center_container" => quote! { ::grui::prelude::center_container() },
        "editorproperty" | "editor_property" => quote! { ::grui::prelude::editor_property() },
        "flowcontainer" | "flow_container" => quote! { ::grui::prelude::flow_container() },
        "hflowcontainer" | "h_flow_container" => quote! { ::grui::prelude::h_flow_container() },
        "vflowcontainer" | "v_flow_container" => quote! { ::grui::prelude::v_flow_container() },
        "gridcontainer" | "grid_container" => quote! { ::grui::prelude::grid_container() },
        "margincontainer" | "margin_container" => quote! { ::grui::prelude::margin_container() },
        "panelcontainer" | "panel_container" => quote! { ::grui::prelude::panel_container() },
        "scrollcontainer" | "scroll_container" => quote! { ::grui::prelude::scroll_container() },
        "splitcontainer" | "split_container" => quote! { ::grui::prelude::split_container() },
        "hsplitcontainer" | "h_split_container" => quote! { ::grui::prelude::h_split_container() },
        "vsplitcontainer" | "v_split_container" => quote! { ::grui::prelude::v_split_container() },
        "subviewportcontainer" | "sub_viewport_container" => {
            quote! { ::grui::prelude::sub_viewport_container() }
        }
        "tabcontainer" | "tab_container" => quote! { ::grui::prelude::tab_container() },
        other => {
            return Err(Error::new(
                name.span(),
                format!("unknown builtin control `{}`", other),
            ))
        }
    };

    Ok(builder)
}

fn component_paths(name: &NodeName) -> Result<(TokenStream, syn::Path)> {
    match name {
        NodeName::Path(path) => {
            let component_path = path.path.clone();
            let last_segment = component_path
                .segments
                .last()
                .ok_or_else(|| Error::new(path.span(), "invalid component path"))?
                .ident
                .clone();
            let props_ident = format_ident!("{}Props", last_segment);
            let mut props_path = component_path.clone();
            if let Some(last) = props_path.segments.last_mut() {
                last.ident = props_ident;
            }
            Ok((quote! { #component_path }, props_path))
        }
        NodeName::Punctuated(_) | NodeName::Block(_) => {
            Err(Error::new(name.span(), "component tags must be paths"))
        }
    }
}

fn attribute_name(attr: &KeyedAttribute) -> Result<String> {
    if attr.key.is_block() {
        return Err(Error::new(
            attr.key.span(),
            "dynamic attribute names are not supported",
        ));
    }
    Ok(attr.key.to_string())
}

fn attribute_value(attr: &KeyedAttribute, allow_missing: bool) -> Result<TokenStream> {
    match &attr.possible_value {
        KeyedAttributeValue::Value(value_expr) => match &value_expr.value {
            KVAttributeValue::Expr(expr) => Ok(quote! { #expr }),
            KVAttributeValue::InvalidBraced(_) => {
                Err(Error::new(attr.span(), "invalid attribute expression"))
            }
        },
        KeyedAttributeValue::None => {
            if allow_missing {
                Ok(quote! { true })
            } else {
                Err(Error::new(attr.span(), "attribute value is required here"))
            }
        }
        KeyedAttributeValue::Binding(_) => Err(Error::new(
            attr.span(),
            "binding-style attributes are not supported",
        )),
    }
}

fn attribute_to_ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

fn path_to_string(path: &syn::ExprPath) -> String {
    path.path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::pretty;
    use pretty_assertions::assert_eq;

    #[test]
    fn simple_builtin_tree() {
        let input = quote! {
            <>
                <panel />
                <vboxcontainer>
                    <button on:click=resume text="Resume" />
                    <button text="Save" />
                    <>
                      <button text="Load" />
                    </>
                </vboxcontainer>
            </>
        };

        let output = transform(input).expect("transform ok");
        let expected = quote! {
            ::grui::prelude::fragment()
              .child(::grui::prelude::panel().build())
              .child(::grui::prelude::v_box_container()
                .child(::grui::prelude::button().on("click", resume).prop("text", "Resume").build())
                .child(::grui::prelude::button().prop("text", "Save").build())
                .child(
                  ::grui::prelude::fragment()
                    .child(::grui::prelude::button().prop("text", "Load").build())
                )
                .build()
              )
        };

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn simple_button_with_text() {
        let input = quote! {
            <button text="Click me" />
        };

        let output = transform(input).expect("transform ok");
        let expected = quote! {
            ::grui::prelude::button().prop("text", "Click me").build()
        };

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn button_with_on_pressed_event() {
        let input = quote! {
            <button on:pressed=on_pressed text="Save" />
        };

        let output = transform(input).expect("transform ok");
        let expected = quote! {
            ::grui::prelude::button().on("pressed", on_pressed).prop("text", "Save").build()
        };

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn button_with_inline_closure() {
        let input = quote! {
            <button on:pressed={Callable::from_fn(|| {
                counter.mutate(|c| *c += 1);
            })} text="Save" />
        };

        let output = transform(input).expect("transform ok");
        let expected = quote! {
            ::grui::prelude::button().on("pressed", { Callable::from_fn(| | { counter.mutate(|c| *c += 1); }) }).prop("text", "Save").build()
        };

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn label_with_text_attribute() {
        let input = quote! {
            <label text=format!("{} {}", title, i) />
        };

        let output = transform(input).expect("transform ok");
        let expected = quote! {
            ::grui::prelude::label().prop("text", format!("{} {}", title, i)).build()
        };

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn component_with_props() {
        let input = quote! {
            <MenuButton label="Resume" on_pressed={resume} />
        };

        let output = transform(input).expect("transform ok");
        let expected = quote! {
            MenuButton(MenuButtonProps {
                label: "Resume",
                on_pressed: { resume },
            })
        };

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn vboxcontainer_with_multiple_children() {
        let input = quote! {
            <vboxcontainer>
                <button text="One" />
                <button text="Two" />
                <button text="Three" />
            </vboxcontainer>
        };

        let output = transform(input).expect("transform ok");
        let expected = quote! {
            ::grui::prelude::v_box_container()
              .child(::grui::prelude::button().prop("text", "One").build())
              .child(::grui::prelude::button().prop("text", "Two").build())
              .child(::grui::prelude::button().prop("text", "Three").build())
              .build()
        };

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn block_expression_as_child() {
        let input = quote! {
            <vboxcontainer>
                {
                  (1..=10).map(|i| {
                      control! {
                          <label text=format!("{} {}", title, i) />
                      }
                  }).collect::<Vec<_>>()
                }
            </vboxcontainer>
        };

        let output = transform(input).expect("transform ok");
        let expected = quote! {
            ::grui::prelude::v_box_container().child(
              { (1..=10).map(|i| { control! { <label text=format!("{} {}", title, i) />} }).collect::<Vec<_> >() }
            ).build()
        };

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn for_macro_with_each_key_and_let() {
        let input = quote! {
            <For each=|| (1..=5) key=|i| *i let(i)>
                <label text=format!("Item {}", i) />
            </For>
        };
        let output = transform(input).expect("transform ok");
        let expected = quote! {
            ::grui::prelude::for_each(
                (| | (1..=5))(),
                |i| *i,
                |(i)| { ::grui::prelude::label().prop("text", format!("Item {}", i)).build() }
            )
        };
        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn empty_element() {
        let input = quote! {
            <panel />
        };

        let output = transform(input).expect("transform ok");
        let expected = quote! {
            ::grui::prelude::panel().build()
        };

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn if_statement() {
        let input = quote! {
            {move || if condition {
                control! { <button text="Has button" /> }.into_any()
            } else {
                control! { <label text="No button" /> }.into_any()
            }}
        };

        let output = transform(input).expect("transform ok");
        let expected = quote! {
            { move | | if condition {
                control! { <button text="Has button" /> }.into_any()
            } else {
                control! { <label text="No button" /> }.into_any()
            } }
        };

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn pass_children() {
        let input = quote! {
            <MyComp>
                <button text="Click me" />
            </MyComp>
        };

        let output = transform(input).expect("transform ok");
        let expected = quote! {
            MyComp(MyCompProps {
                children: ::grui::prelude::button().prop("text", "Click me").build(),
            })
        };

        assert_eq!(pretty(output), pretty(expected));
    }
}
