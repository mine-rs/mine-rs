mod packets;
mod replace;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Protocol, attributes(varint, case, from, fixed, stringuuid))]
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

    // let mut packets = vec![];

    // let input: proc_macro2::TokenStream = input.into();
    // let mut ii = input.into_iter();

    // for t in ii.by_ref() {
    //     use proc_macro2::TokenTree::*;
    //     match t {
    //         Ident(id) => packets.push(id.to_string()),
    //         Punct(p) if p.as_char() == ';' => break,
    //         _ => {
    //             return quote::quote!(compile_error!(
    //                 "invalid input! only accepts idents, terminated using `;`"
    //             ))
    //             .into()
    //         }
    //     }
    // }

    let mut output = proc_macro2::TokenStream::new();

    replace::match_group(input.rest.into_iter(), &mut output, &input.types);

    output
        .into_iter()
        .collect::<proc_macro2::TokenStream>()
        .into()
}

mod protocol;
