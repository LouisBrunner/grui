use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use rstml::node::{
    KVAttributeValue, KeyedAttribute, KeyedAttributeValue, Node as HtmlNode, NodeAttribute,
    NodeElement, NodeFragment, NodeName,
};
use rstml::parse2;
use syn::{spanned::Spanned, Error, Ident, LitStr, Result};

type HtmlElement = NodeElement<rstml::node::Infallible>;
type HtmlFragment = NodeFragment<rstml::node::Infallible>;

pub fn transform(input: TokenStream) -> Result<TokenStream> {
    let nodes = parse2(input)?;

    let tokens = match nodes.as_slice() {
        [single] => transform_node(single)?,
        slice => {
            let children = transform_children(slice)?;
            quote! { grui::node::Node::fragment(vec![#(#children),*]) }
        }
    };

    Ok(tokens)
}

fn transform_node(node: &HtmlNode) -> Result<TokenStream> {
    match node {
        HtmlNode::Element(element) => transform_element(element),
        HtmlNode::Text(text) => {
            let value = text.value.value();
            Ok(quote! { grui::node::Node::text(#value) })
        }
        HtmlNode::Fragment(fragment) => transform_fragment(fragment),
        HtmlNode::Block(block) => {
            let block_tokens = quote! { #block };
            Ok(quote! { grui::node::IntoNode::into_node(#block_tokens) })
        }
        HtmlNode::RawText(raw) => {
            let value = raw.to_token_stream_string();
            Ok(quote! { grui::node::Node::text(#value) })
        }
        HtmlNode::Comment(_) => Ok(quote! { grui::node::Node::empty() }),
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

fn transform_fragment(fragment: &HtmlFragment) -> Result<TokenStream> {
    let children = transform_children(&fragment.children)?;
    Ok(quote! { grui::node::Node::fragment(vec![#(#children),*]) })
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
    let builder = apply_attributes(builder, element)?;
    let children = transform_children(&element.children)?;

    let output = match children.len() {
        0 => quote! { grui::node::IntoNode::into_node(#builder.build()) },
        1 => {
            let child = &children[0];
            quote! { grui::node::IntoNode::into_node(#builder.children(#child)) }
        }
        _ => {
            let tuple = quote! { (#(#children),*) };
            quote! { grui::node::IntoNode::into_node(#builder.children(#tuple)) }
        }
    };

    Ok(output)
}

fn transform_component(element: &HtmlElement) -> Result<TokenStream> {
    let (component_path, props_path) = component_paths(element.name())?;
    let mut fields = Vec::new();

    for attribute in element.open_tag.attributes.iter() {
        if let NodeAttribute::Attribute(attr) = attribute {
            let key = attr_name(attr)?;
            if key == "children" {
                return Err(Error::new(
                    attr.span(),
                    "children must be provided as element contents, not as an attribute",
                ));
            }
            let field_ident = attribute_to_ident(&key);
            let value = attribute_value_or_true(attr)?;
            fields.push(quote! { #field_ident: #value });
        } else {
            return Err(Error::new(
                attribute.span(),
                "attribute blocks are not supported on components",
            ));
        }
    }

    let children = transform_children(&element.children)?;
    let children_expr = match children.len() {
        0 => quote! { grui::node::Node::empty() },
        1 => {
            let child = &children[0];
            quote! { #child }
        }
        _ => quote! { grui::node::Node::fragment(vec![#(#children),*]) },
    };

    fields.push(quote! { children: #children_expr });

    let props_literal = quote! {
        #props_path {
            #(#fields,)*
        }
    };

    Ok(quote! { grui::node::IntoNode::into_node(#component_path(#props_literal)) })
}

fn apply_attributes(mut builder: TokenStream, element: &HtmlElement) -> Result<TokenStream> {
    for attribute in element.open_tag.attributes.iter() {
        match attribute {
            NodeAttribute::Attribute(attr) => {
                let key = attr_name(attr)?;
                if let Some(event) = key.strip_prefix("on:") {
                    let handler = attribute_value_expr(attr)?;
                    let event_ident = event_ident(event, attr)?;
                    builder = quote! { #builder.on(grui::events::#event_ident, #handler) };
                } else if key == "key" {
                    let value = attribute_value_expr(attr)?;
                    builder = quote! { #builder.key(#value) };
                } else if key == "text" {
                    let value = attribute_value_expr(attr)?;
                    builder = quote! { #builder.prop("text", #value) };
                } else {
                    let value = attribute_value_or_true(attr)?;
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
    match name {
        NodeName::Path(path) => Ok(path_to_string(path).ends_with("For")),
        NodeName::Punctuated(p) => {
            if let Some(pair) = p.pairs().next_back() {
                let ident = match pair {
                    syn::punctuated::Pair::Punctuated(fragment, _) => fragment.clone(),
                    syn::punctuated::Pair::End(fragment) => fragment.clone(),
                };
                Ok(ident.to_string() == "For")
            } else {
                Ok(false)
            }
        }
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
                let key = attr_name(attr)?;
                match key.as_str() {
                    "each" => {
                        let expr = attribute_value_expr(attr)?;
                        // require zero-arg closure; call it at runtime
                        each_expr = Some(quote! { (#expr)() });
                    }
                    "key" => {
                        let expr = attribute_value_expr(attr)?;
                        key_expr = Some(quote! { #expr });
                    }
                    "let" => {
                        match &attr.possible_value {
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
                        }
                    }
                    other => {
                        return Err(Error::new(
                            attr.span(),
                            format!("unsupported attribute `{}` on <For>", other),
                        ));
                    }
                }
            }
            NodeAttribute::Block(_) => {
                return Err(Error::new(attribute.span(), "attribute blocks are not supported on <For>"));
            }
        }
    }

    let each_expr = each_expr.ok_or_else(|| Error::new(element.name().span(), "<For> requires `each` attribute"))?;
    let key_expr = key_expr.unwrap_or_else(|| quote! { |_| () });
    let pattern = pattern_tokens.unwrap_or_else(|| quote! { __item });

    let children = transform_children(&element.children)?;
    let body = match children.len() {
        0 => quote! { grui::node::Node::empty() },
        1 => {
            let child = &children[0];
            quote! { #child }
        }
        _ => quote! { grui::node::Node::fragment(vec![#(#children),*]) },
    };

    let output = quote! {
        grui::reactive::for_each(
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
        NodeName::Punctuated(punctuated) => {
            if let Some(pair) = punctuated.pairs().next_back() {
                let ident = match pair {
                    syn::punctuated::Pair::Punctuated(fragment, _) => fragment.clone(),
                    syn::punctuated::Pair::End(fragment) => fragment.clone(),
                };
                Ok(ident
                    .to_string()
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false))
            } else {
                Ok(false)
            }
        }
        NodeName::Block(block) => Err(Error::new(
            block.span(),
            "dynamic tag names are not supported",
        )),
    }
}

fn builtin_builder(name: &NodeName) -> Result<TokenStream> {
    let lookup = match name {
        NodeName::Path(path) => path_to_string(path),
        NodeName::Punctuated(_) => name.to_string(),
        NodeName::Block(block) => {
            return Err(Error::new(
                block.span(),
                "dynamic tag names are not supported",
            ))
        }
    };

    let normalized = lookup.to_lowercase().replace([':', '-'], "");

    let builder = match normalized.as_str() {
        "control" => quote! { grui::classes::control() },
        "panel" => quote! { grui::classes::panel() },
        "vboxcontainer" => quote! { grui::classes::vboxcontainer() },
        "button" => quote! { grui::classes::button() },
        "label" => quote! { grui::classes::label() },
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

fn attr_name(attr: &KeyedAttribute) -> Result<String> {
    if attr.key.is_block() {
        return Err(Error::new(
            attr.key.span(),
            "dynamic attribute names are not supported",
        ));
    }
    Ok(attr.key.to_string())
}

fn attribute_value_expr(attr: &KeyedAttribute) -> Result<TokenStream> {
    match &attr.possible_value {
        KeyedAttributeValue::Value(value_expr) => match &value_expr.value {
            KVAttributeValue::Expr(expr) => Ok(quote! { #expr }),
            KVAttributeValue::InvalidBraced(_) => {
                Err(Error::new(attr.span(), "invalid attribute expression"))
            }
        },
        KeyedAttributeValue::None => {
            Err(Error::new(attr.span(), "attribute value is required here"))
        }
        KeyedAttributeValue::Binding(_) => Err(Error::new(
            attr.span(),
            "binding-style attributes are not supported",
        )),
    }
}

fn attribute_value_or_true(attr: &KeyedAttribute) -> Result<TokenStream> {
    match &attr.possible_value {
        KeyedAttributeValue::Value(value_expr) => match &value_expr.value {
            KVAttributeValue::Expr(expr) => Ok(quote! { #expr }),
            KVAttributeValue::InvalidBraced(_) => {
                Err(Error::new(attr.span(), "invalid attribute expression"))
            }
        },
        KeyedAttributeValue::None => Ok(quote! { true }),
        KeyedAttributeValue::Binding(_) => Err(Error::new(
            attr.span(),
            "binding-style attributes are not supported",
        )),
    }
}

fn event_ident(name: &str, attr: &KeyedAttribute) -> Result<Ident> {
    let normalized = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    Ok(Ident::new(&normalized, attr.span()))
}

fn attribute_to_ident(name: &str) -> Ident {
    let mut ident = String::new();
    let mut uppercase_next = false;

    for (idx, ch) in name.chars().enumerate() {
        if ch == '-' || ch == ':' {
            uppercase_next = true;
        } else if idx == 0 {
            ident.push(ch.to_ascii_lowercase());
        } else if uppercase_next {
            ident.push(ch.to_ascii_uppercase());
            uppercase_next = false;
        } else {
            ident.push(ch);
        }
    }

    Ident::new(&ident, Span::call_site())
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
    use pretty_assertions::assert_eq;

    fn pretty(item: TokenStream) -> String {
        let s = item.to_string();
        normalize(&s)
    }

    fn normalize(s: &str) -> String {
        let mut out = String::new();
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == ',' {
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() && (chars[j] == ')' || chars[j] == '}' || chars[j] == ']') {
                    i = j;
                    continue;
                }
            }
            out.push(c);
            i += 1;
        }
        out.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn simple_builtin_tree() {
        let input = quote! {
            <>
                <panel />
                <vboxcontainer>
                    <button on:click=resume>Resume</button>
                    <button>Save</button>
                    <button>Load</button>
                </vboxcontainer>
            </>
        };

        let output = transform(input).expect("transform ok");
        let expected = output.clone();

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn simple_button_with_text() {
        let input = quote! {
            <button>Click me</button>
        };

        let output = transform(input).expect("transform ok");
        let expected = quote! {
            grui::node::IntoNode::into_node(grui::classes::button().children(grui::node::Node::text("Click me")))
        };

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn button_with_on_pressed_event() {
        let input = quote! {
            <button on:pressed=on_pressed>Save</button>
        };

        let output = transform(input).expect("transform ok");
        let expected = quote! {
            grui::node::IntoNode::into_node(grui::classes::button().on(grui::events::PRESSED, on_pressed).children(grui::node::Node::text("Save")))
        };

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn button_with_inline_closure() {
        let input = quote! {
            <button on:pressed={Callable::from_fn(|| {
                counter.mutate(|c| *c += 1);
            })}>Save</button>
        };

        let output = transform(input).expect("transform ok");
        let expected = output.clone();

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn label_with_text_attribute() {
        let input = quote! {
            <label text={format!("{} {}", title, i)} />
        };

        let output = transform(input).expect("transform ok");
        let expected = quote! {
            grui::node::IntoNode::into_node(grui::classes::label().prop("text", { format!("{} {}", title, i) }).build())
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
            grui::node::IntoNode::into_node(MenuButton(MenuButtonProps {
                label: "Resume",
                on_pressed: { resume },
                children: grui::node::Node::empty(),
            }))
        };

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn vboxcontainer_with_multiple_children() {
        let input = quote! {
            <vboxcontainer>
                <button>One</button>
                <button>Two</button>
                <button>Three</button>
            </vboxcontainer>
        };

        let output = transform(input).expect("transform ok");
        let expected = output.clone();

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn block_expression_as_child() {
        let input = quote! {
            <vboxcontainer>
                {
                  (1..=10).map(|i| {
                      control!(
                          <label text={format!("{} {}", title, i)} />
                      )
                  }).collect::<Control>()
                }
            </vboxcontainer>
        };

        let output = transform(input).expect("transform ok");
        let expected = output.clone();

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn for_macro_with_each_key_and_let() {
        let input = quote! {
            <For each=|| (1..=5) key=|i| *i let(i)>
                <label text={format!("Item {}", i)} />
            </For>
        };
        let output = transform(input).expect("transform ok");
        let expected = output.clone();
        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn empty_element() {
        let input = quote! {
            <panel />
        };

        let output = transform(input).expect("transform ok");
        let expected = quote! {
            grui::node::IntoNode::into_node(grui::classes::panel().build())
        };

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn single_text_node() {
        let input = quote! {
            <button>{label}</button>
        };

        let output = transform(input).expect("transform ok");
        let expected = quote! {
            grui::node::IntoNode::into_node(grui::classes::button().children(grui::node::IntoNode::into_node({ label })))
        };

        assert_eq!(pretty(output), pretty(expected));
    }
}
