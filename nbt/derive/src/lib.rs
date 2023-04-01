use darling::{
    util::{Flag, Override},
    FromDeriveInput, FromField, FromVariant, ast::Fields,
};
use ident_case::RenameRule;
use proc_macro::TokenStream;
use quote::quote;

fn default_crate_path() -> syn::Path {
    syn::parse_quote!(::miners_nbt)
}

#[derive(FromDeriveInput)]
#[darling(attributes(nbt), )]
struct NbtInput {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<NbtVariant, NbtField>,

    // flags
    /// crate path for reexporting
    crate_path: Option<syn::Path>,
    /// renames all identifiers according to the rename rule 
    rename_all: RenameRule,
}

#[derive(FromVariant)]
#[darling(attributes(nbt))]
struct NbtVariant {
    ident: syn::Ident,
    discriminant: Option<syn::Expr>,
    fields: Fields<NbtField>,

    // flags
    rename_all: RenameRule,
}
#[derive(FromField)]
#[darling(attributes(nbt))]
struct NbtField {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    // flags
    /// change the name of the field when encoded (takes precedence over rename_all)
    rename: String,
    #[darling(multiple)]
    /// add aliases when parsing
    alias: Vec<String>,
    /// specify either nothing to use `Default::default()` or a path providing a default value
    /// with the method signature Fn() -> T
    default: Option<Override<syn::Path>>,
    /// merge fields from upper container into this one when parsing
    flatten: Flag,
    skip: Flag,
}

#[proc_macro_derive(Nbt)]
pub fn nbt(input: TokenStream) -> TokenStream {
    let deriveinput = syn::parse_macro_input!(input as syn::DeriveInput);

    let NbtInput {
        ident,
        generics,
        data,
        crate_path,
        rename_all,
    } = match NbtInput::from_derive_input(&deriveinput) {
        Ok(k) => k,
        Err(e) => return e.write_errors().into(),
    };

    let crate_path = crate_path.unwrap_or_else(default_crate_path);

    match data {
        darling::ast::Data::Struct(s) => {
            
        }
        darling::ast::Data::Enum(e) => {

        },
    }
}
