use darling::{
    util::{Flag, SpannedValue},
    FromDeriveInput, FromField, FromVariant, ToTokens,
};
use syn::{parse_macro_input, parse_quote, DeriveInput};

#[macro_use]
extern crate quote;
extern crate proc_macro;

fn default_crate_path() -> syn::Path {
    parse_quote!(::miners_encoding)
}
const NEITHER_CASE_NOR_DISCRIMINANT: &str = "neither #[encoding(case = ...)] nor discriminant specified with no previous case to increment from, going on assuming 0";

#[derive(FromDeriveInput)]
#[darling(attributes(encoding))]
struct EncodingInput {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<EncodingVariant, EncodingField>,

    // flags
    /// whether or not to encode the id as a varint
    varint: Flag,
    /// which type the id should be encoded as
    from: Option<syn::Type>,
    /// crate path for reexporting
    crate_path: Option<syn::Path>,
}

#[derive(FromVariant)]
#[darling(attributes(encoding))]
struct EncodingVariant {
    ident: syn::Ident,
    discriminant: Option<syn::Expr>,
    fields: darling::ast::Fields<EncodingField>,

    // flags
    /// replacement for discriminant where not usable (nightly syntax) or not desired
    case: Option<syn::Expr>,
}

#[derive(FromField)]
#[darling(attributes(encoding))]
struct EncodingField {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    // flags
    /// (de)serialize as varint
    varint: Flag,
    /// precision
    #[darling(default)]
    fixed: SpannedValue<Option<attribute::Fixed>>,
    /// parse string as uuid
    stringuuid: Flag,
    /// count elements using given type (prepended to arrays)
    #[darling(default)]
    counted: SpannedValue<Option<syn::Type>>,
    /// parse mutf8 into string or other compatible type
    mutf8: Flag,
    /// try to parse the rest of the bytes
    /// - must be in a top level struct, else no bytes remain for the rest
    /// - must be the last field in a struct
    rest: Flag,
}

#[proc_macro_derive(Encoding, attributes(encoding))]
pub fn encoding(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let deriveinput = parse_macro_input!(input as DeriveInput);
    let EncodingInput {
        ident,
        generics,
        data,
        varint,
        from,
        crate_path,
    } = match EncodingInput::from_derive_input(&deriveinput) {
        Ok(enc_input) => enc_input,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    let crate_path = crate_path.unwrap_or_else(default_crate_path);

    match match data {
        darling::ast::Data::Enum(variants) => {
            r#enum::enum_from_variants(variants, generics, ident, varint, from, &crate_path)
        }
        darling::ast::Data::Struct(fields) => {
            fields::codegen(fields, &crate_path).map(|fieldscode| {
                r#struct::struct_from_fieldscode(fieldscode, generics, ident, &crate_path)
            })
        }
    } {
        Ok(k) => k.into(),
        Err(e) => e.write_errors().into(),
    }
}

#[derive(FromDeriveInput)]
#[darling(
    attributes(encoding),
    supports(struct_named, struct_newtype, struct_tuple)
)]
struct BitfieldInput {
    ident: syn::Ident,
    #[allow(dead_code)]
    generics: syn::Generics,
    data: darling::ast::Data<EncodingVariant, BitfieldField>,

    // flags
    /// explicitly use a type of a certain size
    typ: Option<syn::Type>,
    /// whether to encode the bits in reverse order
    reverse: Flag,
    /// crate path for reexporting
    crate_path: Option<syn::Path>,
}

#[derive(FromVariant)]
#[darling(attributes(encoding))]
struct BitfieldVariant {}

#[derive(FromField)]
#[darling(attributes(encoding))]
struct BitfieldField {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    // flags
    #[darling(default)]
    bits: SpannedValue<Option<u8>>,
    bool: Flag,
}

