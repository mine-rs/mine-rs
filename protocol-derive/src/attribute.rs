use proc_macro2::{Ident, Span, TokenStream};
use quote::spanned::Spanned;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse2, parse_quote, Expr, LitInt, Token, Type,
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
    Fixed(Fixed),
}
#[derive(Clone)]
pub struct Fixed {
    pub precision: u8,
    pub typ: Ident,
}
impl Parse for Fixed {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let litint: LitInt = input.parse()?;
        Ok(Fixed {
            precision: litint.base10_parse()?,
            typ: {
                let _: Token![,] = input.parse()?;
                input.parse()?
            },
        })
    }
}

pub fn parse_attr(attr: syn::Attribute) -> Option<Result<Attribute, syn::Error>> {
    let span = attr.__span();
    let tokens = attr.tokens;
    struct UnParen<T>(T);
    impl<T: Parse> Parse for UnParen<T> {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let rest;
            parenthesized!(rest in input);
            // eprintln!("attr: `{}`", (rest.fork().parse::<TokenStream>()?));
            let inner: T = rest.parse()?;
            // eprintln!("{:?}", inner.to_token_stream());
            // eprintln!("attr rest: `{}`", rest.parse::<TokenStream>()?);
            Ok(UnParen(inner))
        }
    }
    match attr.path.get_ident() {
        Some(ident) if ident == "varint" => Some(Ok(Attribute {
            span: ident.span(),
            data: AttributeData::VarInt,
        })),
        Some(ident) if ident == "case" => Some({
            let expr: UnParen<TokenStream> = parse_quote!(#tokens);
            parse2(expr.0).map(|expr| Attribute {
                span,
                data: AttributeData::Case(expr),
            })
        }),
        Some(ident) if ident == "from" => Some({
            let typ: UnParen<TokenStream> = parse_quote!(#tokens);
            parse2(typ.0).map(|from| Attribute {
                span,
                data: AttributeData::From(from),
            })
        }),
        Some(ident) if ident == "fixed" => Some({
            parse2::<UnParen<Fixed>>(tokens).map(|a| Attribute {
                span,
                data: AttributeData::Fixed(a.0),
            })
        }),
        _ => None,
    }
}
