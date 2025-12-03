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
            Err(Error::new(input.span(), "expected component type"))
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
            quote! { #ident: self.#ident.clone() }
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
            let component = #comp {
              #fields_comp
            };
            self.grui_renderer = Some(grui::renderer::Renderer::from_component(self.base.to_gd(), component));
          }

          fn process(&mut self, delta: f64) {
              if let Some(renderer) = &mut self.grui_renderer {
                renderer.process(delta);
              }
          }
      }
    };

    Ok(gen)
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
                // look ahead for spaces then a closing bracket
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() && (chars[j] == ')' || chars[j] == '}' || chars[j] == ']') {
                    // skip the comma and following spaces
                    i += 1;
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    continue;
                }
            }
            out.push(c);
            i += 1;
        }
        // collapse consecutive whitespace to single spaces
        let mut collapsed = String::new();
        let mut prev_space = false;
        for ch in out.chars() {
            if ch.is_whitespace() {
                if !prev_space {
                    collapsed.push(' ');
                    prev_space = true;
                }
            } else {
                collapsed.push(ch);
                prev_space = false;
            }
        }
        collapsed.trim().to_string()
    }

    #[test]
    fn simple() {
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
                let component = App {
                  field: self.field.clone(),
                  abc: self.abc.clone(),
                };
                self.grui_renderer = Some(grui::renderer::Renderer::from_component(self.base.to_gd(), component));
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

    #[test]
    fn unit_struct_generates_base_fields() {
        let args = r#"App"#.parse().unwrap();
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
                  let component = App {  };
                  self.grui_renderer = Some(grui::renderer::Renderer::from_component(self.base.to_gd(), component));
                }

                fn process(&mut self, delta: f64) {
                    if let Some(renderer) = &mut self.grui_renderer {
                      renderer.process(delta);
                    }
                }
            }
        };

        assert_eq!(prettyprint(output), prettyprint(expected));
    }

    #[test]
    fn named_fields_cloned_into_props() {
        let args = r#"MyComp"#.parse().unwrap();
        let input = quote! { struct Foo { a: String, b: usize } };
        let output = transform(args, input).expect("transform ok");

        let expected = quote! {
            use godot::classes::IControl;

            #[derive(godot::register::GodotClass)]
            #[class(init, base=Control)]
            struct Foo {
                grui_renderer: Option<grui::renderer::Renderer>,
                base: godot::obj::Base<godot::classes::Control>,
                a: String,
                b: usize,
            }

            #[godot_api]
            impl IControl for Foo {
                fn ready(&mut self) {
                  let component = MyComp {
                    a: self.a.clone(),
                    b: self.b.clone(),
                  };
                  self.grui_renderer = Some(grui::renderer::Renderer::from_component(self.base.to_gd(), component));
                }

                fn process(&mut self, delta: f64) {
                    if let Some(renderer) = &mut self.grui_renderer {
                      renderer.process(delta);
                    }
                }
            }
        };

        assert_eq!(prettyprint(output), prettyprint(expected));
    }
}
