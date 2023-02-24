use darling::{FromDeriveInput, FromField, FromVariant};
use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse_macro_input, parse_quote, DeriveInput};

fn default_crate_path() -> syn::Path {
    parse_quote!(::miners_to_static)
}

#[derive(FromDeriveInput)]
#[darling(attributes(to_static))]
struct ToStaticInput {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<ToStaticVariant, ToStaticField>,

    // flags
    /// crate path for reexporting
    crate_path: Option<syn::Path>,
}

#[derive(FromVariant)]
struct ToStaticVariant {
    ident: syn::Ident,
    fields: darling::ast::Fields<ToStaticField>,
}
#[derive(FromField)]
struct ToStaticField {
    ident: Option<syn::Ident>,
}

#[proc_macro_derive(ToStatic)]
pub fn to_static(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let deriveinput = parse_macro_input!(input as DeriveInput);

    let ToStaticInput {
        ident,
        generics,
        data,
        crate_path,
    } = match ToStaticInput::from_derive_input(&deriveinput) {
        Ok(k) => k,
        Err(e) => return e.write_errors().into(),
    };

    let crate_path = crate_path.unwrap_or_else(default_crate_path);

    let (generics, the_static) = to_static_generics(generics, &ident, &crate_path);
    let (implgenerics, typegenerics, whereclause) = generics.split_for_impl();

    let span = Span::mixed_site();

    match data {
        darling::ast::Data::Struct(fields) => {
            let (destructuring, to_static, into_static) = fields_codegen(fields, &crate_path);
            quote_spanned! {span=>
                impl #implgenerics #crate_path::ToStatic for #ident #typegenerics
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
        darling::ast::Data::Enum(variants) => {
            let mut to_static_match_contents = proc_macro2::TokenStream::new();
            let mut into_static_match_contents = proc_macro2::TokenStream::new();
            for variant in variants {
                let (destructuring, to_static, into_static) =
                    fields_codegen(variant.fields, &crate_path);
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
                impl #implgenerics #crate_path::ToStatic for #ident #typegenerics
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
    }
}

fn to_static_generics(
    mut generics: syn::Generics,
    ident: &Ident,
    crate_path: &syn::Path,
) -> (syn::Generics, syn::Path) {
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
            #ident <#(#lifetime_iter,)* #(<#type_param_iter as #crate_path::ToStatic>::Static),*>
        }
    } else {
        drop(lifetimes);
        drop(type_params);
        parse_quote! {#ident}
    };

    for typ in generics.type_params_mut() {
        typ.bounds.push(parse_quote!(#crate_path::ToStatic));
    }

    (generics, the_static)
}

fn fields_codegen(
    fields: darling::ast::Fields<ToStaticField>,
    crate_path: &syn::Path,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let field_iter = fields
        .iter()
        .enumerate()
        .map(|(num, ToStaticField { ident, .. })| {
            ident.clone().unwrap_or_else(|| {
                proc_macro2::Ident::new(&format!("_{num}"), proc_macro2::Span::mixed_site())
            })
        });
    let destructuring: proc_macro2::TokenStream =
        field_iter.clone().map(|ident| quote!(#ident,)).collect();
    let to_static: proc_macro2::TokenStream = field_iter
        .clone()
        .map(|ident| quote!(let #ident = #crate_path::ToStatic::to_static(#ident);))
        .collect();
    let into_static: proc_macro2::TokenStream = field_iter
        .map(|ident| quote!(let #ident = #crate_path::ToStatic::into_static(#ident);))
        .collect();
    let destructuring = match fields.style {
        darling::ast::Style::Tuple => quote! {(#destructuring)},
        darling::ast::Style::Struct => quote! {{#destructuring}},
        darling::ast::Style::Unit => quote!(),
    };
    (destructuring, to_static, into_static)
}
