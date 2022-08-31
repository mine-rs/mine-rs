use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, Fields, Type};

use crate::attribute::{field_attrs, Attrs, Fixed};

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
                let span = ident.span();
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
