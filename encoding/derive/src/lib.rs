use syn::{parse_macro_input, DeriveInput};

#[macro_use]
extern crate quote;
extern crate proc_macro;

#[proc_macro_derive(
    Encoding,
    attributes(varint, case, counted, from, fixed, stringuuid, bitfield, bits, bool)
)]
pub fn encoding(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        attrs,
        ident,
        generics,
        data,
        ..
    } = parse_macro_input!(input as DeriveInput);

    match data {
        syn::Data::Struct(strukt) => derive_struct(ident, attrs, generics, strukt),
        syn::Data::Enum(enom) => derive_enum(ident, attrs, generics, enom),
        syn::Data::Union(_) => panic!("Union structs not supported"),
    }
    .into()
}

macro_rules! error {
    ($span:ident, $id:ident) => {
        syn::Error::new($span, $id)
            .to_compile_error()
    };
    ($span:ident, $lit:literal) => {
        syn::Error::new($span, $lit)
            .to_compile_error()
    };
    ($span:ident, $($t:tt),+) => {
        syn::Error::new($span, format!($($t),+))
            .to_compile_error()
    };
}

mod attribute;
mod r#enum;
mod fields;
mod generics;
use r#enum::derive_enum;
mod r#struct;
use r#struct::derive_struct;
