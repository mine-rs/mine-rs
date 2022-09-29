use proc_macro2::{Ident, TokenStream, Span};
use quote::{quote_spanned, ToTokens};
use syn::{spanned::Spanned, Fields, Type};

use crate::attribute::{field_attrs, parse_attr, Attribute, Attrs, BitField, Fixed};

#[derive(Clone, Copy)]
pub enum Naming {
    Named,
    Unnamed,
}

#[derive(Default)]
pub struct FieldsCode {
    /// let x = X::read(cursor)?;
    /// let y = Y::read(cursor)?;
    pub parsing: TokenStream,
    /// (_0, _1)
    /// (a, b)
    pub destructuring: TokenStream,
    /// X::write(x, writer)?;
    /// Y::write(_0, writer)?;
    pub serialization: TokenStream,
    // /// let field = field.to_static();
    // pub to_static: TokenStream,
    // /// let field = field.into_static();
    // pub into_static: TokenStream,
}

pub struct Field {
    attrs: Attrs,
    ident: Ident,
    ty: Type,
}

pub fn fields_to_codegen_input(
    fields: Fields,
    res: &mut TokenStream,
) -> Option<(Naming, Vec<Field>)> {
    match fields {
        syn::Fields::Named(fields) => Some((Naming::Named, fields.named)),
        syn::Fields::Unnamed(fields) => Some((Naming::Unnamed, fields.unnamed)),
        syn::Fields::Unit => None,
    }
    .map(|(kind, fields)| {
        let fields = fields
            .into_iter()
            .enumerate()
            .map(|(i, field)| {
                let attrs = field_attrs(field.attrs.into_iter(), res);
                let ident = field_ident(i, field.ident, &field.ty);
                Field {
                    attrs,
                    ident,
                    ty: field.ty,
                }
            })
            .collect();
        (kind, fields)
    })
}

fn field_ident(i: usize, ident: Option<Ident>, ty: &Type) -> Ident {
    ident.unwrap_or_else(|| Ident::new(&format!("_{i}"), ty.span()))
}

