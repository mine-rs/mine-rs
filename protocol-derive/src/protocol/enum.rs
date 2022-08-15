#![forbid(clippy::unwrap_used, clippy::expect_used)]
use super::{field_ident, implgenerics, Naming};
use crate::attribute::*;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
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

    for Attribute { span, data } in attrs.into_iter().flat_map(TryFrom::try_from) {
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
        };
        error!(span, err).to_tokens(&mut res);
    }

    let typ = repr.unwrap_or_else(|| parse_quote!(i32));

    let mut size_hint = quote!(0);

    for variant in enom.variants {
        let mut case = variant.discriminant.map(|(_, expr)| expr);

        for Attribute { span, data } in variant.attrs.into_iter().flat_map(TryFrom::try_from) {
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
                    let span = typ.span();
                    error!(span, "couldn't deduce id, annotate with `case` attribute")
                        .to_tokens(&mut res);
                    continue;
                }
            }
        };

        let (construct, destruct, write) = if let Some((kind, punct_fields)) = match variant.fields
        {
            syn::Fields::Named(fields) => Some((Naming::Named, fields.named)),
            syn::Fields::Unnamed(fields) => Some((Naming::Named, fields.unnamed)),
            syn::Fields::Unit => None,
        } {
            let mut fields: Vec<(Option<Span>, Ident, Type)> = vec![];
            for (i, field) in punct_fields.into_iter().enumerate() {
                let mut varint_span = None;
                for Attribute { span, data } in field.attrs.into_iter().flat_map(TryFrom::try_from)
                {
                    let err = match data {
                        AttributeData::VarInt => {
                            if varint_span.is_none() {
                                varint_span = Some(span);
                                continue;
                            } else {
                                "duplicate `varint` attribute on enum struct field"
                            }
                        }
                        AttributeData::Case(_) => {
                            "`case` attribute not allowed to annotate enum struct fields"
                        }
                        AttributeData::From(_) => {
                            "`from` attribute not allowed to annotate enum struct fields"
                        }
                    };
                    error!(span, err).to_tokens(&mut res);
                }
                let ident = field_ident(i, field.ident, &field.ty);
                fields.push((varint_span, ident, field.ty))
            }
            let construct = enum_protocol_read(kind, fields.clone());
            let (descruct, write, variant_size_hint) = enum_protocol_write(kind, fields);

            size_hint = quote!(usize::max(#size_hint, #variant_size_hint));

            (construct, descruct, write)
        } else {
            (quote!(), quote!(), quote!())
        };

        let variant_ident = variant.ident;
        quote! {
            #case => {
                Self::#variant_ident #construct
            },
        }
        .to_tokens(&mut read_match_contents);
        let write_id = if varint_span.is_some() {
            quote!(<Var<#typ> as ProtocolWrite>::write(Var(#case), writer)?;)
        } else {
            quote!(<#typ as ProtocolWrite>::write(#case, writer)?;)
        };
        quote! {
            Self::#variant_ident #destruct => {
                #write_id
                #write
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

/*

let int: i32 = ProtocolRead::read(buf)?;
// let int: i32 = <Var<_> as ProtocolRead>::read(buf)?.0;

Ok(match int {
    #expr => #Name::#Field,
    #expr => #Name::#Field(
        _0: ProtocolRead::read(buf)?,
        _1: ProtocolRead::read(buf)?,
        _x: ProtocolRead::read(buf)?
    ),
    #expr => #Name::#Field{
        a: ProtocolRead::read(buf)?,
        b: ProtocolRead::read(buf)?,
    }
})

*/

fn enum_protocol_read(kind: Naming, fields: Vec<(Option<Span>, Ident, Type)>) -> TokenStream2 {
    let mut construct = quote!();

    for (varint_attr_span, field, _ty) in fields {
        let code = if let Some(span) = varint_attr_span {
            quote_spanned! {span=>
                #field: <Var<_> as ProtocolRead>::read(buf)?.0,
            }
        } else {
            let span = field.span();
            quote_spanned! {span=>
                #field: ProtocolRead::read(buf)?,
            }
        };
        code.to_tokens(&mut construct);
    }

    match kind {
        Naming::Named => quote! { { #construct } },
        Naming::Unnamed => quote!( (#construct) ),
    }
}

/*

match self {
// match self.0 {
    #Name::#Field => {
        <i32 as ProtocolWrite>::write(#expr)?;
        // <Var<i32> as ProtocolWrite>::write(Var(#expr))?;
    },
    #Name::#Field(_0, _1, _x) => {
        <i32 as ProtocolWrite>::write(#expr)?;
        // <Var<i32> as ProtocolWrite>::write(Var(#expr))?;
        ProtocolWrite::write(_0)?;
        ProtocolWrite::write(_1)?;
        ProtocolWrite::write(_x)?;
    },
    #Name::#Field { a, b } => {
        <i32 as ProtocolWrite>::write(#expr)?;
        // <Var<i32> as ProtocolWrite>::write(Var(#expr))?;
        ProtocolWrite::write(a)?;
        ProtocolWrite::write(b)?;
    }
}


let int: i32 = ProtocolRead::read(buf)?;
// let int: i32 = <Var<_> as ProtocolRead>::read(buf)?.0;

Ok(match int {
    #expr => #Name::#Field,
    #expr => #Name::#Field(
        _0: ProtocolRead::read(buf)?,
        _1: ProtocolRead::read(buf)?,
        _x: ProtocolRead::read(buf)?
    ),
    #expr => #Name::#Field{
        a: ProtocolRead::read(buf)?,
        b: ProtocolRead::read(buf)?,
    }
})

*/
fn enum_protocol_write(
    kind: Naming,
    fields: Vec<(Option<Span>, Ident, Type)>,
) -> (TokenStream2, TokenStream2, TokenStream2) {
    let mut deser = quote!();
    let mut destruct = quote!();
    let mut size_hint = quote!(0);

    for (varint_attr_span, field, ty) in fields {
        let code = if let Some(span) = varint_attr_span {
            quote!( + <Var<#ty> as ProtocolWrite>::size_hint()).to_tokens(&mut size_hint);
            quote_spanned! {span=>
                ProtocolWrite::write(Var(#field), buf)?;
            }
        } else {
            quote!( + <#ty as ProtocolWrite>::size_hint()).to_tokens(&mut size_hint);
            let span = field.span();
            quote_spanned! {span=>
                ProtocolWrite::write(#field, buf)?;
            }
        };
        code.to_tokens(&mut deser);
        quote!(#field,).to_tokens(&mut destruct);
    }

    let destruct = match kind {
        Naming::Named => quote! { { #destruct } },
        Naming::Unnamed => quote!( (#destruct) ),
    };

    (destruct, deser, size_hint)
}
