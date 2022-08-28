mod packets;
mod replace;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Protocol, attributes(varint, case, count, from, fixed, stringuuid))]
pub fn protocol(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        syn::Data::Struct(strukt) => {
            protocol::struct_protocol(input.ident, input.attrs, input.generics, strukt)
        }
        syn::Data::Enum(enom) => {
            protocol::enum_protocol(input.attrs, input.ident, input.generics, enom)
        }
        syn::Data::Union(_) => panic!("Union structs not supported"),
    }
}

#[proc_macro]
pub fn packets(input: TokenStream) -> TokenStream {
    let x = parse_macro_input!(input as packets::PacketsInput);

    packets::packets(x)
}

#[proc_macro]
pub fn replace(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as replace::ReplaceInput);

    let mut output = proc_macro2::TokenStream::new();

    replace::match_group(input.rest.into_iter(), &mut output, &input.types);

    output
        .into_iter()
        .collect::<proc_macro2::TokenStream>()
        .into()
}

mod protocol;
