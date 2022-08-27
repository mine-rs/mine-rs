use proc_macro2::{Ident, Span, TokenStream};
use quote::{spanned::Spanned, ToTokens};
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
    StringUuid,
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
        Some(ident) if ident == "stringuuid" => Some(Ok(Attribute {
            span: ident.span(),
            data: AttributeData::StringUuid,
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

#[derive(Clone)]
pub enum Attrs {
    None,
    Var(Span),
    Fixed(Span, Fixed),
    FixedVar(Span, Fixed, Span),
    StringUuid(Span),
}

pub fn struct_field(attrs: impl Iterator<Item = syn::Attribute>, res: &mut TokenStream) -> Attrs {
    let mut varint_attr_span = None;
    let mut fixed = None;
    let mut stringuuid = None;
    for attr_res in attrs.map(parse_attr) {
        let Attribute { span, data } = match attr_res {
            Some(Ok(attr)) => attr,
            Some(Err(e)) => {
                e.into_compile_error().to_tokens(res);
                continue;
            }
            None => continue,
        };
        let err_message = match data {
            AttributeData::VarInt => {
                if varint_attr_span.is_none() {
                    varint_attr_span = Some(span);
                    continue;
                } else {
                    "duplicate `varint` attribute on struct field"
                }
            }
            AttributeData::Case(_) => "`case` attribute not allowed to annotate struct field",
            AttributeData::From(_) => continue,
            AttributeData::Fixed(fixd) => {
                if fixed.is_none() {
                    fixed = Some((span, fixd));
                    continue;
                } else {
                    "duplicate `fixed` attribute on struct field"
                }
            }
            AttributeData::StringUuid => {
                if stringuuid.is_none() {
                    stringuuid = Some(span);
                    continue;
                } else {
                    "duplicate `stringuuid` attribute on struct field"
                }
            }
        };
        error!(span, err_message).to_tokens(res)
    }
    match (varint_attr_span, fixed, stringuuid) {
        (None, None, None) => Attrs::None,
        (None, None, Some(s)) => Attrs::StringUuid(s),
        (None, Some((fs, f)), None) => Attrs::Fixed(fs, f),
        (None, Some((fs, f)), Some(s)) => {
            error!(s, "`stringuuid` incompatible with `fixed`").to_tokens(res);
            Attrs::Fixed(fs, f)
        }
        (Some(v), None, None) => Attrs::Var(v),
        (Some(v), None, Some(s)) => {
            error!(s, "`stringuuid` incompatible with `varint`").to_tokens(res);
            Attrs::Var(v)
        }
        (Some(v), Some((fs, f)), None) => Attrs::FixedVar(fs, f, v),
        (Some(v), Some((fs, f)), Some(s)) => {
            error!(s, "`stringuuid` incompatible with `fixed` and `varint`").to_tokens(res);
            Attrs::FixedVar(fs, f, v)
        },
    }
}
