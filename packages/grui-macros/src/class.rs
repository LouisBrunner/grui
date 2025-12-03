use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
    Error, Ident, ItemStruct, Result, Token,
};

#[derive(Debug)]
struct Args {
    base: Ident,
    root: Ident,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        use syn::Token;

        let mut base: Option<Ident> = None;
        let mut root: Option<Ident> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: Ident = input.parse()?;

            let key_str = key.to_string();
            if key_str == "base" {
                if base.is_some() {
                    return Err(Error::new(key.span(), "duplicate `base`"));
                }
                base = Some(value);
            } else if key_str == "root" {
                if root.is_some() {
                    return Err(Error::new(key.span(), "duplicate `root`"));
                }
                root = Some(value);
            } else {
                return Err(Error::new(
                    key.span(),
                    "unexpected key; expected `base` or `root`",
                ));
            }

            if input.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        let root = root.ok_or_else(|| Error::new(input.span(), "missing `root` argument"))?;
        let base = base.unwrap_or_else(|| Ident::new("Control", proc_macro2::Span::call_site()));

        Ok(Self { base, root })
    }
}

pub fn transform(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let args = parse2::<Args>(attr)?;
    let item = parse2::<ItemStruct>(item)?;

    let span = item.span();
    let vis = item.vis;
    let ident = item.ident;
    let root = args.root;
    let base = args.base;
    let base_interface = format_ident!("I{}", base);
    let root_props = format_ident!("{}Props", root);
    let need_upcast = base != "Control";
    let get_base = if need_upcast {
        quote! { self.base.to_gd().upcast() }
    } else {
        quote! { self.base.to_gd() }
    };

    let fields = match item.fields {
        syn::Fields::Named(fields_named) => Ok(fields_named.named),
        syn::Fields::Unnamed(_) => Err(Error::new(span, "unnamed fields are not supported")),
        syn::Fields::Unit => Ok(Punctuated::new()),
    }?;

    let fields_comp = fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            quote! { #ident: self.#ident.clone() }
        })
        .collect::<Punctuated<_, Token![,]>>();

    let gen = quote! {
      use godot::classes::#base_interface;

      #[derive(godot::register::GodotClass)]
      #[class(init, base=#base)]
      #vis struct #ident {
          grui_renderer: Option<grui::renderer::Renderer>,
          base: godot::obj::Base<godot::classes::#base>,
          #fields
      }

      #[godot_api]
      impl #base_interface for #ident {
          fn ready(&mut self) {
            let props = #root_props {
              #fields_comp
            };
            self.grui_renderer = Some(grui::renderer::Renderer::mount(#get_base, #root, props));
          }
      }
    };

    Ok(gen)
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
              grui_renderer: Option<grui::renderer::Renderer>,
              base: godot::obj::Base<godot::classes::Control>,
              #[init(val = 10)]
              #[export]
              field: i32,
              abc: f64,
          }

          #[godot_api]
          impl IControl for MyStruct {
              fn ready(&mut self) {
                let props = AppProps {
                  field: self.field.clone(),
                  abc: self.abc.clone(),
                };
                self.grui_renderer = Some(grui::renderer::Renderer::mount(self.base.to_gd(), App, props));
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
                grui_renderer: Option<grui::renderer::Renderer>,
                base: godot::obj::Base<godot::classes::Control>,
            }

            #[godot_api]
            impl IControl for Empty {
                fn ready(&mut self) {
                  let props = AppProps {  };
                  self.grui_renderer = Some(grui::renderer::Renderer::mount(self.base.to_gd(), App, props));
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
                grui_renderer: Option<grui::renderer::Renderer>,
                base: godot::obj::Base<godot::classes::Button>,
                a: String,
                b: usize,
            }

            #[godot_api]
            impl IButton for Foo {
                fn ready(&mut self) {
                  let props = MyCompProps {
                    a: self.a.clone(),
                    b: self.b.clone(),
                  };
                  self.grui_renderer = Some(grui::renderer::Renderer::mount(self.base.to_gd().upcast(), MyComp, props));
                }
            }
        };

        assert_eq!(pretty(output), pretty(expected));
    }
}
