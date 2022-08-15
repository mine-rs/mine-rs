use super::{field_ident, implgenerics, Naming};
use crate::attribute::*;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse_quote, DataStruct, Generics, ItemImpl, Type};

pub fn struct_protocol(
    ident: Ident,
    attrs: Vec<syn::Attribute>,
    generics: Generics,
    strukt: DataStruct,
) -> TokenStream {
    let mut res = quote! {};

    let mut fields: Vec<(Option<Span>, Ident, Type)> = Vec::with_capacity(strukt.fields.len());

    let kind = match &strukt.fields {
        syn::Fields::Named(_) => Naming::Named,
        syn::Fields::Unnamed(_) => Naming::Unnamed,
        syn::Fields::Unit => {
            let span = ident.span();
            return error!(span, "unit structs not supported").into();
        }
    };

    for Attribute { span, data } in attrs.into_iter().flat_map(TryFrom::try_from) {
        let kind = match data {
            AttributeData::VarInt => "varint",
            AttributeData::Case(_) => "case",
            AttributeData::From(_) => continue,
        };
        error!(span, "`{}` attribute not allowed to annotate struct", kind).to_tokens(&mut res);
    }

    for (i, field) in strukt.fields.into_iter().enumerate() {
        let mut varint_attr_span = None;
        for Attribute { span, data } in field.attrs.into_iter().flat_map(TryFrom::try_from) {
            let err_message = match data {
                AttributeData::VarInt => {
                    if varint_attr_span.is_none() {
                        varint_attr_span = Some(span);
                        continue;
                    } else {
                        "duplicate `varint` attribute on struct field"
                    }
                }
                AttributeData::Case(_) => "`case` attribute not allowed to annotate struct field",
                AttributeData::From(_) => continue,
            };
            error!(span, err_message).to_tokens(&mut res)
        }

        let ident = field_ident(i, field.ident, &field.ty);

        fields.push((varint_attr_span, ident, field.ty));
    }

    struct_protocol_read(kind, ident.clone(), generics.clone(), fields.clone()).to_tokens(&mut res);
    struct_protocol_write(kind, ident, generics, fields).to_tokens(&mut res);

    res.into()
}

fn struct_protocol_read(
    kind: Naming,
    ident: Ident,
    generics: Generics,
    fields: Vec<(Option<Span>, Ident, Type)>,
) -> ItemImpl {
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

    let construct = match kind {
        Naming::Named => quote! { Self { #construct } },
        Naming::Unnamed => quote!( Self(#construct) ),
    };

    let implgenerics = implgenerics(
        generics.clone(),
        &parse_quote!(ProtocolRead),
        Some(parse_quote!('read)),
    );
    let where_clause = &implgenerics.where_clause;

    parse_quote! {
        impl #implgenerics ProtocolRead<'read> for #ident #generics
        #where_clause
        {
            fn read(buf: &mut std::io::Cursor<&'read [u8]>) -> Result<Self, ReadError> {
                Ok(#construct)
            }
        }
    }
}

fn struct_protocol_write(
    kind: Naming,
    ident: Ident,
    generics: Generics,
    fields: Vec<(Option<Span>, Ident, Type)>,
) -> ItemImpl {
    let mut deser = quote!();
    let mut destruct = quote!();
    let mut size_hints = quote!(0);
    for (varint_attr_span, field, ty) in fields {
        let code = if let Some(span) = varint_attr_span {
            quote!(+ <Var<#ty> as ProtocolWrite>::size_hint()).to_tokens(&mut size_hints);
            quote_spanned! {span=>
                ProtocolWrite::write(Var(#field), buf)?;
            }
        } else {
            quote!(+ <#ty as ProtocolWrite>::size_hint()).to_tokens(&mut size_hints);
            let span = field.span();
            quote_spanned! {span=>
                ProtocolWrite::write(#field, buf)?;
            }
        };
        code.to_tokens(&mut deser);
        quote!(#field,).to_tokens(&mut destruct);
    }

    let destruct = match kind {
        Naming::Named => quote! { Self { #destruct } },
        Naming::Unnamed => quote!( Self(#destruct) ),
    };

    let implgenerics = implgenerics(generics.clone(), &parse_quote!(ProtocolWrite), None);

    parse_quote! {
        impl #implgenerics ProtocolWrite for #ident #generics {
            fn write(self, buf: &mut impl ::std::io::Write) -> Result<(), WriteError> {
                let #destruct = self;
                #deser
                Ok(())
            }
            #[inline(always)]
            fn size_hint() -> usize {
                #size_hints
            }
        }
    }
}
