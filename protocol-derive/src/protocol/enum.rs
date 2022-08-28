#![forbid(clippy::unwrap_used, clippy::expect_used)]
use super::attribute::{parse_attr, struct_field, Attribute, AttributeData, Attrs};
use super::{field_ident, implgenerics, struct_codegen, Naming, StructCode};

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
            AttributeData::Count(_) => "`count` attribute not allowed to annotate enum",
        };
        error!(span, err).to_tokens(&mut res);
    }

    let typ = repr.unwrap_or_else(|| parse_quote!(i32));

    let mut size_hint = quote!(0);

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
                AttributeData::Count(_) => "`count` attribute not allowed to annotate enum variant",
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

        let (parsing, destructuring, serialization) = if let Some((kind, punct_fields)) =
            match variant.fields {
                syn::Fields::Named(fields) => Some((Naming::Named, fields.named)),
                syn::Fields::Unnamed(fields) => Some((Naming::Unnamed, fields.unnamed)),
                syn::Fields::Unit => None,
            } {
            let mut fields: Vec<(Attrs, Ident, Type)> = vec![];
            for (i, field) in punct_fields.into_iter().enumerate() {
                let attrs = struct_field(field.attrs.into_iter(), &mut res);

                let ident = field_ident(i, field.ident, &field.ty);

                fields.push((attrs, ident, field.ty))
            }
            let StructCode {
                parsing,
                destructuring,
                size_hint: sh,
                serialization,
            } = struct_codegen(kind, fields.clone());
            // let (descruct, write, variant_size_hint) = struct_write(kind, fields);

            size_hint = quote!(usize::max(#size_hint, #sh));

            (parsing, destructuring, serialization)
        } else {
            (quote!(), quote!(), quote!())
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
            quote!(<Var<#typ> as ProtocolWrite>::write(Var(#case), writer)?;)
        } else {
            quote!(<#typ as ProtocolWrite>::write(#case, writer)?;)
        };
        quote! {
            Self::#variant_ident #destructuring => {
                #write_id
                #serialization
            },
        }
        .to_tokens(&mut write_match_contents);
    }

    let readgenerics = implgenerics(
        generics.clone(),
        &parse_quote!(ProtocolRead),
        Some(parse_quote!('read)),
    );
    let where_clause = &readgenerics.where_clause;
    let read_id = if varint_span.is_some() {
        quote!(<Var<#typ> as ProtocolRead>::read(cursor)?.0)
    } else {
        quote!(<#typ as ProtocolRead>::read(cursor)?)
    };
    quote! {
        impl #readgenerics ProtocolRead<'read> for #ident #generics
        #where_clause
        {
            fn read(cursor: &mut ::std::io::Cursor<&'read [u8]>) -> Result<Self, ReadError> {
                Ok(match #read_id {
                    #read_match_contents
                    _ => Err(InvalidEnumId)?
                })
            }
        }
    }
    .to_tokens(&mut res);

    let size_hint_id = if varint_span.is_some() {
        quote!(<Var<#typ> as ProtocolWrite>::size_hint())
    } else {
        quote!(<#typ as ProtocolWrite>::size_hint())
    };

    let writegenerics = implgenerics(generics.clone(), &parse_quote!(ProtocolWrite), None);
    quote! {
        impl #writegenerics ProtocolWrite for #ident #generics {
            fn write(self, writer: &mut impl ::std::io::Write) -> Result<(), WriteError> {
                Ok(match self {
                    #write_match_contents
                })
            }
            #[inline(always)]
            fn size_hint() -> usize {
                #size_hint_id +
                #size_hint
            }
        }
    }
    .to_tokens(&mut res);

    res.into()
}
