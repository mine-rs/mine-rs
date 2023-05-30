use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::{parse_quote, Generics};

use crate::{fields::FieldsCode, generics::prepare_generics};

pub(crate) fn struct_from_fieldscode(
    fieldscode: FieldsCode,
    generics: Generics,
    ident: Ident,
    crate_path: &syn::Path,
) -> TokenStream {
    let FieldsCode {
        parsing,
        destructuring,
        serialization,
    } = fieldscode;

    let mut res = quote!();

    let mut encode_generics = generics.clone();
    prepare_generics(
        &mut encode_generics,
        parse_quote!(#crate_path::Encode),
        None,
    );
    let (implgenerics, typegenerics, whereclause) = encode_generics.split_for_impl();
    quote! {
        impl #implgenerics #crate_path::Encode for #ident #typegenerics
        #whereclause
        {
            fn encode(&self, writer: &mut impl ::std::io::Write) -> #crate_path::encode::Result<()> {
                let Self #destructuring = self;
                #serialization
                Ok(())
            }
        }
    }
    .to_tokens(&mut res);

    let mut decode_generics = generics;
    prepare_generics(
        &mut decode_generics,
        parse_quote!(#crate_path::Decode<'dec>),
        Some(parse_quote!('dec)),
    );
    let (implgenerics, _, whereclause) = decode_generics.split_for_impl();
    quote! {
        impl #implgenerics #crate_path::Decode<'dec> for #ident #typegenerics
        #whereclause
        {
            fn decode(cursor: &mut ::std::io::Cursor<&'dec [::core::primitive::u8]>) -> #crate_path::decode::Result<Self> {
                #parsing
                Ok(Self #destructuring)
            }
        }
    }
    .to_tokens(&mut res);

    res
}
