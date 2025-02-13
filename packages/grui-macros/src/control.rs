use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use rstml::parse2;
use syn::{spanned::Spanned, Error, Result};

pub fn transform(input: TokenStream) -> Result<TokenStream> {
    let span = input.span().clone();
    let nodes = parse2(input)?;
    if nodes.len() != 1 {
        return Err(Error::new(span, "expected a single root element"));
    }
    let root = transform_node(&nodes[0]);
    let gen = quote! { #root };
    Ok(gen.into())
}

struct Node {}

impl ToTokens for Node {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let output = quote! {
            // #(#attrs)*
            // #vis #sig #body
        };

        tokens.append_all(output)
    }
}

fn transform_node(node: &rstml::node::Node) -> Node {
    Node {}
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
        let input = r#"
          <>
            <panel />
            <vboxcontainer>
              <button on:click=resume>Resume</button>
              <button>Save</button>
              <button>Load</button>
              <Button color="red">"Exit " {increment}</Button>
            </vboxcontainer>
          </>
        "#
        .parse()
        .expect("input to be parsable");

        let expected = quote! {
          grui::classes::control().children(
            grui::classes::panel().children(),
            grui::classes::vboxcontainer().children(
              grui::classes::button().on(grui::events::click, resume).children("Resume"),
              grui::classes::button().children("Save"),
              grui::classes::button().children("Load"),
              Button(ButtonProps {
                color: "red",
                children: ("Exit " , increment),
              }),
            ),
          )
        };

        let output = transform(input).expect("transform to succeed");

        assert_eq!(prettyprint(output), prettyprint(expected));
    }
}
