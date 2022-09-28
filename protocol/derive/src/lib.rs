#[macro_use]
extern crate quote;
extern crate proc_macro;

mod parsing_tree;
mod replace;

#[proc_macro]
pub fn parsing_tree(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let x = syn::parse_macro_input!(input as parsing_tree::ParsingTreeInput);

    parsing_tree::parsing_tree(x)
}

#[proc_macro]
pub fn replace(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as replace::ReplaceInput);

    let mut output = proc_macro2::TokenStream::new();

    replace::match_group(input.rest.into_iter(), &mut output, &input.types);

    output.into()
}
