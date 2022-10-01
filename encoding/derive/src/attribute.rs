use proc_macro2::{Ident, Span, TokenStream};
use quote::{spanned::Spanned, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse2, parse_quote, Expr, LitInt, Token, Type, TypePath,
};

pub struct Attribute {
    pub span: Span,
    pub data: AttributeData,
}

#[allow(clippy::large_enum_variant)]
pub enum AttributeData {
    VarInt,
    Case(Expr),
    From(Type),
    Fixed(Fixed),
    Counted(TypePath),
    Mutf8,
    Rest,
    StringUuid,
    BitField(BitField),
    Bits(u8),
    Bool,
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
#[derive(Default)]
pub struct BitField(pub Option<Type>, pub bool);
impl Parse for BitField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(input
            .parse()
            .map(|ty| {
                BitField(Some(ty), {
                    <Token![,]>::parse(input)
                        .and_then(|_| Ident::parse(input))
                        .map(|ident| ident == "reverse")
                        .unwrap_or_default()
                })
            })
            .unwrap_or_default())
    }
}
pub struct Bits(u8);
impl Parse for Bits {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let litint: LitInt = input.parse()?;
        Ok(Bits(litint.base10_parse()?))
    }
}

pub fn parse_attr(attr: syn::Attribute) -> Option<Result<Attribute, syn::Error>> {
    let span = attr.__span().resolved_at(Span::call_site());
    let tokens = attr.tokens;
    struct UnParen<T>(T);
    impl<T: Parse> Parse for UnParen<T> {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let rest;
            parenthesized!(rest in input);
            let inner: T = rest.parse()?;
            Ok(UnParen(inner))
        }
    }
    match attr.path.get_ident() {
        Some(ident) if ident == "varint" => Some(Ok(Attribute {
            span: ident.span().resolved_at(Span::call_site()),
            data: AttributeData::VarInt,
        })),
        Some(ident) if ident == "stringuuid" => Some(Ok(Attribute {
            span: ident.span().resolved_at(Span::call_site()),
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
        Some(ident) if ident == "counted" => Some({
            parse2::<UnParen<TypePath>>(tokens).map(|a| Attribute {
                span,
                data: AttributeData::Counted(a.0),
            })
        }),
        Some(ident) if ident == "mutf8" => Some({
            Ok(Attribute {
                span,
                data: AttributeData::Mutf8,
            })
        }),
        Some(ident) if ident == "rest" => Some(Ok(Attribute {
            span: ident.span().resolved_at(Span::call_site()),
            data: AttributeData::Rest,
        })),
        Some(ident) if ident == "bitfield" => Some({
            let data = AttributeData::BitField(
                parse2::<UnParen<BitField>>(tokens)
                    .map(|a| a.0)
                    .unwrap_or(BitField(None, false)),
            );
            Ok(Attribute { span, data })
        }),
        Some(ident) if ident == "bits" => Some({
            parse2::<UnParen<Bits>>(tokens).map(|a| Attribute {
                span: ident.span().resolved_at(Span::call_site()),
                data: AttributeData::Bits(a.0 .0),
            })
        }),
        Some(ident) if ident == "bool" => Some(Ok(Attribute {
            span,
            data: AttributeData::Bool,
        })),
        _ => None,
    }
}

#[derive(Clone)]
pub enum Attrs {
    None,
    Var(Span),
    Fixed(Span, Fixed),
    StringUuid(Span),
    Counted(Span, TypePath),
    Mutf8(Span),
    Rest(Span),
}

pub fn field_attrs(attrs: impl Iterator<Item = syn::Attribute>, res: &mut TokenStream) -> Attrs {
    let mut varint = None;
    let mut fixed = None;
    let mut stringuuid = None;
    let mut counted = None;
    let mut mutf8 = None;
    let mut rest = None;

    for attr_res in attrs.map(parse_attr) {
        let Attribute { span, data } = match attr_res {
            Some(Ok(attr)) => attr,
            Some(Err(e)) => {
                e.into_compile_error().to_tokens(res);
                continue;
            }
            None => continue,
        };
        use crate::attribute::AttributeData::*;
        let err_message = match data {
            VarInt => {
                if varint.is_none() {
                    varint = Some(span);
                    continue;
                } else {
                    "duplicate `#[varint]` attribute on field"
                }
            }
            Case(_) => "`#[case(id)]` not allowed on field",
            From(_) => continue,
            Fixed(fixd) => {
                if fixed.is_none() {
                    fixed = Some((span, fixd));
                    continue;
                } else {
                    "duplicate `#[fixed(prec, ty)]` attribute on field"
                }
            }
            Counted(ty) => {
                if counted.is_none() {
                    if rest.is_some() {
                        "`#[counted]` and `#[rest]` are mutually exclusive"
                    } else {
                        counted = Some((span, ty));
                        continue;
                    }
                } else {
                    "duplicate `#[counted(ty)]` attribute on field"
                }
            }
            Mutf8 => {
                if mutf8.is_none() {
                    mutf8 = Some(span);
                    continue;
                } else {
                    "duplicate `#[mutf8]` attribute on field"
                }
            }
            Rest => {
                if rest.is_none() {
                    if counted.is_some() {
                        "`#[counted]` and `#[rest]` are mutually exclusive"
                    } else {
                        rest = Some(span);
                        continue;
                    }
                } else {
                    "duplicate `#[rest(ty)]` attribute on field"
                }
            }
            StringUuid => {
                if stringuuid.is_none() {
                    stringuuid = Some(span);
                    continue;
                } else {
                    "duplicate `#[stringuuid]` attribute on field"
                }
            }
            BitField(_) => "`#[bitfield]` not allowed on field",
            Bits(_) => {
                "`#[bits(size)]` not allowed on field without `#[bitfield]` on struct declaration"
            }
            Bool => "`#[bool]` not allowed on field without `#[bitfield]` on struct declaration",
        };
        error!(span, err_message).to_tokens(res)
    }
    match (varint, fixed, stringuuid, counted, mutf8, rest) {
        (None, None, None, None, None, None) => Attrs::None,
        (None, None, None, None, None, Some(rs)) => Attrs::Rest(rs),
        (None, None, None, None, Some(ms), a) => {
            if a.is_some() {
                error!(
                    ms,
                    "`#[mutf8]` incompatible with other annotated attribute(s)"
                )
                .to_tokens(res);
            }
            Attrs::Mutf8(ms)
        }
        (None, None, None, Some((cs, c)), a, b) => {
            if a.is_some() || b.is_some() {
                error!(
                    cs,
                    "`#[counted(ty)]` incompatible with other annotated attribute(s)"
                )
                .to_tokens(res);
            }
            Attrs::Counted(cs, c)
        }
        (None, None, Some(s), a, b, c) => {
            if a.is_some() || b.is_some() || c.is_some() {
                error!(
                    s,
                    "`#[stringuuid]` incompatible with other annotated attribute(s)"
                )
                .to_tokens(res);
            }
            Attrs::StringUuid(s)
        }
        (None, Some((fs, f)), a, b, c, d) => {
            if a.is_some() || b.is_some() || c.is_some() || d.is_some() {
                error!(
                    fs,
                    "`#[fixed(prec, ty)]` incompatible with other annotated attribute(s)"
                )
                .to_tokens(res);
            }
            Attrs::Fixed(fs, f)
        }
        (Some(v), a, b, c, d, e) => {
            if a.is_some() || b.is_some() || c.is_some() || d.is_some() || e.is_some() {
                error!(
                    v,
                    "`#[varint]` incompatible with other annotated attribute(s)"
                )
                .to_tokens(res);
            }
            Attrs::Var(v)
        }
    }
}
