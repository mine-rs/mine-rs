use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, parse_quote, DeriveInput};

#[proc_macro_derive(ToStatic)]
pub fn to_static(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let generics = input.generics.clone();
    let ident = input.ident;

    let mut to_static_generics = to_static_generics(input.generics);
    let where_clause = to_static_generics.where_clause.take();

    match input.data {
        syn::Data::Struct(strukt) => {
            let (destructuring, to_static, into_static) = fields(strukt.fields);
            quote! {
                impl #generics ToStatic for #ident #generics
                where
                    #where_clause
                {
                    type Static = #ident #to_static_generics;
                    fn to_static(&self) -> Self::Static {
                        let Self #destructuring = self;
                        #to_static
                        Self::Static #destructuring
                    }
                    fn into_static(self) -> Self::Static {
                        let Self #destructuring = self;
                        #into_static
                        Self::Static #destructuring
                    }
                }
            }
            .into()
        }
        syn::Data::Enum(enom) => {
            let mut to_static_match_contents = proc_macro2::TokenStream::new();
            let mut into_static_match_contents = proc_macro2::TokenStream::new();
            for variant in enom.variants {
                let (destructuring, to_static, into_static) = fields(variant.fields);
                let ident = variant.ident;
                quote! {
                    Self::#ident #destructuring => {
                        #to_static
                        Self::Static::#ident #destructuring
                    }
                }
                .to_tokens(&mut to_static_match_contents);
                quote! {
                    Self::#ident #destructuring => {
                        #into_static
                        Self::Static::#ident #destructuring
                    }
                }
                .to_tokens(&mut into_static_match_contents);
            }
            quote! {
                impl #generics ToStatic for #ident #generics
                where
                    #where_clause
                {
                    type Static = #ident #to_static_generics;
                    fn to_static(&self) -> Self::Static {
                        match self {
                            #to_static_match_contents
                        }
                    }
                    fn into_static(self) -> Self::Static {
                        match self {
                            #into_static_match_contents
                        }
                    }
                }
            }
            .into()
        }
        syn::Data::Union(_) => todo!("calling ToStatic on Unions is not supported"),
    }
}

fn to_static_generics(mut generics: syn::Generics) -> syn::Generics {
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

fn fields(
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