#[proc_macro_derive(Bitfield, attributes(encoding))]
pub fn bitfield(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let deriveinput = syn::parse_macro_input!(input as DeriveInput);
    let BitfieldInput {
        ident,
        // todo! generics
        generics: _,
        data,
        typ,
        reverse,
        crate_path,
    } = match BitfieldInput::from_derive_input(&deriveinput) {
        Ok(k) => k,
        Err(e) => return e.write_errors().into(),
    };

    let crate_path = crate_path.unwrap_or_else(default_crate_path);

    let darling::ast::Data::Struct(fields) = data else { panic!("enum not supported") };

    let mut errors = darling::Error::accumulator();

    let mut cumultative_size = 0;

    struct BitField {
        ident: syn::Ident,
        ty: syn::Type,
        // absence of this indicates a bool is being encoded
        explicit_bits: Option<u8>,
    }
    let mut bitfields: Vec<BitField> = vec![];
    let mut destructuring = quote! {};

    for (i, field) in fields.fields.into_iter().enumerate() {
        let explicit_bits = if field.bool.is_present() {
            if field.bits.is_some() {
                let err = darling::Error::custom("bool is incompatible with bits");
                errors.push(err.with_span(&field.bool));
            }
            cumultative_size += 1;
            None
        } else if let Some(amount) = field.bits.as_ref() {
            cumultative_size += amount;
            Some(*amount)
        } else {
            let err = darling::Error::custom("Missing fields `bool` or `bits`.");
            errors.push(err.with_span(&field.ty));
            continue;
        };

        let ident = fields::field_ident(i, field.ident, &field.ty);

        quote! {#ident,}.to_tokens(&mut destructuring);

        bitfields.push(BitField {
            ident,
            ty: field.ty,
            explicit_bits,
        })
    }

    let destructuring = match fields.style {
        darling::ast::Style::Tuple => quote!( ( # destructuring ) ),
        darling::ast::Style::Struct => quote! { { #destructuring } },
        darling::ast::Style::Unit => quote!(),
    };

    let ty = match typ {
        Some(ty) => quote::ToTokens::to_token_stream(&ty),
        None => match cumultative_size {
            0..=8 => quote!(::core::primitive::u8),
            9..=16 => quote!(::core::primitive::u16),
            17..=32 => quote!(::core::primitive::u32),
            33..=64 => quote!(::core::primitive::u64),
            65..=128 => quote!(::core::primitive::u128),
            _ => {
                let err = darling::Error::custom("bitfield size too large, maximum is currently `u128`, rust's largest number type");
                errors.push(err.with_span(&ident));
                quote!(::core::primitive::u128)
            }
        },
    };

    if let Err(e) = errors.finish() {
        return e.write_errors().into();
    }

    /* | ((x as ty & (ty::BITS as ty - size)) << offset) */
    let mut encode = quote! {0};
    let mut decode = quote! {};
    let mut neg_checks = quote! {};

    let mut offset = 0;

    let mut res = quote!();

    for BitField {
        ident,
        ty: field_ty,
        explicit_bits,
    } in bitfields
    {
        let size = explicit_bits.unwrap_or(1);
        let shift = if reverse.is_present() {
            let shift = quote! { (#offset as #ty)};
            offset += size;
            shift
        } else {
            offset += size;
            quote! { (#ty::BITS as #ty - #offset as #ty) }
        };
        if explicit_bits.is_some() {
            quote! {
                | ((*#ident as #ty & (!0 >> (#ty::BITS as #ty - #size as #ty))) << #shift)
            }
            .to_tokens(&mut encode);
            quote!{
                let mut #ident = ((__value >> #shift) & !0 >> (#ty::BITS as #ty - #size as #ty)) as _;
            }.to_tokens(&mut decode);
            quote! {
                if #field_ty::MIN != 0 && #ident >= 1 << (#size as #field_ty - 1) {
                    #ident ^= !0 << #size as #field_ty;
                }
            }
            .to_tokens(&mut neg_checks);
        } else {
            quote! {
                | ((*#ident as #ty & (!0 >> (#ty::BITS as #ty - #size as #ty))) << #shift)
            }
            .to_tokens(&mut encode);
            quote!{
                let mut #ident = ((__value >> #shift) & !0 >> (#ty::BITS as #ty - #size as #ty)) != 0;
            }.to_tokens(&mut decode);
        }
    }

    quote! {
        impl<'dec> #crate_path::Decode<'dec> for #ident {
            fn decode(cursor: &mut ::std::io::Cursor<&'dec [::core::primitive::u8]>) -> #crate_path::decode::Result<Self> {
                let __value = #ty::decode(cursor)?;
                #decode
                #neg_checks
                Ok(Self #destructuring)
            }
        }
        impl #crate_path::Encode for #ident {
            fn encode(&self, writer: &mut impl ::std::io::Write) -> #crate_path::encode::Result<()> {
                let Self #destructuring = self;
                #[allow(clippy::identity_op)]
                #crate_path::Encode::encode(&(#encode), writer)
            }
        }
    }
    .to_tokens(&mut res);

    res.into()
}

// macro_rules! error {
//     ($span:ident, $id:ident) => {
//         syn::Error::new($span, $id)
//             .to_compile_error()
//     };
//     ($span:ident, $lit:literal) => {
//         syn::Error::new($span, $lit)
//             .to_compile_error()
//     };
//     ($span:ident, $($t:tt),+) => {
//         syn::Error::new($span, format!($($t),+))
//             .to_compile_error()
//     };
// }

mod attribute;
mod r#enum;
mod fields;
mod generics;
mod r#struct;
