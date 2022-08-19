use proc_macro2::{Ident, Span};
use quote::spanned::Spanned;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_quote, Expr, Type,
};

// #[derive(Clone)]
pub struct Attribute {
    pub span: Span,
    pub data: AttributeData,
}
// #[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum AttributeData {
    VarInt,
    Case(Expr),
    From(Type),
}

impl TryFrom<syn::Attribute> for Attribute {
    type Error = ();
    fn try_from(attr: syn::Attribute) -> Result<Self, Self::Error> {
        let tokens = &attr.tokens;
        struct UnParen<T>(T);
        impl<T: Parse> Parse for UnParen<T> {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let inner;
                parenthesized!(inner in input);
                Ok(UnParen(inner.parse()?))
            }
        }

        match attr.path.get_ident() {
            Some(ident) if ident == "varint" => Ok(Attribute {
                span: ident.span(),
                data: AttributeData::VarInt,
            }),
            Some(ident) if ident == "case" => Ok({
                let expr: UnParen<_> = parse_quote!(#tokens);
                Attribute {
                    span: attr.__span(),
                    data: AttributeData::Case(expr.0),
                }
            }),
            Some(ident) if ident == "from" => Ok({
                let typ: UnParen<_> = parse_quote!(#tokens);
                Attribute {
                    span: attr.__span(),
                    data: AttributeData::From(typ.0),
                }
            }),
            _ => Err(()),
        }
    }
}
