use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, spanned::Spanned, Error, FnArg, Ident, ItemFn, Pat, PatIdent, Result, Type};

pub fn transform(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    if !args.is_empty() {
        return Err(Error::new(args.span(), "no arguments are supported"));
    }

    let mut function = parse2::<ItemFn>(item)?;
    validate_signature(&function)?;

    let props_ident = format_ident!("{}Props", function.sig.ident);
    let (field_idents, field_types) = extract_fields(&function.sig)?;
    let destructure: TokenStream = quote! { #props_ident { #( #field_idents, )* children } };

    function.sig.inputs.clear();
    function
        .sig
        .inputs
        .push(syn::parse_quote! { props: #props_ident });

    function
        .block
        .stmts
        .insert(0, syn::parse_quote! { let #destructure = props; });

    let vis = function.vis.clone();
    let attrs = function.attrs.clone();
    function.attrs.clear();

    function.sig.output = syn::parse_quote! { -> impl grui::node::IntoControl };

    let props = quote! {
        #[derive(Clone, Debug)]
        #vis struct #props_ident {
            #(pub #field_idents: #field_types,)*
            pub children: grui::node::Children,
        }
    };

    let output = quote! {
        #props
        #[allow(non_snake_case)]
        #(#attrs)*
        #function
    };

    Ok(output)
}

fn validate_signature(function: &ItemFn) -> Result<()> {
    if matches!(function.sig.inputs.first(), Some(FnArg::Receiver(_))) {
        return Err(Error::new(
            function.sig.inputs.span(),
            "component functions cannot take self",
        ));
    }
    Ok(())
}

fn extract_fields(sig: &syn::Signature) -> Result<(Vec<Ident>, Vec<Type>)> {
    let mut idents = Vec::new();
    let mut types = Vec::new();

    for input in sig.inputs.iter() {
        match input {
            FnArg::Typed(pat_ty) => {
                let Pat::Ident(PatIdent { ident, .. }) = &*pat_ty.pat else {
                    return Err(Error::new(
                        pat_ty.pat.span(),
                        "component parameters must be simple identifiers",
                    ));
                };

                let ident = ident.clone();
                let ty = (*pat_ty.ty).clone();
                idents.push(ident);
                types.push(ty);
            }
            FnArg::Receiver(_) => {}
        }
    }

    Ok((idents, types))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn prettyprint(item: TokenStream) -> String {
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
        // collapse whitespace
        out.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    pub fn simple() {
        let args = r#""#.parse().expect("args to be parsable");

        let input = quote! {
            pub fn Button(label: String, disabled: bool) -> impl grui::node::IntoControl {
                if disabled {
                    return Node::empty();
                }
                control!(
                  <button>{label}</button>
                )
            }
        };

        let expected = quote! {
            #[derive(Clone, Debug)]
            pub struct ButtonProps {
                pub label: String,
                pub disabled: bool,
                pub children: grui::node::Children,
            }

            #[allow(non_snake_case)]
            pub fn Button(props: ButtonProps) -> impl grui::node::IntoControl {
                let ButtonProps { label, disabled, children } = props;
                if disabled {
                    return Node::empty();
                }
                control!(
                    <button>{label}</button>
                )
            }
        };

        let output = transform(args, input).expect("transform to succeed");

        assert_eq!(prettyprint(output), prettyprint(expected));
    }

    #[test]
    pub fn readme_menu_button_example() {
        let args = r#""#.parse().expect("args to be parsable");

        let input = quote! {
            fn MenuButton(label: String, on_pressed: Callable) -> Control {
                control!(
                    <button on:pressed=on_pressed>{label}</button>
                )
            }
        };

        let expected = quote! {
            #[derive(Clone, Debug)]
            struct MenuButtonProps {
                pub label: String,
                pub on_pressed: Callable,
                pub children: grui::node::Children,
            }

            #[allow(non_snake_case)]
            fn MenuButton(props: MenuButtonProps) -> impl grui::node::IntoControl {
                let MenuButtonProps { label, on_pressed, children } = props;
                control!(
                    <button on:pressed=on_pressed>{label}</button>
                )
            }
        };

        let output = transform(args, input).expect("transform to succeed");

        assert_eq!(prettyprint(output), prettyprint(expected));
    }

    #[test]
    pub fn no_params_component() {
        let args = r#""#.parse().expect("args to be parsable");

        let input = quote! {
            fn SimpleButton() -> Control {
                control!(<button>Click</button>)
            }
        };

        let expected = quote! {
            #[derive(Clone, Debug)]
            struct SimpleButtonProps {
                pub children: grui::node::Children,
            }

            #[allow(non_snake_case)]
            fn SimpleButton(props: SimpleButtonProps) -> impl grui::node::IntoControl {
                let SimpleButtonProps { children } = props;
                control!(<button>Click</button>)
            }
        };

        let output = transform(args, input).expect("transform to succeed");

        assert_eq!(prettyprint(output), prettyprint(expected));
    }

    #[test]
    fn captures_var_names() {
        let args = r#""#.parse().unwrap();
        let input = quote! { fn Foo(bar: i32) -> impl grui::node::IntoControl { control!(<label>{bar}</label>) } };
        let output = transform(args, input).expect("ok");
        let expected = quote! {
            #[derive(Clone, Debug)]
            struct FooProps {
                pub bar: i32,
                pub children: grui::node::Children,
            }

            #[allow(non_snake_case)]
            fn Foo(props: FooProps) -> impl grui::node::IntoControl {
                let FooProps { bar, children } = props;
                control!(<label>{bar}</label>)
            }
        };
        assert_eq!(prettyprint(output), prettyprint(expected));
    }

    #[test]
    fn rejects_receiver() {
        let args = r#""#.parse().unwrap();
        let input =
            quote! { fn method(&self) -> impl grui::node::IntoControl { control!(<label/> ) } };
        let err = transform(args, input).unwrap_err();
        let msg = err.to_string();
        let expected = "component functions cannot take self";
        assert_eq!(msg, expected);
    }

    #[test]
    fn debug_print_component_outputs() {
        // Prints outputs for a few cases to help update expected strings
        let args: TokenStream = r#""#.parse().unwrap();

        let cases = vec![
            quote! { pub fn Button(label: String, disabled: bool) -> impl grui::node::IntoControl { if disabled { return Node::empty(); } control!(<button>{label}</button>) } },
            quote! { fn MenuButton(label: String, on_pressed: Callable) -> Control { control!(<button on:pressed=on_pressed>{label}</button>) } },
            quote! { fn SimpleButton() -> Control { control!(<button>Click</button>) } },
        ];

        for (i, input) in cases.into_iter().enumerate() {
            let out = transform(args.clone(), input)
                .map(|t| t.to_string())
                .unwrap_or_else(|e| e.to_string());
            println!("CASE {}: {}", i, out);
        }
    }
}
