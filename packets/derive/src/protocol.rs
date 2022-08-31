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

pub(super) fn tostaticgenerics(mut generics: Generics) -> Generics {
    let mut where_clause = generics.where_clause.unwrap_or_else(|| WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });
    for item in generics.params.iter_mut() {
        match item {
            syn::GenericParam::Lifetime(lt) => {
                lt.lifetime.ident = Ident::new("static", lt.lifetime.ident.span())
            }
            syn::GenericParam::Type(ty) => where_clause.predicates.push(parse_quote! {
                #ty: ToStatic
            }),
            _ => {}
        }
    }
    Generics {
        where_clause: Some(where_clause),
        ..generics
    }
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

#[derive(Default)]
pub struct StructCode {
    /// let x = X::read(cursor)?;
    /// let y = Y::read(cursor)?;
    pub parsing: TokenStream,
    /// (_0, _1)
    /// (a, b)
    pub destructuring: TokenStream,
    /// X::write(x, writer)?;
    /// Y::write(_0, writer)?;
    pub serialization: TokenStream,
    /// let field = field.to_static();
    pub to_static: TokenStream,
    /// let field = field.into_static();
    pub into_static: TokenStream,
}

pub fn struct_codegen(kind: Naming, fields: Vec<(Attrs, Ident, Type)>) -> StructCode {
    let mut parsing = quote!();
    let mut destructuring = quote!();
    let mut serialization = quote!();
    let mut to_static = quote!();
    let mut into_static = quote!();

    for (attrs, field, ty) in fields {
        quote!(#field,).to_tokens(&mut destructuring);
        quote!(let #field = #field.to_static();).to_tokens(&mut to_static);
        quote!(let #field = #field.into_static();).to_tokens(&mut into_static);

        match attrs {
            Attrs::None => {
                let span = field.span();
                quote_spanned! {span=>
                    let #field = Decode::decode(cursor)?;
                }
                .to_tokens(&mut parsing);
                let span = field.span();
                quote_spanned! {span=>
                    Encode::encode(#field, writer)?;
                }
                .to_tokens(&mut serialization);
            }
            Attrs::Fixed(span, fixed) => {
                let Fixed { precision, typ } = fixed;
                quote_spanned! {span=>
                    let #field = <Fixed<#precision, #typ, _> as Decode>::decode(cursor)?.into_inner();
                }
                .to_tokens(&mut parsing);
                quote_spanned! {span=>
                    Encode::encode(&Fixed::<#precision, #typ, #ty>::from(#field), writer)?;
                }
                .to_tokens(&mut serialization);
            }
            Attrs::Var(span) => {
                quote_spanned! {span=>
                    let #field = <Var<_> as Decode>::decode(cursor)?.into_inner();
                }
                .to_tokens(&mut parsing);
                quote_spanned! {span=>
                    Encode::encode(&Var::<#ty>::from(*#field), writer)?;
                }
                .to_tokens(&mut serialization);
            }
            // Attrs::FixedVar(fixed_span, fixed, varint_span) => {
            //     let Fixed { precision, typ } = fixed;
            //     let var_part = quote_spanned!(varint_span=> Var<#typ>);
            //     quote_spanned! {fixed_span=>
            //         let #field = <Fixed<#precision, #var_part, _> as Decode>::decode(cursor)?.into_inner();
            //     }.to_tokens(&mut parsing);
            //     quote_spanned! {fixed_span=>
            //         Encode::encode(Fixed::<#precision, #var_part, #ty>::new(#field), writer)?;
            //     }
            //     .to_tokens(&mut serialization);
            // }
            Attrs::StringUuid(span) => {
                quote_spanned! {span=>
                    let #field = <StringUuid as Decode>::decode(cursor)?.into_inner();
                }
                .to_tokens(&mut parsing);
                quote_spanned! {span=>
                    Encode::encode(&StringUuid::from(*#field), writer)?;
                }
                .to_tokens(&mut serialization);
            }
            Attrs::Counted(cs, c) => {
                quote_spanned! {cs=>
                    let #field = <Counted<_, #c> as Decode>::decode(cursor)?.inner;
                }
                .to_tokens(&mut parsing);
                quote_spanned! {cs=>
                    Encode::encode(<&Counted<#ty, #c>>::from(#field), writer)?;
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
        serialization,
        to_static,
        into_static,
    }
}