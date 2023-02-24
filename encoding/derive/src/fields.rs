use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{spanned::Spanned, Type};

use crate::{attribute::Fixed, EncodingField};

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
}

pub(crate) fn codegen(fields: darling::ast::Fields<EncodingField>, crate_path: &syn::Path) -> darling::Result<FieldsCode> {
    let mut errors = darling::Error::accumulator();
    let mut parsing = quote!();
    let mut destructuring = quote!();
    let mut serialization = quote!();

    for (i, field) in fields.fields.into_iter().enumerate() {
        let EncodingField {
            ident,
            ty,
            varint,
            fixed,
            stringuuid,
            counted,
            mutf8,
            rest,
            ..
        } = field;
        let ident = field_ident(i, ident, &ty);

        macro_rules! incompatible {
            ($main:ident $($span:ident)?: $($other:ident),* $(,)?) => {{
                let mut list = vec![];
                $(if incompatible!(@impl cond $other) {
                    list.push(stringify!($other));
                })*
                if let Some((last, others)) = list.split_last() {
                    let main_msg = if others.is_empty() {
                        format!("{} is incompatible with {}", stringify!($main), last)
                    } else {
                        format!("{} is incompatible with {} and {}", stringify!($main), others.join(", "), last)
                    };
                    let err = darling::Error::custom(main_msg);
                    errors.push(err.with_span(&incompatible!(@impl span $main $($span)?)));
                }
            }};
            (@impl cond fixed) => {fixed.is_some()};
            (@impl cond counted) => {counted.is_some()};
            (@impl cond $id:ident) => {$id.is_present()};
            (@impl span fixed $span:ident) => {$span};
            (@impl span counted $span:ident) => {$span};
            (@impl span $id:ident $(span:ident)?) => {$id.span()};
        }

        quote!(#ident,).to_tokens(&mut destructuring);

        if varint.is_present() {
            let span = varint.span();
            incompatible!(varint: fixed, stringuuid, counted, mutf8, rest);

            quote_spanned! {span=>
                let #ident = <#crate_path::attrs::Var<_> as #crate_path::Decode>::decode(cursor)?.into_inner();
            }
            .to_tokens(&mut parsing);
            quote_spanned! {span=>
                #crate_path::Encode::encode(&#crate_path::attrs::Var::<#ty>::from(*#ident), writer)?;
            }
            .to_tokens(&mut serialization);
        } else if let Some(value) = fixed.as_ref() {
            let span = fixed.span();
            let Fixed { precision, typ } = value;
            incompatible!(fixed span: stringuuid, counted, mutf8, rest);

            quote_spanned! {span=>
                let #ident = <#crate_path::attrs::Fixed<#precision, #typ, _> as #crate_path::Decode>::decode(cursor)?.into_inner();
            }
            .to_tokens(&mut parsing);
            quote_spanned! {span=>
                #crate_path::Encode::encode(&#crate_path::attrs::Fixed::<#precision, #typ, #ty>::from(#ident), writer)?;
            }
            .to_tokens(&mut serialization);
        } else if stringuuid.is_present() {
            incompatible!(stringuuid: counted, mutf8, rest);
            let span = stringuuid.span();

            quote_spanned! {span=>
                let #ident = <#crate_path::attrs::StringUuid as #crate_path::Decode>::decode(cursor)?.into_inner();
            }
            .to_tokens(&mut parsing);
            quote_spanned! {span=>
                #crate_path::Encode::encode(&#crate_path::attrs::StringUuid::from(*#ident), writer)?;
            }
            .to_tokens(&mut serialization);
        } else if let Some(value) = counted.as_ref() {
            let span = counted.span();
            incompatible!(counted span: mutf8, rest);

            quote_spanned! {span=>
                let #ident = <#crate_path::attrs::Counted<_, #value> as #crate_path::Decode>::decode(cursor)?.inner;
            }
            .to_tokens(&mut parsing);
            quote_spanned! {span=>
                #crate_path::Encode::encode(<&#crate_path::attrs::Counted<#ty, #value>>::from(#ident), writer)?;
            }
            .to_tokens(&mut serialization);
        } else if mutf8.is_present() {
            let span = mutf8.span();
            incompatible!(mutf8: rest);

            quote_spanned! {span=>
                let #ident = <#crate_path::attrs::Mutf8<_> as #crate_path::Decode>::decode(cursor)?.into_inner();
            }
            .to_tokens(&mut parsing);
            quote_spanned! {span=>
                #crate_path::Encode::encode(<#crate_path::attrs::Mutf8<_>>::from(#ident), writer)?;
            }
            .to_tokens(&mut serialization);
        } else if rest.is_present() {
            let span = rest.span();

            quote_spanned! {span=>
                let #ident = <#crate_path::attrs::Rest<_> as #crate_path::Decode>::decode(cursor)?.into_inner();
            }
            .to_tokens(&mut parsing);
            quote_spanned! {span=>
                #crate_path::Encode::encode(<&#crate_path::attrs::Rest<#ty>>::from(#ident), writer)?;
            }
            .to_tokens(&mut serialization);
        } else {
            let span = ident.span().resolved_at(Span::call_site());

            quote_spanned! {span=>
                let #ident = #crate_path::Decode::decode(cursor)?;
            }
            .to_tokens(&mut parsing);
            quote_spanned! {span=>
                #crate_path::Encode::encode(#ident, writer)?;
            }
            .to_tokens(&mut serialization);
        };
    }

    let destructuring = match fields.style {
        darling::ast::Style::Tuple => quote! ( ( #destructuring ) ),
        darling::ast::Style::Struct => quote! { { #destructuring } },
        darling::ast::Style::Unit => quote! {},
    };

    errors.finish_with(FieldsCode {
        parsing,
        destructuring,
        serialization,
    })
}

pub(crate) fn field_ident(i: usize, ident: Option<Ident>, ty: &Type) -> Ident {
    ident.unwrap_or_else(|| Ident::new(&format!("_{i}"), ty.span()))
}
