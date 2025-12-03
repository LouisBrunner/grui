pub(crate) mod class;
pub(crate) mod component;
pub(crate) mod control;

#[cfg(test)]
pub(crate) mod test_utils;

#[proc_macro_attribute]
pub fn class(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);
    let output = class::transform(attr, item).unwrap_or_else(|err| err.to_compile_error());
    proc_macro::TokenStream::from(output)
}

#[proc_macro_attribute]
pub fn component(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);
    let output = component::transform(attr, item).unwrap_or_else(|err| err.to_compile_error());
    proc_macro::TokenStream::from(output)
}

#[proc_macro]
pub fn control(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let output = control::transform(input).unwrap_or_else(|err| err.to_compile_error());
    proc_macro::TokenStream::from(output)
}
