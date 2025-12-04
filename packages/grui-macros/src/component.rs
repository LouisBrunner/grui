use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, spanned::Spanned, Error, FnArg, Ident, ItemFn, Pat, PatIdent, Result, Type};

pub fn transform(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    if !args.is_empty() {
        return Err(Error::new(args.span(), "no arguments are supported"));
    }

    let mut function = parse2::<ItemFn>(item)?;

    function.sig.ident = make_snake_case(&function.sig.ident);
    let props_ident = format_ident!("{}Props", function.sig.ident);

    let (impl_generics, ty_generics, where_clause) = function.sig.generics.split_for_impl();

    let (field_idents, field_types) = extract_fields(&function.sig)?;
    let has_fields = !field_idents.is_empty();
    let destructure: Option<TokenStream> = if has_fields {
        Some(quote! { #props_ident { #( #field_idents, )* } })
    } else {
        None
    };
    function.sig.inputs.clear();
    if let Some(destructure_ts) = &destructure {
        function
            .sig
            .inputs
            .push(syn::parse_quote! { props: #props_ident #ty_generics });
        function
            .block
            .stmts
            .insert(0, syn::parse_quote! { let #destructure_ts = props; });
    } else {
        function
            .sig
            .inputs
            .push(syn::parse_quote! { _: #props_ident #ty_generics });
    }

    let vis = function.vis.clone();
    let attrs = function.attrs.clone();
    function.attrs.clear();

    let props = quote! {
        #[derive(Clone, Debug)]
        #vis struct #props_ident #impl_generics #where_clause {
            #(pub #field_idents: #field_types,)*
        }
    };

    let output = quote! {
        #props
        #(#attrs)*
        #[allow(non_snake_case)]
        #function
    };

    Ok(output)
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
            FnArg::Receiver(_) => {
                return Err(Error::new(
                    input.span(),
                    "component functions cannot take self",
                ));
            }
        }
    }

    Ok((idents, types))
}

fn make_snake_case(name: &Ident) -> Ident {
    let name_str = name.to_string();
    if !name_str.is_case(Case::Snake) {
        name.clone()
    } else {
        Ident::new(&name_str.to_case(Case::Pascal), name.span())
    }
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
            pub fn Button(label: String, disabled: bool) -> impl IntoControl {
                control! {
                  <button disabled=disabled text=label />
                }
            }
        };

        let expected = quote! {
            #[derive(Clone, Debug)]
            pub struct ButtonProps {
                pub label: String,
                pub disabled: bool,
            }

            #[allow(non_snake_case)]
            pub fn Button(props: ButtonProps) -> impl IntoControl {
                let ButtonProps { label, disabled } = props;
                control! {
                    <button disabled=disabled text=label />
                }
            }
        };

        let output = transform(args, input).expect("transform to succeed");

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    pub fn readme_menu_button_example() {
        let args = r#""#.parse().expect("args to be parsable");

        let input = quote! {
            fn MenuButton(label: String, on_pressed: Callable) -> impl IntoControl {
                control! {
                    <button on:pressed=on_pressed text=label />
                }
            }
        };

        let expected = quote! {
            #[derive(Clone, Debug)]
            struct MenuButtonProps {
                pub label: String,
                pub on_pressed: Callable,
            }

            #[allow(non_snake_case)]
            fn MenuButton(props: MenuButtonProps) -> impl IntoControl {
                let MenuButtonProps { label, on_pressed } = props;
                control! {
                    <button on:pressed=on_pressed text=label />
                }
            }
        };

        let output = transform(args, input).expect("transform to succeed");

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    pub fn no_params_component() {
        let args = r#""#.parse().expect("args to be parsable");

        let input = quote! {
            fn SimpleButton() -> impl IntoControl {
                control! {<button text="Click" />}
            }
        };

        let expected = quote! {
            #[derive(Clone, Debug)]
            struct SimpleButtonProps { }

            #[allow(non_snake_case)]
            fn SimpleButton(_: SimpleButtonProps) -> impl IntoControl {
                control! {<button text="Click" />}
            }
        };

        let output = transform(args, input).expect("transform to succeed");

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn captures_var_names() {
        let args = r#""#.parse().unwrap();
        let input = quote! { fn foo(bar: i32, children: String) -> impl IntoControl { control! {<label max_lines_visible=bar text=children />} } };
        let output = transform(args, input).expect("ok");
        let expected = quote! {
            #[derive(Clone, Debug)]
            struct FooProps {
                pub bar: i32,
                pub children: String,
            }

            #[allow(non_snake_case)]
            fn Foo(props: FooProps) -> impl IntoControl {
                let FooProps { bar, children } = props;
                control! {<label max_lines_visible=bar text=children />}
            }
        };
        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn rejects_receiver() {
        let args = r#""#.parse().unwrap();
        let input = quote! { fn method(&self) -> impl IntoControl { control! {<label/>} } };
        let err = transform(args, input).unwrap_err();
        let msg = err.to_string();
        let expected = "component functions cannot take self";
        assert_eq!(msg, expected);
    }
}
