use nom::{
    bytes::complete::{tag, take_while, take_while1},
    combinator::{map_parser, recognize, verify},
    sequence::pair,
    AsChar,
};

use super::NomResult;

pub const KEYWORDS: [&str; 51] = [
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

fn identifier_impl(input: &str) -> NomResult<&str> {
    recognize(pair(
        take_while1(|c: char| c.is_alpha() || c == '_' || c >= '\u{0080}'),
        take_while(|c: char| c.is_alphanum() || c == '_' || c >= '\u{0080}'),
    ))(input)
}

pub fn identifier(input: &str) -> NomResult<&str> {
    verify(identifier_impl, |s| !KEYWORDS.contains(s))(input)
}

pub fn keyword<'a>(kw: &'a str) -> impl FnMut(&'a str) -> NomResult<&str> {
    map_parser(identifier_impl, tag(kw))
}
