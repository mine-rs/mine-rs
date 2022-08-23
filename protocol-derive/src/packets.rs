use std::collections::BTreeMap;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TS};
use proc_macro2_diagnostics::{Diagnostic, Level};
use quote::ToTokens;
use quote::{quote, quote_spanned, spanned::Spanned, TokenStreamExt};
use syn::{braced, Token, TypePath};
use syn::{punctuated::Punctuated, token::Brace, Expr, ExprLit, Lit, LitInt, Pat, PatLit, PatOr};

pub fn packets(x: PacketsInput) -> TokenStream {
    let mut ret = TS::new();

    let mut ver_id_mappings = IdPvMappings::default();

    let mut all_packets: BTreeMap<Ident, TypePath> = BTreeMap::default();

    let mut id_match_body = TS::new();

    for id in x.id_mappings {
        let litid = id.id;
        let (idi32, id_span) = match litint2i32(litid.clone()) {
            Ok(k) => k,
            Err(e) => {
                ret.extend(e);
                break;
            }
        };

        let mut pv_match_body = TS::new();

        for ver in id.version_mappings {
            let versions: Vec<_> = match ver
                .pat
                .clone()
                .into_iter()
                .map(pat2veci32)
                .collect::<Result<Vec<_>, _>>()
            {
                Ok(k) => k.into_iter().flat_map(|x| x.into_iter()).collect(),
                Err(e) => {
                    ret.extend(e);
                    break;
                }
            };
            for (pv_lo, pv_hi, pv_span) in versions {
                let def = Def {
                    ident: ver.packet.clone(),
                    id: idi32,
                    id_span,
                    pv_lo,
                    pv_hi,
                    pv_span,
                };
                if let Some(err) = ver_id_mappings.insert(def) {
                    ret.extend(err.emit_as_expr_tokens())
                }
            }

            let mut ver_pat = TS::new();

            ver_pat.append_separated(ver.pat, quote!(|));

            let packet = ver.packet;
            let last_ident = packet
                .path
                .segments
                .last()
                .expect("path without last segment?")
                .ident
                .clone();
            all_packets.insert(last_ident, packet.clone());

            eprintln!("{}", (&packet).into_token_stream());

            pv_match_body.extend(quote!(
                #ver_pat => {
                    protocol_derive::replace! {
                        #packet ;
                        #($($t)*)
                    }
                }
            ))
        }

        pv_match_body.extend(quote!(_ => $($e)*,));

        id_match_body.extend(quote!(#litid => match $pv {#pv_match_body},));
    }

    id_match_body.extend(quote!(_ => $($e)*,));

    let mut packets_body = TS::new();
    let mut custom_body: TS = TS::new();

    for (ident, packet) in &all_packets {
        packets_body.extend(quote!(#ident(#packet),));
        custom_body.extend(quote!(#packet));
    }

    // ret.extend(quote!( enum Packet { #packets_body } ));

    // all_packets.into_iter().map(|a|a.to_tokens(&mut custom_body));

    ret.extend(quote!(
        #[allow(unused)]
        macro_rules! custom {
        ($($t:tt)*) => {
            protocol_derive::replace!{
                #custom_body ;
                $($t)*
            }
        }
    }
    ));

    // ret.extend(
    //     quote!(fn packet<'a>(pv: i32, id: i32, data: &'a [u8]) -> Result<Packet, ReadError> {
    //         let mut cursor = ::std::io::Cursor::new(data);
    //         match id {
    //             #id_match_body
    //         }
    //     }),
    // );

    ret.extend(quote! {
        macro_rules! tree {
            ($id:ident, $pv:ident, {$($t:tt)*}, {$($e:tt)*}) => {
                match $id {
                    #id_match_body
                }
            }
        }
    });

    ret.into()
}

pub struct PacketsInput {
    id_mappings: Punctuated<IdMapping, Token![,]>,
}

impl syn::parse::Parse for PacketsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(PacketsInput {
            id_mappings: input.parse_terminated(IdMapping::parse)?,
        })
    }
}

pub struct IdMapping {
    id: syn::LitInt,
    _arrow: Token![=>],
    _brace: Brace,
    version_mappings: Punctuated<VersionMapping, Token![,]>,
}
impl syn::parse::Parse for IdMapping {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(IdMapping {
            id: input.parse()?,
            _arrow: input.parse()?,
            _brace: braced!(content in input),
            version_mappings: content.parse_terminated(VersionMapping::parse)?,
        })
    }
}

pub struct VersionMapping {
    pat: Vec<syn::Pat>,
    _arrow: Token![=>],
    pub packet: TypePath,
}

impl syn::parse::Parse for VersionMapping {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut pat = vec![];
        loop {
            pat.push(input.parse()?);
            if !input.lookahead1().peek(Token![|]) {
                break;
            }
            let _: Token![|] = input.parse()?;
        }
        Ok(VersionMapping {
            pat,
            _arrow: input.parse()?,
            packet: input.parse()?,
        })
    }
}

fn litint2i32(litint: LitInt) -> Result<(i32, Span), TS> {
    <i32 as std::str::FromStr>::from_str(litint.base10_digits())
        .map_err(|_| quote_spanned!(litint.span()=> compile_error!("tf are you doing1")))
        .map(|i| (i, litint.span()))
}

fn expr2i32(e: Expr) -> Result<(i32, Span), TS> {
    match e {
        Expr::Lit(ExprLit {
            lit: Lit::Int(litint),
            ..
        }) => litint2i32(litint),
        _ => Err(quote!(compile_error!("tf are you doing2"))),
    }
}

#[derive(Default)]
pub struct IdPvMappings {
    stuff: Vec<Def>,
}
fn mk_overlap_err(a: &Def, b: &Def, id: i32, lo: i32, hi: i32) -> Diagnostic {
    Diagnostic::spanned(
        &[a.ident.__span(), b.ident.__span()][..],
        Level::Error,
        &format!("Two Packets defined for id {id} and versions {lo}..={hi}"),
    )
    .span_help(&[a.id_span, a.pv_span][..], "first one defined here")
    .span_help(&[b.id_span, b.pv_span][..], "second one defined here")
}
impl IdPvMappings {
    fn insert(&mut self, x: Def) -> Option<Diagnostic> {
        let ret = self
            .has(x.id, x.pv_lo, x.pv_hi)
            .map(|(a, id, lo, hi)| mk_overlap_err(a, &x, id, lo, hi));
        self.stuff.push(x);
        ret
    }
    fn has(&self, id: i32, pv_lo: i32, pv_hi: i32) -> Option<(&Def, i32, i32, i32)> {
        for x in &self.stuff {
            if x.id == id && !(x.pv_lo < pv_hi && pv_lo > x.pv_hi) {
                let lo = x.pv_lo.max(pv_lo);
                let hi = x.pv_hi.min(pv_hi);
                return Some((x, id, lo, hi));
            }
        }
        None
    }
}
pub struct Def {
    pub ident: TypePath,
    pub id: i32,
    pub id_span: Span,
    pub pv_lo: i32,
    pub pv_hi: i32,
    pub pv_span: Span,
}

fn pat2veci32(pat: Pat) -> Result<Vec<(i32, i32, Span)>, TS> {
    Ok(match pat {
        syn::Pat::Lit(PatLit { expr, .. }) => match *expr {
            Expr::Lit(ExprLit {
                lit: Lit::Int(int), ..
            }) => {
                let (i, span) = litint2i32(int)?;
                vec![(i, i, span)]
            }
            _ => return Err(quote!(compile_error!("tf are you doing3"))),
        },
        syn::Pat::Or(PatOr { cases, .. }) => cases
            .into_iter()
            .map(pat2veci32)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect(),
        syn::Pat::Range(range) => {
            let span = range.__span();
            let (lo, _) = expr2i32(*range.lo)?;
            let (hi, _) = expr2i32(*range.hi)?;
            match range.limits {
                syn::RangeLimits::HalfOpen(_) => vec![(lo, hi + 1, span)],
                syn::RangeLimits::Closed(_) => vec![(lo, hi, span)],
            }
        }
        syn::Pat::Wild(_) => return Err(quote!(compile_error!("be explicit please"))),
        _ => return Err(quote!(compile_error!("tf are you doing4"))),
    })
}
