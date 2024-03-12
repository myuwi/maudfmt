use nom::{
    bytes::complete::{tag, take_while, take_while1},
    combinator::{map_parser, recognize},
    sequence::pair,
    AsChar,
};

use super::NomResult;

pub fn keyword<'a>(kw: &'a str) -> impl FnMut(&'a str) -> NomResult<&str> {
    map_parser(identifier, tag(kw))
}

pub fn identifier(input: &str) -> NomResult<&str> {
    recognize(pair(
        take_while1(|c: char| c.is_alpha() || c == '_' || c >= '\u{0080}'),
        take_while(|c: char| c.is_alphanum() || c == '_' || c >= '\u{0080}'),
    ))(input)
}
