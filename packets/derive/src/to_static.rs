use proc_macro2::Ident;
use syn::parse_quote;

pub fn to_static_generics(mut generics: syn::Generics) -> syn::Generics {
    let mut where_clause = generics.where_clause.unwrap_or_else(|| syn::WhereClause {
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
    syn::Generics {
        where_clause: Some(where_clause),
        ..generics
    }
}
