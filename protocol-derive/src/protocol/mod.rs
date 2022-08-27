use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned, spanned::Spanned, ToTokens};
use syn::parse_quote;
use syn::Generics;
use syn::Lifetime;
use syn::Type;
use syn::WhereClause;

macro_rules! error {
    ($span:ident, $id:ident) => {
        syn::Error::new($span, $id)
            .to_compile_error()
    };
    ($span:ident, $lit:literal) => {
        syn::Error::new($span, $lit)
            .to_compile_error()
    };
    ($span:ident, $($t:tt),+) => {
        syn::Error::new($span, format!($($t),+))
            .to_compile_error()
    };
}

mod r#struct;
pub use r#struct::struct_protocol;
mod r#enum;
pub use r#enum::enum_protocol;
use syn::punctuated::Punctuated;

#[derive(Clone, Copy)]
pub enum Naming {
    Named,
    Unnamed,
}

fn implgenerics(generics: Generics, traid: &Ident, lifetime: Option<Lifetime>) -> Generics {
    let mut where_clause = generics.where_clause.unwrap_or_else(|| WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });
    let mut params = Punctuated::new();
    if let Some(lifetime) = &lifetime {
        params.push(parse_quote!(#lifetime));
    }
    for param in generics.params.into_iter() {
        match &param {
            syn::GenericParam::Type(t) => where_clause.predicates.push(parse_quote! {
                #t: #traid,
            }),
            syn::GenericParam::Lifetime(lt) => {
                if let Some(lifetime) = &lifetime {
                    where_clause.predicates.push(parse_quote! {
                        #lifetime: #lt
                    })
                }
            }
            _ => {}
        }
        params.push(param);
    }

    Generics {
        where_clause: Some(where_clause),
        params,
        ..generics
    }
}

fn field_ident(i: usize, ident: Option<Ident>, ty: &Type) -> Ident {
    ident.unwrap_or_else(|| Ident::new(&format!("_{i}"), ty.__span()))
}

mod attribute;
use attribute::{Attrs, Fixed};

pub struct StructCode {
    /// let x = X::read(cursor)?;
    /// let y = Y::read(cursor)?;
    pub parsing: TokenStream,
    /// (_0, _1)
    /// (a, b)
    pub destructuring: TokenStream,
    /// 0 + X::size_hint() + Y::size_hint()
    pub size_hint: TokenStream,
    /// X::write(x, writer)?;
    /// Y::write(_0, writer)?;
    pub serialization: TokenStream,
}

pub fn struct_codegen(kind: Naming, fields: Vec<(Attrs, Ident, Type)>) -> StructCode {
    let mut parsing = quote!();
    let mut destructuring = quote!();
    let mut size_hint = quote!(0);
    let mut serialization = quote!();

    for (attrs, field, ty) in fields {
        quote!(#field,).to_tokens(&mut destructuring);

        match attrs {
            Attrs::None => {
                let span = field.span();
                quote_spanned! {span=>
                    let #field = ProtocolRead::read(cursor)?;
                }
                .to_tokens(&mut parsing);
                quote!(+ <#ty as ProtocolWrite>::size_hint()).to_tokens(&mut size_hint);
                let span = field.span();
                quote_spanned! {span=>
                    ProtocolWrite::write(#field, writer)?;
                }
                .to_tokens(&mut serialization);
            }
            Attrs::Fixed(span, fixed) => {
                let Fixed { precision, typ } = fixed;
                quote_spanned! {span=>
                    let #field = <Fixed<#precision, #typ, #ty> as ProtocolRead>::read(cursor)?.data;
                }
                .to_tokens(&mut parsing);
                quote!(+ <Fixed<#precision, #typ, #ty> as ProtocolWrite>::size_hint())
                    .to_tokens(&mut size_hint);
                quote_spanned! {span=>
                    ProtocolWrite::write(Fixed::<#precision, #typ, #ty>::fixed(#field), writer)?;
                }
                .to_tokens(&mut serialization);
            }
            Attrs::Var(span) => {
                quote_spanned! {span=>
                    let #field = <Var<#ty> as ProtocolRead>::read(cursor)?.0;
                }
                .to_tokens(&mut parsing);
                quote!(+ <Var<#ty> as ProtocolWrite>::size_hint()).to_tokens(&mut size_hint);
                quote_spanned! {span=>
                    ProtocolWrite::write(Var::<#ty>(#field), writer)?;
                }
                .to_tokens(&mut serialization);
            }
            Attrs::FixedVar(fixed_span, fixed, varint_span) => {
                let Fixed { precision, typ } = fixed;
                let var_part = quote_spanned!(varint_span=> Var<#typ>);
                quote_spanned! {fixed_span=>
                    let #field = <Fixed<#precision, #var_part, #ty> as ProtocolRead>::read(cursor)?.data;
                }.to_tokens(&mut parsing);
                let var_part = quote_spanned!(varint_span=>  Var<#typ>);
                quote!(+ <Fixed<#precision, #var_part, #ty> as ProtocolWrite>::size_hint())
                    .to_tokens(&mut size_hint);
                quote_spanned! {fixed_span=>
                    ProtocolWrite::write(Fixed::<#precision, #var_part, #ty>::fixed(#field), writer)?;
                }
                .to_tokens(&mut serialization);
            }
            Attrs::StringUuid(span) => {
                quote_spanned! {span=>
                    let #field = <StringUuid as ProtocolRead>::read(cursor)?.0;
                }
                .to_tokens(&mut parsing);
                quote!(+ <StringUuid as ProtocolWrite>::size_hint()).to_tokens(&mut size_hint);
                quote_spanned! {span=>
                    ProtocolWrite::write(StringUuid(#field), writer)?;
                }
                .to_tokens(&mut serialization);
            }
        }
    }

    let destructuring = match kind {
        Naming::Named => quote! { { #destructuring } },
        Naming::Unnamed => quote!( (#destructuring) ),
    };

    StructCode {
        parsing,
        destructuring,
        size_hint,
        serialization,
    }
}
