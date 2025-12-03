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
    let has_fields = !field_idents.is_empty();
    let destructure: Option<TokenStream> = if has_fields {
        Some(quote! { #props_ident { #( #field_idents, )* } })
    } else {
        None
    };

    function.sig.inputs.clear();
    function
        .sig
        .inputs
        .push(syn::parse_quote! { props: #props_ident });

    if let Some(destructure_ts) = &destructure {
        function
            .block
            .stmts
            .insert(0, syn::parse_quote! { let #destructure_ts = props; });
    }

    let vis = function.vis.clone();
    let attrs = function.attrs.clone();
    function.attrs.clear();

    function.sig.output = syn::parse_quote! { -> impl grui::node::IntoControl };

    let props = quote! {
        #[derive(Clone, Debug)]
        #vis struct #props_ident {
            #(pub #field_idents: #field_types,)*
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
    use crate::test_utils::pretty;
    use pretty_assertions::assert_eq;

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
            }

            #[allow(non_snake_case)]
            pub fn Button(props: ButtonProps) -> impl grui::node::IntoControl {
                let ButtonProps { label, disabled } = props;
                if disabled {
                    return Node::empty();
                }
                control!(
                    <button>{label}</button>
                )
            }
        };

        let output = transform(args, input).expect("transform to succeed");

        assert_eq!(pretty(output), pretty(expected));
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
            }

            #[allow(non_snake_case)]
            fn MenuButton(props: MenuButtonProps) -> impl grui::node::IntoControl {
                let MenuButtonProps { label, on_pressed } = props;
                control!(
                    <button on:pressed=on_pressed>{label}</button>
                )
            }
        };

        let output = transform(args, input).expect("transform to succeed");

        assert_eq!(pretty(output), pretty(expected));
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
            struct SimpleButtonProps { }

            #[allow(non_snake_case)]
            fn SimpleButton(props: SimpleButtonProps) -> impl grui::node::IntoControl {
                control!(<button>Click</button>)
            }
        };

        let output = transform(args, input).expect("transform to succeed");

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn captures_var_names() {
        let args = r#""#.parse().unwrap();
        let input = quote! { fn Foo(bar: i32, children: String) -> impl grui::node::IntoControl { control!(<label max_lines_visible=bar>{children}</label>) } };
        let output = transform(args, input).expect("ok");
        let expected = quote! {
            #[derive(Clone, Debug)]
            struct FooProps {
                pub bar: i32,
                pub children: String,
            }

            #[allow(non_snake_case)]
            fn Foo(props: FooProps) -> impl grui::node::IntoControl {
                let FooProps { bar, children } = props;
                control!(<label max_lines_visible=bar>{children}</label>)
            }
        };
        assert_eq!(pretty(output), pretty(expected));
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
}