pub fn fields_codegen((kind, fields): (Naming, Vec<Field>)) -> FieldsCode {
    let mut parsing = quote!();
    let mut destructuring = quote!();
    let mut serialization = quote!();
    let mut to_static = quote!();
    let mut into_static = quote!();

    for Field { attrs, ident, ty } in fields {
        quote!(#ident,).to_tokens(&mut destructuring);
        quote!(let #ident = #ident.to_static();).to_tokens(&mut to_static);
        quote!(let #ident = #ident.into_static();).to_tokens(&mut into_static);

        match attrs {
            Attrs::None => {
                let span = ident.span().resolved_at(Span::call_site());
                quote_spanned! {span=>
                    let #ident = Decode::decode(cursor)?;
                }
                .to_tokens(&mut parsing);
                quote_spanned! {span=>
                    Encode::encode(#ident, writer)?;
                }
                .to_tokens(&mut serialization);
            }
            Attrs::Fixed(span, fixed) => {
                let Fixed { precision, typ } = fixed;
                quote_spanned! {span=>
                    let #ident = <Fixed<#precision, #typ, _> as Decode>::decode(cursor)?.into_inner();
                }
                .to_tokens(&mut parsing);
                quote_spanned! {span=>
                    Encode::encode(&Fixed::<#precision, #typ, #ty>::from(#ident), writer)?;
                }
                .to_tokens(&mut serialization);
            }
            Attrs::Var(span) => {
                quote_spanned! {span=>
                    let #ident = <Var<_> as Decode>::decode(cursor)?.into_inner();
                }
                .to_tokens(&mut parsing);
                quote_spanned! {span=>
                    Encode::encode(&Var::<#ty>::from(*#ident), writer)?;
                }
                .to_tokens(&mut serialization);
            }
            Attrs::StringUuid(span) => {
                quote_spanned! {span=>
                    let #ident = <StringUuid as Decode>::decode(cursor)?.into_inner();
                }
                .to_tokens(&mut parsing);
                quote_spanned! {span=>
                    Encode::encode(&StringUuid::from(*#ident), writer)?;
                }
                .to_tokens(&mut serialization);
            }
            Attrs::Counted(cs, c) => {
                quote_spanned! {cs=>
                    let #ident = <Counted<_, #c> as Decode>::decode(cursor)?.inner;
                }
                .to_tokens(&mut parsing);
                quote_spanned! {cs=>
                    Encode::encode(<&Counted<#ty, #c>>::from(#ident), writer)?;
                }
                .to_tokens(&mut serialization);
            }
            Attrs::Rest(cs) => {
                quote_spanned! {cs=>
                    let #ident = <Rest<_> as Decode>::decode(cursor)?.into_inner();
                }
                .to_tokens(&mut parsing);
                quote_spanned! {cs=>
                    Encode::encode(<&Rest<#ty>>::from(#ident), writer)?;
                }
                .to_tokens(&mut serialization)
            }
        }
    }

    let destructuring = match kind {
        Naming::Named => quote! { { #destructuring } },
        Naming::Unnamed => quote! ( ( #destructuring ) ),
    };

    FieldsCode {
        parsing,
        destructuring,
        serialization,
        // to_static,
        // into_static,
    }
}

pub fn bitfield_codegen(
    span: proc_macro2::Span,
    BitField(ty, reverse): BitField,
    ident: Ident,
    strukt: syn::DataStruct,
    res: &mut TokenStream,
) {
    let mut cumultative_size = 0;

    let mut bitfields: Vec<(bool, u8, Ident, Type)> = vec![];

    let mut destructuring = quote! {};

    let kind = match strukt.fields {
        Fields::Named(_) => Naming::Named,
        Fields::Unnamed(_) => Naming::Unnamed,
        Fields::Unit => panic!("`#[bitfield]` not allowed on unit structs"),
    };

    // let fields: Vec<_> = match reverse {
    //     true => strukt.fields.into_iter().rev().enumerate().collect(),
    //     false => strukt.fields.into_iter().enumerate().collect(),
    // };

    for (
        i,
        syn::Field {
            attrs, ident, ty, ..
        },
    ) in strukt.fields.into_iter().enumerate()
    {
        let mut bool_and_size = None;
        for attr in attrs.into_iter().flat_map(parse_attr) {
            let Attribute { span, data } = match attr {
                Ok(attr) => attr,
                Err(e) => {
                    e.into_compile_error().to_tokens(res);
                    continue;
                }
            };
            use crate::attribute::AttributeData::*;
            let err = match data {
                VarInt => "`#[varint]`",
                Case(_) => "`#[case(id)]`",
                From(_) => "`#[from(ty)]`",
                Fixed(_) => "`#[fixed(prec, ty)]`",
                Counted(_) => "`#[counted(ty)]`",
                Rest => "`#[rest]`",
                StringUuid => "`#[stringuuid]`",
                BitField(_) => "`#[bitfield]`",
                Bits(s) => {
                    if bool_and_size.is_some() {
                        error!(
                            span,
                            "duplicate `#[bits(size)]` or `#[bool]` attribute specified on field"
                        )
                        .to_tokens(res);
                    } else {
                        bool_and_size = Some((false, s));
                    }
                    continue;
                }
                Bool => {
                    if bool_and_size.is_some() {
                        error!(
                            span,
                            "duplicate `#[bits(size)]` or `#[bool]` attribute specified on field"
                        )
                        .to_tokens(res);
                    } else {
                        bool_and_size = Some((true, 1));
                    }
                    continue;
                }
            };
            error!(span, "{} not allowed on field", err).to_tokens(res)
        }
        let (is_bool, size) = match bool_and_size {
            Some(size) => size,
            None => {
                let span = ident.span();
                error!(span, "no `#[bits(size)]` attribute on `#[bitfield]` field").to_tokens(res);
                continue;
            }
        };
        cumultative_size += size;

        let ident =
            ident.unwrap_or_else(|| Ident::new(&format!("_{i}"), proc_macro2::Span::call_site()));

        quote! {#ident,}.to_tokens(&mut destructuring);

        bitfields.push((is_bool, size, ident, ty));
    }

    let destructuring = match kind {
        Naming::Named => quote! {{#destructuring}},
        Naming::Unnamed => quote!((#destructuring)),
    };

    let ty = match ty {
        Some(ty) => ty.to_token_stream(),
        None => match cumultative_size {
            0..=8 => quote!(u8),
            9..=16 => quote!(u16),
            17..=32 => quote!(u32),
            33..=64 => quote!(u64),
            65..=128 => quote!(u128),
            _ => {
                error!(
                span,
                "bitfield size too large, maximum is currently `u128`, rust's largest number type"
            )
                .to_tokens(res);
                return;
            }
        },
    };

    /* | ((x as ty & (ty::BITS as ty - size)) << offset) */
    let mut encode = quote! {0};
    let mut decode = quote! {};
    let mut neg_checks = quote! {};

    let mut offset = 0;

    for (is_bool, size, ident, field_ty) in bitfields {
        let shift = if reverse {
            let shift = quote! { (#offset as #ty)};
            offset += size;
            shift
        } else {
            offset += size;
            quote! { (#ty::BITS as #ty - #offset as #ty) }
        };
        if !is_bool {
            quote! {
                | ((*#ident as #ty & (!0 >> (#ty::BITS as #ty - #size as #ty))) << #shift)
            }
            .to_tokens(&mut encode);
            quote!{
                let mut #ident = ((__value >> #shift) & !0 >> (#ty::BITS as #ty - #size as #ty)) as _;
            }.to_tokens(&mut decode);
            quote! {
                if #field_ty::MIN != 0 && #ident >= 1 << (#size as #field_ty - 1) {
                    #ident ^= !0 << #size as #field_ty;
                }
            }
            .to_tokens(&mut neg_checks);
        } else {
            quote! {
                | ((*#ident as #ty & (!0 >> (#ty::BITS as #ty - #size as #ty))) << #shift)
            }
            .to_tokens(&mut encode);
            quote!{
                let mut #ident = ((__value >> #shift) & !0 >> (#ty::BITS as #ty - #size as #ty)) != 0;
            }.to_tokens(&mut decode);
        }
    }

    quote! {
        impl<'dec> Decode<'dec> for #ident {
            fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
                let __value = #ty::decode(cursor)?;
                #decode
                #neg_checks
                Ok(Self #destructuring)
            }
        }
        impl Encode for #ident {
            fn encode(&self, writer: &mut impl ::std::io::Write) -> encode::Result<()> {
                let Self #destructuring = self;
                #[allow(clippy::identity_op)]
                (#encode).encode(writer)
            }
        }
    }
    .to_tokens(res)
}
