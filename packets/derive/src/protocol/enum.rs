#![forbid(clippy::unwrap_used, clippy::expect_used)]
use super::attribute::{parse_attr, struct_field, Attribute, AttributeData, Attrs};
use super::{field_ident, implgenerics, struct_codegen, tostaticgenerics, Naming, StructCode};

use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal};
use quote::spanned::Spanned;
use quote::{quote, ToTokens};
use syn::Generics;
use syn::{parse_quote, DataEnum, ExprLit, Lit, Type};

pub fn enum_protocol(
    attrs: Vec<syn::Attribute>,
    ident: Ident,
    generics: Generics,
    enom: DataEnum,
) -> TokenStream {
    let mut res = quote! {};

    let mut read_match_contents = quote!();
    let mut write_match_contents = quote!();
    let mut to_static_match_contents = quote!();
    let mut into_static_match_contents = quote!();

    let mut prev = None;

    let mut varint_span = None;
    let mut repr = None;

    for attr_res in attrs.into_iter().flat_map(parse_attr) {
        let Attribute { span, data } = match attr_res {
            Ok(attr) => attr,
            Err(e) => {
                e.into_compile_error().to_tokens(&mut res);
                continue;
            }
        };
        let err = match data {
            AttributeData::VarInt => {
                if varint_span.is_none() {
                    varint_span = Some(span);
                    continue;
                } else {
                    "specified `varint` twice"
                }
            }
            AttributeData::Case(_) => "`case` attribute not allowed to annotate enum",
            AttributeData::From(rep) => {
                if repr.is_none() {
                    repr = Some(rep);
                    continue;
                } else {
                    "`repr` specified more than once, only using the first one"
                }
            }
            AttributeData::Fixed(_) => "`fixed` attribute not allowed to annotate enum",
            AttributeData::StringUuid => "`stringuuid` attribute not allowed to annotate enum",
            AttributeData::Counted(_) => "`count` attribute not allowed to annotate enum",
        };
        error!(span, err).to_tokens(&mut res);
    }

    let typ = repr.unwrap_or_else(|| parse_quote!(i32));

    let variant_count = enom.variants.len();

    for variant in enom.variants {
        let mut case = variant.discriminant.map(|(_, expr)| expr);

        for attr_res in variant.attrs.into_iter().flat_map(parse_attr) {
            let Attribute { span, data } = match attr_res {
                Ok(attr) => attr,
                Err(e) => {
                    e.into_compile_error().to_tokens(&mut res);
                    continue;
                }
            };
            let err = match data {
                AttributeData::VarInt => "`varint` attribute not allowed to annotate enum variant",
                AttributeData::Case(expr) => {
                    if case.is_none() {
                        case = Some(expr);
                        continue;
                    } else {
                        "specified more than one case"
                    }
                }
                AttributeData::From(_) => "`from` attribute not allowed to annotate enum variant",
                AttributeData::Fixed(_) => "`fixed` attribute not allowed to annotate enum variant",
                AttributeData::StringUuid => {
                    "`stringuuid` attribute not allowed to annotate enum variant"
                }
                AttributeData::Counted(_) => {
                    "`count` attribute not allowed to annotate enum variant"
                }
            };
            error!(span, err).to_tokens(&mut res);
        }

        let case = match case {
            Some(case) => {
                if let syn::Expr::Lit(ExprLit {
                    lit: Lit::Int(int), ..
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
                    let lit = Literal::i128_unsuffixed(*prev);
                    parse_quote!(#lit)
                } else {
                    let span = (&typ).into_token_stream().__span();
                    error!(span, "couldn't deduce id, annotate with `case` attribute")
                        .to_tokens(&mut res);
                    continue;
                }
            }
        };

        let StructCode {
            parsing,
            destructuring,
            serialization,
            to_static,
            into_static,
        } = {
            match variant.fields {
                syn::Fields::Named(fields) => Some((Naming::Named, fields.named)),
                syn::Fields::Unnamed(fields) => Some((Naming::Unnamed, fields.unnamed)),
                syn::Fields::Unit => None,
            }
            .map(|(kind, punct_fields)| {
                let mut fields: Vec<(Attrs, Ident, Type)> = vec![];
                for (i, field) in punct_fields.into_iter().enumerate() {
                    let attrs = struct_field(field.attrs.into_iter(), &mut res);

                    let ident = field_ident(i, field.ident, &field.ty);

                    fields.push((attrs, ident, field.ty))
                }
                struct_codegen(kind, fields.clone())
            })
            .unwrap_or_default()
        };

        let variant_ident = variant.ident;
        quote! {
            #case => {
                #parsing
                Self::#variant_ident #destructuring
            },
        }
        .to_tokens(&mut read_match_contents);
        let write_id = if varint_span.is_some() {
            quote!(Var::<#typ>::from(#case).encode(writer)?;)
        } else {
            quote!(#typ::encode(&#case, writer)?;)
        };
        quote! {
            Self::#variant_ident #destructuring => {
                #write_id
                #serialization
            },
        }
        .to_tokens(&mut write_match_contents);

        quote! {
            Self::#variant_ident #destructuring => {
                #to_static
                Self::Static::#variant_ident #destructuring
            },
        }
        .to_tokens(&mut to_static_match_contents);
        quote! {
            Self::#variant_ident #destructuring => {
                #into_static
                Self::Static::#variant_ident #destructuring
            },
        }
        .to_tokens(&mut into_static_match_contents);
    }

    let (allow_unreachable, wildcard_match) = if variant_count == 0 {
        (
            quote!(#[allow(unreachable_code)]),
            quote!(_ => unimplemented!()),
        )
    } else {
        (quote!(), quote!())
    };

    let readgenerics = implgenerics(
        generics.clone(),
        &parse_quote!(Decode),
        Some(parse_quote!('read)),
    );
    let where_clause = &readgenerics.where_clause;
    let read_id = if varint_span.is_some() {
        quote!(<Var<#typ> as Decode>::decode(cursor)?.into_inner())
    } else {
        quote!(<#typ as Decode>::decode(cursor)?)
    };
    quote! {
        impl #readgenerics Decode<'read> for #ident #generics
        #where_clause
        {
            fn decode(cursor: &mut ::std::io::Cursor<&'read [u8]>) -> decode::Result<Self> {
                Ok(match #read_id {
                    #read_match_contents
                    _ => Err(decode::Error::InvalidId)?
                })
            }
        }
    }
    .to_tokens(&mut res);

    let writegenerics = implgenerics(generics.clone(), &parse_quote!(Encode), None);
    quote! {
        impl #writegenerics Encode for #ident #generics {
            fn encode(&self, writer: &mut impl ::std::io::Write) -> encode::Result<()> {
                #allow_unreachable
                #[allow(unused_must_use)]
                Ok(match self {
                    #write_match_contents
                    #wildcard_match
                })
            }
        }
    }
    .to_tokens(&mut res);

    let mut tostaticgenerics = tostaticgenerics(generics.clone());
    let tostaticwhere = tostaticgenerics.where_clause.take();
    quote! {
        impl #generics ToStatic for #ident #generics
        where
            #tostaticwhere
        {
            type Static = #ident #tostaticgenerics;
            fn to_static(&self) -> Self::Static {
                #allow_unreachable
                match self {
                    #to_static_match_contents
                    #wildcard_match
                }
            }
            fn into_static(self) -> Self::Static {
                #allow_unreachable
                match self {
                    #into_static_match_contents
                    #wildcard_match
                }
            }
        }
    }
    .to_tokens(&mut res);

    res.into()
}