use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::{parse_quote, spanned::Spanned, DataEnum, Generics, Variant};

use crate::{
    attribute::{parse_attr, Attribute},
    fields::{fields_codegen, fields_to_codegen_input, FieldsCode},
    generics::implgenerics,
};

pub fn derive_enum(
    ident: Ident,
    attrs: Vec<syn::Attribute>,
    generics: Generics,
    enom: DataEnum,
) -> TokenStream {
    let mut res = TokenStream::new();

    let mut varint = None;
    let mut from = None;

    for attr in attrs.into_iter().flat_map(parse_attr) {
        let Attribute { span, data } = match attr {
            Ok(attr) => attr,
            Err(e) => {
                e.into_compile_error().to_tokens(&mut res);
                continue;
            }
        };
        use crate::attribute::AttributeData::*;
        match data {
            VarInt => {
                if varint.is_none() {
                    varint = Some(span);
                    continue;
                } else {
                    error!(span, "`#[varint]` specified more than once")
                }
            }
            Case(_) => error!(span, "`#[case(id)]` not allowed on enum declaration"),
            From(ty) => {
                if from.is_none() {
                    from = Some(ty);
                    continue;
                } else {
                    error!(span, "`#[from(ty)]` specified more than once")
                }
            }
            Fixed(_) => error!(span, "`#[fixed(prec, ty)]` not allowed on enum declaration"),
            Counted(_) => error!(span, "#[counted(ty)] not allowed on enum declaration"),
            StringUuid => error!(span, "#[stringuuid] not allowed on enum declaration"),
            Rest => error!(span, "#[rest] not allowed on enum declaration"),
            BitField(_) => error!(span, "#[bitfield] not allowed on enum declaration"),
            Bits(_) => error!(span, "#[bits(size)] not allowed on enum declaration"),
            Bool => error!(span, "#[bool] not allowed on enum declaration"),
        }
        .to_tokens(&mut res);
    }

    let variant_count = enom.variants.len();

    let typ = from.unwrap_or_else(|| parse_quote!(i32));

    let mut decode_match_contents = quote!();
    let mut encode_match_contents = quote!();

    let mut prev = None;

    for Variant {
        attrs,
        ident,
        fields,
        discriminant,
    } in enom.variants
    {
        let mut case = discriminant.map(|(_, expr)| expr);

        for attr_res in attrs.into_iter().flat_map(parse_attr) {
            let Attribute { span, data } = match attr_res {
                Ok(attr) => attr,
                Err(e) => {
                    e.into_compile_error().to_tokens(&mut res);
                    continue;
                }
            };
            use crate::attribute::AttributeData::*;
            let err = match data {
                VarInt => "`#[varint]` not allowed on enum variant",
                Case(expr) => {
                    if case.is_none() {
                        case = Some(expr);
                        continue;
                    } else {
                        "specified more than one case"
                    }
                }
                From(_) => "`#[from(ty)]` not allowed on enum variant",
                Fixed(_) => "`#[fixed(prec, ty)]` not allowed on enum variant",
                Counted(_) => "`#[counted(ty)]` not allowed on enum variant",
                StringUuid => "`#[stringuuid]` not allowed on enum variant",
                Rest => "`#[rest]` not allowed on enum variant",
                BitField(_) => "`#[bitfield]` not allowed on enum variant",
                Bits(_) => "`#[bits(size)]` not allowed on enum variant",
                Bool => "`#[bool]` not allowed on enum variant",
            };
            error!(span, err).to_tokens(&mut res);
        }

        let case = match case {
            Some(case) => {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(int),
                    ..
                }) = &case
                {
                    match int.base10_parse::<i128>() {
                        Ok(n) => prev = Some(n),
                        Err(e) => {
                            e.into_compile_error().to_tokens(&mut res);
                            continue;
                        }
                    }
                }
                case
            }
            None => {
                if let Some(prev) = &mut prev {
                    *prev += 1;
                    // todo! support u128 probably using enum
                    let lit = proc_macro2::Literal::i128_unsuffixed(*prev);
                    parse_quote!(#lit)
                } else {
                    let span = (&typ).into_token_stream().span();
                    error!(span, "couldn't deduce id, annotate with `case` attribute")
                        .to_tokens(&mut res);
                    continue;
                }
            }
        };

        let FieldsCode {
            parsing,
            destructuring,
            serialization,
            // to_static,
            // into_static,
        } = fields_to_codegen_input(fields, &mut res)
            .map(fields_codegen)
            .unwrap_or_default();

        quote! {
            #case => {
                #parsing
                Self::#ident #destructuring
            },
        }
        .to_tokens(&mut decode_match_contents);

        let encode_id = if varint.is_some() {
            quote!(Var::<#typ>::from(#case).encode(writer)?;)
        } else {
            quote!(#typ::encode(&#case, writer)?;)
        };

        quote! {
            Self::#ident #destructuring => {
                #encode_id
                #serialization
            },
        }
        .to_tokens(&mut encode_match_contents);
    }

    let (allow_unreachable, wildcard_match) = if variant_count == 0 {
        (
            quote!(#[allow(unreachable_code)]),
            quote!(_ => unimplemented!()),
        )
    } else {
        (quote!(), quote!())
    };

    let decode_generics = implgenerics(
        generics.clone(),
        &parse_quote!(Decode),
        Some(parse_quote!('dec)),
    );
    let decode_where_clause = &decode_generics.where_clause;
    let decode_id = if varint.is_some() {
        quote!(<Var<#typ> as Decode>::decode(cursor)?.into_inner())
    } else {
        quote!(<#typ as Decode>::decode(cursor)?)
    };
    quote! {
        impl #decode_generics Decode<'dec> for #ident #generics
        #decode_where_clause
        {
            fn decode(cursor: &mut ::std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
                Ok(match #decode_id {
                    #decode_match_contents
                    _ => Err(decode::Error::InvalidId)?
                })
            }
        }
    }
    .to_tokens(&mut res);

    let encode_generics = implgenerics(generics.clone(), &parse_quote!(Encode), None);
    quote! {
        impl #encode_generics Encode for #ident #generics {
            fn encode(&self, writer: &mut impl ::std::io::Write) -> encode::Result<()> {
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

    res
}
