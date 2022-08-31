use super::attribute::{parse_attr, struct_field, Attribute, AttributeData, Attrs};
use super::{field_ident, implgenerics, struct_codegen, tostaticgenerics, Naming, StructCode};

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{parse_quote, DataStruct, Generics, ItemImpl, Type};

pub fn struct_protocol(
    ident: Ident,
    attrs: Vec<syn::Attribute>,
    generics: Generics,
    strukt: DataStruct,
) -> TokenStream {
    let mut res = quote! {};

    let mut fields: Vec<(Attrs, Ident, Type)> = Vec::with_capacity(strukt.fields.len());

    let kind = match &strukt.fields {
        syn::Fields::Named(_) => Naming::Named,
        syn::Fields::Unnamed(_) => Naming::Unnamed,
        syn::Fields::Unit => {
            let span = ident.span();
            return error!(span, "unit structs not supported").into();
        }
    };

    for attr_res in attrs.into_iter().flat_map(parse_attr) {
        let Attribute { span, data } = match attr_res {
            Ok(attr) => attr,
            Err(e) => {
                e.into_compile_error().to_tokens(&mut res);
                continue;
            }
        };
        let kind = match data {
            AttributeData::VarInt => "varint",
            AttributeData::Case(_) => "case",
            AttributeData::From(_) => continue,
            AttributeData::Fixed(_) => "fixed",
            AttributeData::StringUuid => "stringuuid",
            AttributeData::Counted(_) => "count",
        };
        error!(span, "`{}` attribute not allowed to annotate struct", kind).to_tokens(&mut res);
    }

    for (i, field) in strukt.fields.into_iter().enumerate() {
        let attrs = struct_field(field.attrs.into_iter(), &mut res);

        let ident = field_ident(i, field.ident, &field.ty);

        fields.push((attrs, ident, field.ty));
    }

    let StructCode {
        parsing,
        destructuring,
        serialization,
        to_static,
        into_static,
    } = struct_codegen(kind, fields);

    let staticgenerics = tostaticgenerics(generics.clone());
    let to_static: ItemImpl = parse_quote! {
        impl #generics ToStatic for #ident #generics {
            type Static = #ident #staticgenerics;
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
    };
    to_static.to_tokens(&mut res);

    let implgenerics = implgenerics(
        generics.clone(),
        &parse_quote!(Decode),
        Some(parse_quote!('read)),
    );
    let where_clause = &implgenerics.where_clause;

    let read: ItemImpl = parse_quote! {
        impl #implgenerics Decode<'read> for #ident #generics
        #where_clause
        {
            fn decode(cursor: &mut std::io::Cursor<&'read [u8]>) -> decode::Result<Self> {
                #parsing
                Ok(Self #destructuring)
            }
        }
    };
    read.to_tokens(&mut res);

    let write: ItemImpl = parse_quote! {
        impl #implgenerics Encode for #ident #generics {
            fn encode(&self, writer: &mut impl ::std::io::Write) -> encode::Result<()> {
                let Self #destructuring = self;
                #serialization
                Ok(())
            }
        }
    };
    write.to_tokens(&mut res);

    // panic!();
    res.into()
}
