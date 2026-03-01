use convert_case::{Case, Casing};
use from_attr::FromAttr;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse2, spanned::Spanned, Error, Expr, FnArg, Ident, ImplGenerics, ItemFn, Pat, PatIdent,
    Result, Type, TypeGenerics, WhereClause,
};

pub fn transform(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    if !args.is_empty() {
        return Err(Error::new(args.span(), "no arguments are supported"));
    }

    let mut function = parse2::<ItemFn>(item)?;

    function.sig.ident = make_snake_case(&function.sig.ident);
    let (impl_generics, ty_generics, where_clause) = function.sig.generics.split_for_impl();
    let (props, props_ident, destructure) =
        transform_props(&function, &impl_generics, &ty_generics, where_clause)?;

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

    let attrs = function.attrs.clone();
    function.attrs.clear();

    let output = quote! {
        #props
        #(#attrs)*
        #[allow(non_snake_case)]
        #function
    };

    Ok(output)
}

fn transform_props(
    function: &ItemFn,
    impl_generics: &ImplGenerics,
    _ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
) -> Result<(TokenStream, Ident, Option<TokenStream>)> {
    let props_ident = format_ident!("{}Props", function.sig.ident);

    let prop_list = extract_props(&function.sig)?;
    let has_fields = !prop_list.is_empty();
    let destructure: Option<TokenStream> = if has_fields {
        let destructure_fields = prop_list
            .iter()
            .map(|prop| prop.ident.clone())
            .collect::<Vec<_>>();

        Some(quote! { #props_ident { #( #destructure_fields, )* } })
    } else {
        None
    };

    let prop_fields = prop_list
        .iter()
        .map(|prop| prop.get_field_type())
        .collect::<Vec<_>>();

    let vis = function.vis.clone();
    let props = quote! {
        #[derive(Debug, ::grui::internal::typed_builder::TypedBuilder)]
        #[builder(crate_module_path=::grui::internal::typed_builder)]
        #vis struct #props_ident #impl_generics #where_clause {
            #(#prop_fields,)*
        }
    };

    Ok((props, props_ident, destructure))
}

#[derive(Debug, Default, FromAttr)]
#[attribute(idents = [prop])]
struct PropAttributes {
    optional: bool,
    into: bool,
    default: Option<Expr>,
}

#[derive(Debug)]
struct Prop {
    ident: Ident,
    ty: Type,
    attrs: PropAttributes,
}

impl Prop {
    fn get_field_type(&self) -> TokenStream {
        let Prop { ident, ty, .. } = &self;
        let field_type = if self.attrs.optional {
            quote! { Option<#ty> }
        } else {
            quote! { #ty }
        };
        let mut builder_opts = vec![];
        if self.attrs.into {
            builder_opts.push(quote! { #[builder(setter(into))] });
        }
        if let Some(default) = &self.attrs.default {
            builder_opts.push(quote! { #[builder(default = #default)] });
        }
        if self.attrs.optional {
            if self.attrs.default.is_none() {
                builder_opts.push(quote! { #[builder(default)] });
            }
            builder_opts.push(quote! { #[builder(setter(strip_option))] });
        }
        quote! { #(#builder_opts)* pub #ident: #field_type }
    }
}

fn extract_props(sig: &syn::Signature) -> Result<Vec<Prop>> {
    let mut props = Vec::new();

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
                props.push(Prop {
                    ident,
                    ty,
                    attrs: PropAttributes::from_attributes(&pat_ty.attrs)
                        .map_err(|err| err.value)?
                        .map(|av| av.value)
                        .unwrap_or_default(),
                });
            }
            FnArg::Receiver(_) => {
                return Err(Error::new(
                    input.span(),
                    "component functions cannot take self",
                ));
            }
        }
    }

    Ok(props)
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
            #[derive(Debug, ::grui::internal::typed_builder::TypedBuilder)]
            #[builder(crate_module_path = ::grui::internal::typed_builder)]
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
            #[derive(Debug, ::grui::internal::typed_builder::TypedBuilder)]
            #[builder(crate_module_path = ::grui::internal::typed_builder)]
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
            #[derive(Debug, ::grui::internal::typed_builder::TypedBuilder)]
            #[builder(crate_module_path = ::grui::internal::typed_builder)]
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
    pub fn component_with_generics() {
        let args = r#""#.parse().expect("args to be parsable");

        let input = quote! {
            fn SimpleButton<S>(label: S) -> impl IntoControl where S: Into<String> {
                control! {<button text=label />}
            }
        };

        let expected = quote! {
            #[derive(Debug, ::grui::internal::typed_builder::TypedBuilder)]
            #[builder(crate_module_path = ::grui::internal::typed_builder)]
            struct SimpleButtonProps<S> where S: Into<String> {
              pub label: S,
            }

            #[allow(non_snake_case)]
            fn SimpleButton<S>(props: SimpleButtonProps<S>) -> impl IntoControl where S: Into<String> {
                let SimpleButtonProps { label } = props;
                control! {<button text=label />}
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
            #[derive(Debug, ::grui::internal::typed_builder::TypedBuilder)]
            #[builder(crate_module_path = ::grui::internal::typed_builder)]
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
