mod attribute;
mod protocol;

#[proc_macro_derive(Protocol, attributes(varint, case, from))]
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

use proc_macro::TokenStream;
use proc_macro2::Ident;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{braced, parse_macro_input, token, LitInt, Token};
use syn::{DeriveInput, Expr};

struct Binding {
    expr: Expr,
    arrow: token::FatArrow,
    protocol_version: LitInt,
}
impl Parse for Binding {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            expr: input.parse()?,
            arrow: input.parse()?,
            protocol_version: input.parse()?,
        })
    }
}
struct PacketIdBindings {
    ident: Ident,
    arrow: token::FatArrow,
    bracket_token: token::Brace,
    bindings: Punctuated<Binding, Token![,]>,
}
impl Parse for PacketIdBindings {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            ident: input.parse()?,
            arrow: input.parse()?,
            bracket_token: braced!(content in input),
            bindings: content.parse_terminated(Binding::parse)?,
        })
    }
}
struct PacketIdInput {
    inner: Punctuated<PacketIdBindings, Token![,]>,
}
impl Parse for PacketIdInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            inner: input.parse_terminated(PacketIdBindings::parse)?,
        })
    }
}

#[proc_macro]
pub fn packet_id(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as PacketIdInput);

    // let mut unbounded: Vec<(RangeFrom<i32>,)> = vec![];
    // let mut bounded: Vec<(Range<i32>,)> = vec![];

    for inner in input.inner {
        for binding in inner.bindings {
            match binding.expr {
                Expr::Lit(lit) => match lit.lit {
                    syn::Lit::Int(_) => todo!(),
                    _ => todo!(),
                },
                Expr::Range(_) => todo!(),
                _ => todo!(),
            }
        }
    }

    todo!()
}
