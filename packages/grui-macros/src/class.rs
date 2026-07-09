use std::str::FromStr;

use from_attr::FromAttr;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use strum::EnumString;
use syn::{
    parse2, punctuated::Punctuated, spanned::Spanned, Error, Field, Ident, ItemStruct, Result, Type,
};

#[derive(Debug, Default, FromAttr)]
#[attribute(idents = [prop])]
struct Args {
    base: Option<Ident>,
    root: Option<Ident>,
    forward: Option<Ident>,
    no_impl: bool,
}

pub fn transform(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let attr_span = attr.span();
    let args = Args::from_tokens(attr)?;
    let item = parse2::<ItemStruct>(item)?;

    let base = args
        .base
        .unwrap_or_else(|| Ident::new("Control", proc_macro2::Span::call_site()));
    let base_interface = format_ident!("I{}", base);

    let (fields, mut props) = extract_fields(&item)?;
    if let Some(forward) = args.forward {
        props.push(quote! { #forward={gd.clone().upcast()} });
    }

    let ident = item.ident;
    let godot_impl = if args.no_impl {
        quote! {}
    } else {
        quote! {
            #[godot_api]
            impl #base_interface for #ident {
                fn ready(&mut self) {
                    self.mount_controls();
                }
            }
        }
    };

    let root = args
        .root
        .ok_or_else(|| Error::new(attr_span, "missing `root` argument"))?;
    let vis = item.vis;
    let generated = quote! {
      use godot::classes::#base_interface;

      #[derive(godot::register::GodotClass)]
      #[class(init, base=#base)]
      #vis struct #ident {
          grui_renderer: Option<::godot_grui::prelude::Renderer>,
          base: godot::obj::Base<godot::classes::#base>,
          #(#fields,)*
      }

      impl #ident {
          fn mount_controls(&mut self) {
              let gd = self.to_gd();
              self.grui_renderer = Some(::godot_grui::prelude::Renderer::mount(&gd, || ::godot_grui::prelude::control!{ <#root #(#props)* /> }));
          }
      }

      #godot_impl
    };

    Ok(generated)
}

#[derive(Debug, EnumString)]
#[strum(serialize_all = "kebab-case")]
enum SignalForward {
    Read,
    Write,
}

#[derive(Debug, Default, FromAttr)]
#[attribute(idents = [prop])]
struct PropAttributes {
    signal: Option<String>,
}

struct FieldProp {
    field: Field,
    ident: Ident,
    attrs: PropAttributes,
}

impl FieldProp {
    fn get_prop(&self) -> Result<TokenStream> {
        let FieldProp { ident, .. } = self;
        let value = if let Some(sig) = &self.attrs.signal {
            let rw = SignalForward::from_str(sig).map_err(|err| {
                Error::new(
                    self.field.span(),
                    format!("signal must be 'read' or 'write': {}", err),
                )
            })?;
            match rw {
                SignalForward::Read => quote! { self.#ident.0.clone() },
                SignalForward::Write => quote! { self.#ident.1.clone() },
            }
        } else {
            quote! { self.#ident.clone() }
        };
        Ok(quote! { #ident=#value })
    }

    fn get_field_type(&self) -> TokenStream {
        let mut field = self.field.clone();
        let mut attr = None;
        if self.attrs.signal.is_some() {
            let ty = field.ty.clone();
            let def_value = quote! { ::std::default::Default::default() };
            attr = Some(quote! { #[init(val = ::godot_grui::prelude::signal(#def_value))] });
            field.ty = Type::Verbatim(quote! {
              (::godot_grui::prelude::ReadSignal<#ty>, ::godot_grui::prelude::WriteSignal<#ty>)
            });
        }
        quote! { #attr #field }
    }
}

fn extract_fields(item: &ItemStruct) -> Result<(Vec<TokenStream>, Vec<TokenStream>)> {
    let mut fields = match &item.fields {
        syn::Fields::Named(fields_named) => Ok(fields_named.named.clone()),
        syn::Fields::Unnamed(_) => Err(Error::new(item.span(), "unnamed fields are not supported")),
        syn::Fields::Unit => Ok(Punctuated::new()),
    }?;

    let props = fields
        .iter_mut()
        .map(|field| {
            Ok(FieldProp {
                attrs: PropAttributes::remove_attributes(&mut field.attrs)
                    .map_err(|err| err.value)?
                    .map(|av| av.value)
                    .unwrap_or_default(),
                field: field.clone(),
                ident: field
                    .ident
                    .as_ref()
                    .ok_or_else(|| Error::new(field.span(), "field must have an identifier"))?
                    .clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let prop_fields = props
        .iter()
        .map(|prop| prop.get_field_type())
        .collect::<Vec<_>>();

    let prop_list = props
        .iter()
        .map(|prop| prop.get_prop())
        .collect::<Result<Vec<_>>>()?;

    Ok((prop_fields, prop_list))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::pretty;
    use pretty_assertions::assert_eq;

    #[test]
    fn with_props() {
        let args = r#"root=App"#.parse().expect("args to be parsable");

        let input = quote! {
          pub struct MyStruct {
              #[init(val = 10)]
              #[export]
              field: i32,
              abc: f64,
          }
        };

        let expected = quote! {
          use godot::classes::IControl;

          #[derive(godot::register::GodotClass)]
          #[class(init, base=Control)]
          pub struct MyStruct {
              grui_renderer: Option<::godot_grui::prelude::Renderer>,
              base: godot::obj::Base<godot::classes::Control>,
              #[init(val = 10)]
              #[export]
              field: i32,
              abc: f64,
          }

          impl MyStruct {
              fn mount_controls(&mut self) {
                let gd = self.to_gd();
                self.grui_renderer = Some(::godot_grui::prelude::Renderer::mount(&gd, || ::godot_grui::prelude::control! { <App field=self.field.clone() abc=self.abc.clone() /> }));
              }
          }

          #[godot_api]
          impl IControl for MyStruct {
              fn ready(&mut self) {
                self.mount_controls();
              }
          }
        };

        let output = transform(args, input).expect("transform to succeed");
        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn without_props() {
        let args = r#"root=App"#.parse().unwrap();
        let input = quote! { struct Empty; };
        let output = transform(args, input).expect("transform ok");

        let expected = quote! {
            use godot::classes::IControl;

            #[derive(godot::register::GodotClass)]
            #[class(init, base=Control)]
            struct Empty {
                grui_renderer: Option<::godot_grui::prelude::Renderer>,
                base: godot::obj::Base<godot::classes::Control>,
            }

            impl Empty {
                fn mount_controls(&mut self) {
                  let gd = self.to_gd();
                  self.grui_renderer = Some(::godot_grui::prelude::Renderer::mount(&gd, || ::godot_grui::prelude::control! { <App /> }));
                }
            }

            #[godot_api]
            impl IControl for Empty {
                fn ready(&mut self) {
                  self.mount_controls();
                }
            }
        };

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn with_custom_base() {
        let args = r#"root=MyComp,base=Button"#.parse().unwrap();
        let input = quote! { struct Foo { a: String, b: usize } };
        let output = transform(args, input).expect("transform ok");

        let expected = quote! {
            use godot::classes::IButton;

            #[derive(godot::register::GodotClass)]
            #[class(init, base=Button)]
            struct Foo {
                grui_renderer: Option<::godot_grui::prelude::Renderer>,
                base: godot::obj::Base<godot::classes::Button>,
                a: String,
                b: usize,
            }

            impl Foo {
                fn mount_controls(&mut self) {
                  let gd = self.to_gd();
                  self.grui_renderer = Some(::godot_grui::prelude::Renderer::mount(&gd, || ::godot_grui::prelude::control! { <MyComp a=self.a.clone() b=self.b.clone() /> }));
                }
            }

            #[godot_api]
            impl IButton for Foo {
                fn ready(&mut self) {
                  self.mount_controls();
                }
            }
        };

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn with_props_attributes() {
        let args = r#"root=MyComp,forward=root"#.parse().unwrap();
        let input = quote! {
          struct Foo {
            a: String,
            #[prop(signal="read")]
            b: usize
          }
        };
        let output = transform(args, input).expect("transform ok");

        let expected = quote! {
            use godot::classes::IControl;

            #[derive(godot::register::GodotClass)]
            #[class(init, base=Control)]
            struct Foo {
                grui_renderer: Option<::godot_grui::prelude::Renderer>,
                base: godot::obj::Base<godot::classes::Control>,
                a: String,
                #[init(val=::godot_grui::prelude::signal(::std::default::Default::default()))]
                b: (::godot_grui::prelude::ReadSignal<usize>, ::godot_grui::prelude::WriteSignal<usize>),
            }

            impl Foo {
                fn mount_controls(&mut self) {
                  let gd = self.to_gd();
                  self.grui_renderer = Some(::godot_grui::prelude::Renderer::mount(&gd, || ::godot_grui::prelude::control! { <MyComp a=self.a.clone() b=self.b.0.clone() root={gd.clone().upcast()} /> }));
                }
            }

            #[godot_api]
            impl IControl for Foo {
                fn ready(&mut self) {
                  self.mount_controls();
                }
            }
        };

        assert_eq!(pretty(output), pretty(expected));
    }

    #[test]
    fn with_other_attrs() {
        let args = r#"root=MyComp,no_impl"#.parse().unwrap();
        let input = quote! {
          struct Foo {
            a: String,
            #[prop(signal="write")]
            b: usize
          }
        };
        let output = transform(args, input).expect("transform ok");

        let expected = quote! {
            use godot::classes::IControl;

            #[derive(godot::register::GodotClass)]
            #[class(init, base=Control)]
            struct Foo {
                grui_renderer: Option<::godot_grui::prelude::Renderer>,
                base: godot::obj::Base<godot::classes::Control>,
                a: String,
                #[init(val=::godot_grui::prelude::signal(::std::default::Default::default()))]
                b: (::godot_grui::prelude::ReadSignal<usize>, ::godot_grui::prelude::WriteSignal<usize>),
            }

            impl Foo {
                fn mount_controls(&mut self) {
                  let gd = self.to_gd();
                  self.grui_renderer = Some(::godot_grui::prelude::Renderer::mount(&gd, || ::godot_grui::prelude::control! { <MyComp a=self.a.clone() b=self.b.1.clone() /> }));
                }
            }
        };

        assert_eq!(pretty(output), pretty(expected));
    }
}
