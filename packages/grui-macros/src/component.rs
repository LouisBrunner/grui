use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned,
    Attribute, Error, Result, Signature, Visibility,
};

struct LenientFn {
    attrs: Vec<Attribute>,
    vis: Visibility,
    sig: Signature,
    body: TokenStream,
}

impl Parse for LenientFn {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let vis: Visibility = input.parse()?;
        let sig: Signature = input.parse()?;

        let body: TokenStream = input.parse()?;

        Ok(Self {
            attrs,
            vis,
            sig,
            body,
        })
    }
}

impl ToTokens for LenientFn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            attrs,
            vis,
            sig,
            body,
        } = self;

        let output = quote! {
            #(#attrs)*
            #vis #sig #body
        };

        tokens.append_all(output)
    }
}

pub fn transform(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    if !args.is_empty() {
        return Err(Error::new(args.span(), "no arguments are supported"));
    }

    let original = parse2::<LenientFn>(item)?;

    let gen = quote! {
      #[allow(non_snake_case)]
      #original
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
        let args = r#""#.parse().expect("args to be parsable");

        let input = quote! {
          fn Button(name: String) -> impl IntoControl {
              control!(
                <button>{name}</button>
              )
          }
        };

        let expected = quote! {
          #[allow(non_snake_case)]
          fn Button(name: String) -> impl IntoControl {
              control!(
                <button>{name}</button>
              )
          }
        };

        let output = transform(args, input).expect("transform to succeed");

        assert_eq!(prettyprint(output), prettyprint(expected));
    }
}
