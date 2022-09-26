use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{Token, TypePath};

pub struct ReplaceInput {
    pub types: Vec<(Option<Token![$]>, TypePath)>,
    pub rest: TokenStream,
}

impl syn::parse::Parse for ReplaceInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut types = vec![];
        let mut dollar = None;
        while !input.lookahead1().peek(Token![;]) {
            if dollar.is_none() && input.peek(Token![$]) {
                dollar = Some(input.parse()?);
                continue;
            }
            types.push((dollar, input.parse()?));
            dollar = None;
        }
        if dollar.is_some() {
            panic!("no lone `$` allowed")
        }
        let _: Token![;] = input.parse()?;
        let rest = input.parse()?;
        Ok(ReplaceInput { types, rest })
    }
}

pub fn match_group(
    mut iter: impl Iterator<Item = TokenTree>,
    ts: &mut TokenStream,
    packets: &Vec<(Option<Token![$]>, TypePath)>,
) {
    while let Some(t) = iter.next() {
        match t {
            TokenTree::Punct(p) if p.as_char() == '#' => match iter.next() {
                Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                    replace_group(g.stream().into_iter(), ts, packets)
                }
                Some(other) => {
                    p.to_tokens(ts);
                    other.to_tokens(ts);
                }
                None => {
                    p.to_tokens(ts);
                }
            },
            TokenTree::Group(g) => {
                let mut acc = TokenStream::new();
                match_group(g.stream().into_iter(), &mut acc, packets);
                Group::new(g.delimiter(), acc.into_iter().collect()).to_tokens(ts);
            }
            other => other.to_tokens(ts),
        }
    }
}
pub enum X {
    ReplaceWithPacket(Ident, Case),
    ReplaceWithPath,
    ReplaceWithPathWithLifetimes,
    Group(Delimiter, Vec<X>),
    Keep(TokenTree),
}
#[allow(non_camel_case_types)]
pub enum Case {
    snake_case,
    camelCase,
    PascalCase,
    SCREAMING_SNAKE_CASE,
}
impl Case {
    fn ident(&self, name: &str, span: Span) -> Ident {
        use convert_case::Casing;
        let name = match self {
            Case::snake_case => name.to_string().to_case(convert_case::Case::Snake),
            Case::camelCase => name.to_case(convert_case::Case::Camel),
            Case::PascalCase => name.to_case(convert_case::Case::Pascal),
            Case::SCREAMING_SNAKE_CASE => name.to_case(convert_case::Case::ScreamingSnake),
        };
        Ident::new(&name, span)
    }
}

fn tttox(mut iter: impl Iterator<Item = TokenTree>, output: &mut Vec<X>) {
    while let Some(t) = iter.next() {
        match t {
            TokenTree::Punct(p) if p.as_char() == '#' => match iter.next() {
                Some(TokenTree::Ident(id)) if id == "packet_name" => {
                    output.push(X::ReplaceWithPacket(id, Case::snake_case))
                }
                Some(TokenTree::Ident(id)) if id == "packetName" => {
                    output.push(X::ReplaceWithPacket(id, Case::camelCase))
                }
                Some(TokenTree::Ident(id)) if id == "PacketName" => {
                    output.push(X::ReplaceWithPacket(id, Case::PascalCase))
                }
                Some(TokenTree::Ident(id)) if id == "PACKET_NAME" => {
                    output.push(X::ReplaceWithPacket(id, Case::SCREAMING_SNAKE_CASE))
                }
                Some(TokenTree::Ident(id)) if id == "PacketType" => output.push(X::ReplaceWithPath),
                Some(TokenTree::Ident(id)) if id == "PacketTypeLt" => {
                    output.push(X::ReplaceWithPathWithLifetimes)
                }
                Some(TokenTree::Group(g)) => {
                    let mut x = vec![];
                    tttox(g.stream().into_iter(), &mut x);
                    output.push(X::Group(g.delimiter(), x));
                }
                Some(other) => {
                    output.push(X::Keep(TokenTree::Punct(p)));
                    output.push(X::Keep(other));
                }
                None => output.push(X::Keep(TokenTree::Punct(p))),
            },
            TokenTree::Group(g) => {
                let mut x = vec![];
                tttox(g.stream().into_iter(), &mut x);
                output.push(X::Group(g.delimiter(), x));
            }
            other => output.push(X::Keep(other)),
        }
    }
}

fn replace_group(
    g: impl Iterator<Item = TokenTree>,
    output: &mut TokenStream,
    packets: &Vec<(Option<Token![$]>, TypePath)>,
) {
    let mut innerout = vec![];
    tttox(g, &mut innerout);
    for (dollar, packet) in packets {
        fn replace(x: &X, dollar: Option<&Token![$]>, packet: &TypePath, ts: &mut TokenStream) {
            match x {
                X::ReplaceWithPacket(id, case) => {
                    let ident_name = packet
                        .path
                        .segments
                        .last()
                        .expect("no last segment?")
                        .ident
                        .to_string();
                    case.ident(ident_name.as_str(), id.span()).to_tokens(ts);
                }
                X::ReplaceWithPath => {
                    if let Some(dollar) = dollar {
                        dollar.to_tokens(ts);
                    }
                    let mut packet = packet.clone();
                    for segm in packet.path.segments.iter_mut() {
                        segm.arguments = syn::PathArguments::None;
                    }
                    packet.path.to_tokens(ts);
                }
                X::ReplaceWithPathWithLifetimes => {
                    if let Some(dollar) = dollar {
                        dollar.to_tokens(ts);
                    }
                    packet.path.to_tokens(ts);
                }
                X::Group(delim, innerer) => {
                    let mut g = TokenStream::new();
                    for x in innerer {
                        replace(x, dollar, packet, &mut g);
                    }
                    Group::new(*delim, g).to_tokens(ts);
                }
                X::Keep(k) => k.to_tokens(ts),
            }
        }
        for x in &innerout {
            replace(x, dollar.as_ref(), packet, output)
        }
    }
}
