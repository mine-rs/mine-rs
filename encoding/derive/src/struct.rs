use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::{parse_quote, DataStruct, Generics};

use crate::{
    attribute::{parse_attr, Attribute},
    fields::{bitfield_codegen, fields_codegen, fields_to_codegen_input, FieldsCode},
    generics::implgenerics,
};

pub fn derive_struct(
    ident: Ident,
    attrs: Vec<syn::Attribute>,
    generics: Generics,
    strukt: DataStruct,
) -> TokenStream {
    let mut res = TokenStream::new();

    let mut bitfield = None;

    for attr_res in attrs.into_iter().flat_map(parse_attr) {
        let Attribute { span, data } = match attr_res {
            Ok(attr) => attr,
            Err(e) => {
                e.into_compile_error().to_tokens(&mut res);
                continue;
            }
        };
        use crate::attribute::AttributeData::*;
        let kind = match data {
            VarInt => "#[varint]",
            Case(_) => "#[case(ty)]",
            From(_) => "#[from(ty)]",
            Fixed(_) => "#[fixed(prec, ty)]",
            Counted(_) => "#[counted(ty)]",
            Rest => "#[rest]",
            StringUuid => "#[stringuuid]",
            BitField(ty) => {
                if bitfield.is_some() {
                    error!(
                        span,
                        "duplicate `#[bitfield]` specified on struct declaration"
                    )
                    .to_tokens(&mut res);
                } else {
                    bitfield = Some((span, ty));
                };
                continue;
            }
            Bits(_) => "#[bits(size)]",
            Bool => "#[bool]",
        };
        error!(span, "`{}` not allowed on struct", kind).to_tokens(&mut res);
    }

    if let Some((span, ty)) = bitfield {
        bitfield_codegen(span, ty, ident, strukt, &mut res);

        return res;
    }

    let FieldsCode {
        parsing,
        destructuring,
        serialization,
        // to_static,
        // into_static,
    } = fields_to_codegen_input(strukt.fields, &mut res)
        .map(fields_codegen)
        .unwrap_or_default();

    let decode_generics = implgenerics(
        generics.clone(),
        &parse_quote!(Decode),
        Some(parse_quote!('dec)),
    );
    let decode_where_clause = &decode_generics.where_clause;
    quote! {
        impl #decode_generics Decode<'dec> for #ident #generics
        #decode_where_clause
        {
            fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
                #parsing
                Ok(Self #destructuring)
            }
        }
    }
    .to_tokens(&mut res);

    let encode_generics = implgenerics(generics.clone(), &parse_quote!(Encode), None);
    quote! {
        impl #encode_generics Encode for #ident #generics {
            fn encode(&self, writer: &mut impl ::std::io::Write) -> encode::Result<()> {
                let Self #destructuring = self;
                #serialization
                Ok(())
            }
        }
    }
    .to_tokens(&mut res);

    res
}
