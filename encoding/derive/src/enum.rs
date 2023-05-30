use darling::util::Flag;
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::{parse_quote, Generics};

use crate::{
    fields::{self, FieldsCode},
    generics::prepare_generics,
    EncodingVariant,
};

pub(crate) fn enum_from_variants(
    variants: Vec<EncodingVariant>,
    generics: Generics,
    ident: Ident,
    varint: Flag,
    from: Option<syn::Type>,
    crate_path: &syn::Path,
) -> darling::Result<TokenStream> {
    let mut errors = darling::Error::accumulator();

    let mut decode_match_contents = quote!();
    let mut encode_match_contents = quote!();

    let mut prev_case = None;

    let id_type = from.unwrap_or_else(|| syn::parse_quote! {i32});

    let variant_count = variants.len();

    for variant in variants {
        let Some(case) = variant.case.or(variant.discriminant).map(|expr|{
            // update discriminant
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(int),
                ..
            }) = &expr {
                if let Ok(num) = int.base10_parse::<i128>() {
                    prev_case = Some(num);
                }
            }
            expr
        }).or_else(||{
            if let Some(prev) = &mut prev_case {
                *prev += 1;
                let lit = proc_macro2::Literal::i128_unsuffixed(*prev);
                Some(parse_quote!(#lit))
            } else {
                let err = darling::Error::custom(crate::NEITHER_CASE_NOR_DISCRIMINANT);
                errors.push(err.with_span(&variant.ident.span()));
                None
            }
        }) else {
            continue;
        };

        let FieldsCode {
            parsing,
            destructuring,
            serialization,
        } = match fields::codegen(variant.fields, crate_path) {
            Ok(k) => k,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };

        let var_ident = variant.ident;

        quote! {
            #case => {
                #parsing
                Self::#var_ident #destructuring
            },
        }
        .to_tokens(&mut decode_match_contents);

        let encode_id = if varint.is_present() {
            quote!(#crate_path::Encode::encode(&#crate_path::attrs::Var::<#id_type>::from(#case), writer)?;)
        } else {
            quote!(<#id_type as #crate_path::Encode>::encode(&#case, writer)?;)
        };

        quote! {
            Self::#var_ident #destructuring => {
                #encode_id
                #serialization
            },
        }
        .to_tokens(&mut encode_match_contents);
    }

    errors.finish()?;

    let mut res = quote!();

    let (allow_unreachable, wildcard_match) = if variant_count == 0 {
        (
            quote!(#[allow(unreachable_code)]),
            quote!(_ => unimplemented!()),
        )
    } else {
        (quote!(), quote!())
    };

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
                #allow_unreachable
                #[allow(unused_must_use)]
                Ok(match self {
                    #encode_match_contents
                    #wildcard_match
                })
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

    let decode_id = if varint.is_present() {
        quote!(<#crate_path::attrs::Var<#id_type> as #crate_path::Decode>::decode(cursor)?.into_inner())
    } else {
        quote!(<#id_type as #crate_path::Decode>::decode(cursor)?)
    };
    quote! {
        impl #implgenerics #crate_path::Decode<'dec> for #ident #typegenerics
        #whereclause
        {
            fn decode(cursor: &mut ::std::io::Cursor<&'dec [::core::primitive::u8]>) -> #crate_path::decode::Result<Self> {
                Ok(match #decode_id {
                    #decode_match_contents
                    _ => Err(#crate_path::decode::Error::InvalidId)?
                })
            }
        }
    }
    .to_tokens(&mut res);

    Ok(res)
}
