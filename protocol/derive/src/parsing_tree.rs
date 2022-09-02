use std::collections::BTreeMap;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TS, TokenTree};
use quote::ToTokens;
use quote::{quote_spanned, spanned::Spanned, TokenStreamExt};
use syn::{braced, Token, TypePath};
use syn::{punctuated::Punctuated, token::Brace, Expr, ExprLit, Lit, LitInt, Pat, PatLit, PatOr};

pub fn parsing_tree(x: ParsingTreeInput) -> TokenStream {
    let mut ret = TS::new();

    let prefix = x.prefix;

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

            let packet = ver.packet;
            let last_ident = packet
                .path
                .segments
                .last()
                .expect("path without last segment?")
                .ident
                .clone();

            for (pv_lo, pv_hi, pv_span) in versions {
                ver_id_mappings.insert(
                    Def {
                        ident: last_ident.clone(),
                        id: idi32,
                        id_span,
                        pv_lo,
                        pv_hi,
                        pv_span,
                    },
                    &mut ret,
                )
            }

            all_packets.insert(last_ident, packet.clone());

            let mut ver_pat = TS::new();

            ver_pat.append_separated(ver.pat, quote!(|));

            pv_match_body.extend(quote!(
                #ver_pat => {
                    replace! {
                        #prefix #packet ;
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
        packets_body.extend(quote!(#ident(#prefix #packet),));
        custom_body.extend(quote!(#prefix #packet));
    }

    let custom = x.custom;

    ret.extend(quote!(
        #[allow(unused)]
        macro_rules! #custom {
            ($($t:tt)*) => {
                replace!{
                    #custom_body ;
                    $($t)*
                }
            }
        }
    ));

    let tree = x.tree;

    ret.extend(quote! {
        macro_rules! #tree {
            ($id:ident, $pv:ident, {$($t:tt)*}, {$($e:tt)*}) => {
                match $id {
                    #id_match_body
                }
            }
        }
    });

    let packet_to_id_mappings: BTreeMap<Ident, _> = all_packets
        .into_iter()
        .map(|(ident, path)| {
            let ids: Vec<(i32, i32, i32)> = ver_id_mappings
                .stuff
                .iter()
                .filter(|a| a.ident == ident)
                .map(|def| (def.id, def.pv_lo, def.pv_hi))
                .collect();
            (ident, (path, ids))
        })
        .collect();

    for (_, (path, mappings)) in packet_to_id_mappings {
        let ltsiter = path
            .path
            .segments
            .iter()
            .filter_map(|segment| match &segment.arguments {
                syn::PathArguments::AngleBracketed(bracketed) => Some(bracketed),
                _ => None,
            })
            .flat_map(|bracketed| {
                bracketed.args.iter().filter_map(|arg| match arg {
                    syn::GenericArgument::Lifetime(lt) => Some(lt),
                    _ => None,
                })
            });
        let mut generics = quote! {};
        let mut where_clause = quote! {};
        for lts in ltsiter {
            quote! {, #lts}.to_tokens(&mut generics);
            quote! {'dec: #lts,}.to_tokens(&mut where_clause);
        }
        let mut match_body = quote!{};
        for (id, lo, hi) in mappings {
            quote!{
                #lo..=#hi => Some(#id),
            }.to_tokens(&mut match_body)
        }
        quote!{_ => None}.to_tokens(&mut match_body);
        quote! {
            impl<'dec #generics> Packet<'dec> for #prefix #path
            where #where_clause {
                fn id_for_version(version: i32) -> Option<i32> {
                    match version {
                        #match_body
                    }
                }
            }
        }
        .to_tokens(&mut ret)
    }

    ret.into()
}

pub struct ParsingTreeInput {
    pub custom: Ident,
    pub tree: Ident,
    pub prefix: TS,
    id_mappings: Punctuated<IdMapping, Token![,]>,
}

impl syn::parse::Parse for ParsingTreeInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let custom = input.parse()?;
        let tree = input.parse()?;
        let mut prefix = TS::new();
        while !input.peek(Token![;]) {
            input.parse::<TokenTree>()?.to_tokens(&mut prefix);
        }
        let _: Token![;] = input.parse()?;
        Ok(ParsingTreeInput {
            custom,
            tree,
            prefix,
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
        .map_err(|_| quote_spanned!(litint.span()=> compile_error!("invalid integer")))
        .map(|i| (i, litint.span()))
}

fn expr2i32(e: Expr) -> Result<(i32, Span), TS> {
    match e {
        Expr::Lit(ExprLit {
            lit: Lit::Int(litint),
            ..
        }) => litint2i32(litint),
        _ => Err(quote!(compile_error!("invalid expression"))),
    }
}

#[derive(Default)]
pub struct IdPvMappings {
    stuff: Vec<Def>,
}
fn mk_overlap_err(_a: &Def, b: &Def, id: i32, lo: i32, hi: i32, ret: &mut TS) {
    let id_span = b.id_span;
    let error = format!("Two Packets defined for id {id} and versions {lo}..={hi}");
    quote_spanned! {id_span=>
        compile_error!(#error);
    }
    .to_tokens(ret);
}
impl IdPvMappings {
    fn insert(&mut self, x: Def, ret: &mut TS) {
        if let Some((a, id, lo, hi)) = self.has(x.id, x.pv_lo, x.pv_hi) {
            mk_overlap_err(a, &x, id, lo, hi, ret)
        }
        self.stuff.push(x);
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
    pub ident: Ident,
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
            _ => return Err(quote!(compile_error!("invalid expression"))),
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
        syn::Pat::Wild(_) => return Err(quote!(compile_error!("please be explicit"))),
        _ => return Err(quote!(compile_error!("invalid pattern"))),
    })
}
