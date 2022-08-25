use super::{field_ident, implgenerics, Naming};
use crate::attribute::*;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse_quote, DataStruct, Generics, ItemImpl, Type};

type Attrs = (Option<Span>, Option<(Span, Fixed)>);

pub fn struct_protocol(
    ident: Ident,
    attrs: Vec<syn::Attribute>,
    generics: Generics,
    strukt: DataStruct,
) -> TokenStream {
    let mut res = quote! {};

    let mut fields: Vec<(Attrs, Ident, Type)> = Vec::with_capacity(strukt.fields.len());

    let kind = match &strukt.fields {
        syn::Fields::Named(_) => Naming::Named,
        syn::Fields::Unnamed(_) => Naming::Unnamed,
        syn::Fields::Unit => {
            let span = ident.span();
            return error!(span, "unit structs not supported").into();
        }
    };

    for attr_res in attrs.into_iter().flat_map(parse_attr) {
        let Attribute { span, data } = match attr_res {
            Ok(attr) => attr,
            Err(e) => {
                e.into_compile_error().to_tokens(&mut res);
                continue;
            }
        };
        let kind = match data {
            AttributeData::VarInt => "varint",
            AttributeData::Case(_) => "case",
            AttributeData::From(_) => continue,
            AttributeData::Fixed(_) => "fixed",
        };
        error!(span, "`{}` attribute not allowed to annotate struct", kind).to_tokens(&mut res);
    }

    for (i, field) in strukt.fields.into_iter().enumerate() {
        let mut varint_attr_span = None;
        let mut fixed = None;
        for attr_res in field.attrs.into_iter().map(parse_attr) {
            let Attribute { span, data } = match attr_res {
                Some(Ok(attr)) => attr,
                Some(Err(e)) => {
                    e.into_compile_error().to_tokens(&mut res);
                    continue;
                }
                None => continue,
            };
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
                AttributeData::Fixed(fixd) => {
                    if fixed.is_none() {
                        fixed = Some((span, fixd));
                        continue;
                    } else {
                        "duplicate `fixed` attribute on struct field"
                    }
                }
            };
            error!(span, err_message).to_tokens(&mut res)
        }

        let ident = field_ident(i, field.ident, &field.ty);

        fields.push(((varint_attr_span, fixed), ident, field.ty));
    }

    struct_protocol_read(kind, ident.clone(), generics.clone(), fields.clone()).to_tokens(&mut res);
    struct_protocol_write(kind, ident, generics, fields).to_tokens(&mut res);

    res.into()
}

fn struct_protocol_read(
    kind: Naming,
    ident: Ident,
    generics: Generics,
    fields: Vec<(Attrs, Ident, Type)>,
) -> ItemImpl {
    let mut construct = quote!();

    for (attrs, field, ty) in fields {
        let code = match attrs {
            (None, None) => {
                let span = field.span();
                quote_spanned! {span=>
                    #field: ProtocolRead::read(buf)?,
                }
            }
            (None, Some((fixed_span, fixed))) => {
                let Fixed { precision, typ, .. } = fixed;
                quote_spanned! {fixed_span=>
                    #field: <Fixed<#precision, #typ, #ty> as ProtocolRead>::read(buf)?.data,
                }
            }
            (Some(varint_span), None) => {
                quote_spanned! {varint_span=>
                    #field: <Var<#ty> as ProtocolRead>::read(buf)?.0,
                }
            }
            (Some(varint_span), Some((fixed_span, fixed))) => {
                let Fixed { precision, typ, .. } = fixed;
                let var_part = quote_spanned!(varint_span=> Var<#typ>);
                quote_spanned! {fixed_span=>
                    #field: <Fixed<#precision, #var_part, #ty> as ProtocolRead>::read(buf)?.data,
                }
            }
        };
        // let code = if let Some(span) = attrs {
        //     quote_spanned! {span=>
        //         #field: <Var<_> as ProtocolRead>::read(buf)?.0,
        //     }
        // } else {
        //     let span = field.span();
        //     quote_spanned! {span=>
        //         #field: ProtocolRead::read(buf)?,
        //     }
        // };
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
    fields: Vec<(Attrs, Ident, Type)>,
) -> ItemImpl {
    let mut deser = quote!();
    let mut destruct = quote!();
    let mut size_hints = quote!(0);
    for (attrs, field, ty) in fields {
        let code = match attrs {
            (None, None) => {
                quote!(+ <#ty as ProtocolWrite>::size_hint()).to_tokens(&mut size_hints);
                let span = field.span();
                quote_spanned! {span=>
                    ProtocolWrite::write(#field, buf)?;
                }
            }
            (None, Some((fixed_span, fixed))) => {
                let Fixed { precision, typ, .. } = fixed;
                quote!(+ <Fixed<#precision, #typ, #ty> as ProtocolWrite>::size_hint())
                    .to_tokens(&mut size_hints);
                quote_spanned! {fixed_span=>
                    ProtocolWrite::write(Fixed::<#precision, #typ, #ty>::fixed(#field), buf)?;
                }
            }
            (Some(varint_span), None) => {
                quote!(+ <Var<#ty> as ProtocolWrite>::size_hint()).to_tokens(&mut size_hints);
                quote_spanned! {varint_span=>
                    ProtocolWrite::write(Var::<#ty>(#field), buf)?;
                }
            }
            (Some(varint_span), Some((fixed_span, fixed))) => {
                let Fixed { precision, typ, .. } = fixed;
                let var_part = quote_spanned!(varint_span=>  Var<#typ>);
                quote!(+ <Fixed<#precision, #var_part, #ty> as ProtocolWrite>::size_hint())
                    .to_tokens(&mut size_hints);
                quote_spanned! {fixed_span=>
                    ProtocolWrite::write(Fixed::<#precision, #var_part, #ty>::fixed(#field), buf)?;
                }
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
