use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse_macro_input, parse_quote, DeriveInput};

#[proc_macro_derive(ToStatic)]
pub fn to_static(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = input.ident;

    let (generics, the_static) = to_static_generics(input.generics, &ident);
    let (implgenerics, typegenerics, whereclause) = generics.split_for_impl();

    let span = Span::mixed_site();

    match input.data {
        syn::Data::Struct(strukt) => {
            let (destructuring, to_static, into_static) = fields(strukt.fields);
            quote_spanned! {span=>
                impl #implgenerics ToStatic for #ident #typegenerics
                #whereclause
                {
                    type Static = #the_static;
                    fn to_static(&self) -> Self::Static {
                        let Self #destructuring = self;
                        #to_static
                        #ident #destructuring
                    }
                    fn into_static(self) -> Self::Static {
                        let Self #destructuring = self;
                        #into_static
                        #ident #destructuring
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
            quote_spanned! {span=>
                impl #implgenerics ToStatic for #ident #typegenerics
                #whereclause
                {
                    type Static = #the_static;
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
        syn::Data::Union(_) => quote_spanned! {span=>
            impl #implgenerics ToStatic for #ident #typegenerics
            #whereclause
            {
                type Static = #the_static;
                fn to_static(&self) -> Self::Static {
                    Self::Static
                }
                fn into_static(self) -> Self::Static {
                    Self::Static
                }
            }
        }
        .into(),
    }
}

fn to_static_generics(mut generics: syn::Generics, ident: &Ident) -> (syn::Generics, syn::Path) {
    let lifetimes = generics.lifetimes().next().is_some().then(|| {
        generics.lifetimes().map(|ltdef| syn::Lifetime {
            apostrophe: ltdef.lifetime.apostrophe,
            ident: Ident::new("static", Span::mixed_site()),
        })
    });

    let type_params = generics
        .type_params()
        .next()
        .is_some()
        .then(|| generics.type_params().map(|tp| &tp.ident));

    let the_static: syn::Path = if lifetimes.is_some() || type_params.is_some() {
        let lifetime_iter = lifetimes.into_iter().flatten();
        let type_param_iter = type_params.into_iter().flatten();
        parse_quote! {
            #ident <#(#lifetime_iter,)* #(<#type_param_iter as ToStatic>::Static),*>
        }
    } else {
        drop(lifetimes);
        drop(type_params);
        parse_quote! {#ident}
    };

    for typ in generics.type_params_mut() {
        typ.bounds.push(parse_quote!(ToStatic));
    }

    (generics, the_static)
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
