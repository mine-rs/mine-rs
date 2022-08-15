use proc_macro2::Ident;
use quote::spanned::Spanned;
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
enum Naming {
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
