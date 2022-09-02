use proc_macro2::Ident;
use syn::parse_quote;

pub fn generics(mut generics: syn::Generics) -> syn::Generics {
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

pub fn fields(
    fields: syn::Fields,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let field_iter = fields
        .iter()
        .enumerate()
        .map(|(num, syn::Field { ident, .. })| {
            ident.clone().unwrap_or_else(|| {
                proc_macro2::Ident::new(&format!("_{num}"), proc_macro2::Span::mixed_site())
            })
        });
    let destructuring: proc_macro2::TokenStream =
        field_iter.clone().map(|ident| quote!(#ident,)).collect();
    let to_static: proc_macro2::TokenStream = field_iter
        .clone()
        .map(|ident| quote!(let #ident = #ident.to_static();))
        .collect();
    let into_static: proc_macro2::TokenStream = field_iter
        .map(|ident| quote!(let #ident = #ident.into_static();))
        .collect();
    let destructuring = match fields {
        syn::Fields::Named(_) => quote! {{#destructuring}},
        syn::Fields::Unnamed(_) => quote! {(#destructuring)},
        syn::Fields::Unit => quote!(),
    };
    (destructuring, to_static, into_static)
}
