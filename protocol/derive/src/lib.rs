use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput};

#[macro_use]
extern crate quote;
extern crate proc_macro;

mod parsing_tree;
mod replace;
mod to_static;

#[proc_macro]
pub fn parsing_tree(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let x = parse_macro_input!(input as parsing_tree::ParsingTreeInput);

    parsing_tree::parsing_tree(x)
}

#[proc_macro]
pub fn replace(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as replace::ReplaceInput);

    let mut output = proc_macro2::TokenStream::new();

    replace::match_group(input.rest.into_iter(), &mut output, &input.types);

    output.into()
}

#[proc_macro_derive(ToStatic)]
pub fn to_static(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let generics = input.generics.clone();
    let ident = input.ident;

    let mut tostaticgenerics = to_static::generics(input.generics);
    let where_clause = tostaticgenerics.where_clause.take();

    match input.data {
        syn::Data::Struct(strukt) => {
            let (destructuring, to_static, into_static) = to_static::fields(strukt.fields);
            quote! {
                impl #generics ToStatic for #ident #generics
                where
                    #where_clause
                {
                    type Static = #ident #tostaticgenerics;
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
                let (destructuring, to_static, into_static) = to_static::fields(variant.fields);
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
                    type Static = #ident #tostaticgenerics;
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
