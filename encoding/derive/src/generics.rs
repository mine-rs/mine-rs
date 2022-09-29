use syn::{parse_quote, punctuated::Punctuated, Generics, Lifetime, WhereClause, Type};

pub fn implgenerics(generics: Generics, traid: &Type, lifetime: Option<Lifetime>) -> Generics {
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
            syn::GenericParam::Type(t) => {
                where_clause.predicates.push(parse_quote! {
                    #t: #traid
                })
            }
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
