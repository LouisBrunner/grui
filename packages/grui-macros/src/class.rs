use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
    Error, Ident, ItemStruct, Result, Token,
};

#[derive(Debug)]
struct Args {
    component: Ident,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Err(Error::new(input.span(), "expected component type"));
        } else {
            let arg_type = input.parse()?;
            if !input.is_empty() {
                return Err(Error::new(input.span(), "unexpected token"));
            }
            Ok(Self {
                component: arg_type,
            })
        }
    }
}

pub fn transform(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let args = parse2::<Args>(attr)?;
    let item = parse2::<ItemStruct>(item)?;

    let span = item.span();
    let vis = item.vis;
    let ident = item.ident;
    let comp = args.component;

    let fields = match item.fields {
        syn::Fields::Named(fields_named) => Ok(fields_named.named),
        syn::Fields::Unnamed(_) => Err(Error::new(span, "unnamed fields are not supported")),
        syn::Fields::Unit => Ok(Punctuated::new()),
    }?;
    let fields_comp = fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            quote! {
                #ident: self.#ident.clone()
            }
        })
        .collect::<Punctuated<_, Token![,]>>();

    let gen = quote! {
      use godot::classes::IControl;

      #[derive(godot::register::GodotClass)]
      #[class(init, base=Control)]
      #vis struct #ident {
          grui_renderer: Option<grui::renderer::Renderer>,
          base: godot::obj::Base<godot::classes::Control>,
          #fields
      }

      #[godot_api]
      impl IControl for #ident {
          fn ready(&mut self) {
            // TODO: watch for changes?
            self.grui_renderer = Some(grui::renderer::Renderer::new(self.base.to_gd(), #comp {
              #fields_comp
            }.render()))
          }

          fn process(&mut self, delta: f64) {
              if let Some(renderer) = &mut self.grui_renderer {
                renderer.process(delta);
              }
          }
      }
    };
    Ok(gen.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn prettyprint(item: TokenStream) -> String {
        item.to_string()
    }

    #[test]
    pub fn simple() {
        let args = r#"App"#.parse().expect("args to be parsable");

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
                  self.grui_renderer = Some(grui::renderer::Renderer::new(self.base.to_gd(), App {
                    field: self.field.clone(),
                    abc: self.abc.clone()
                  }.render()))
                }

                fn process(&mut self, delta: f64) {
                    if let Some(renderer) = &mut self.grui_renderer {
                      renderer.process(delta);
                    }
                }
            }
        };

        let output = transform(args, input).expect("transform to succeed");

        assert_eq!(prettyprint(output), prettyprint(expected));
    }
}
