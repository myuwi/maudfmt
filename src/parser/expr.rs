use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, none_of},
    combinator::{cond, map_opt, recognize},
    multi::many0_count,
    sequence::delimited,
    InputTake,
};

use super::{
    literal::{char_lit, str_lit},
    NomResult,
};

pub fn group<'a>(start_delim: char, end_delim: char) -> impl FnMut(&'a str) -> NomResult<&str> {
    move |i| {
        recognize(delimited(
            char(start_delim),
            expr_impl(true, false),
            char(end_delim),
        ))(i)
    }
}

// TODO: handle comments
fn expr_impl<'a>(eager_brace: bool, break_at_semi: bool) -> impl FnMut(&'a str) -> NomResult<&str> {
    recognize(many0_count(alt((
        str_lit,
        char_lit,
        recognize(none_of("(){}[]<>;")),
        map_opt(cond(!break_at_semi, tag(";")), |o| o),
        group('(', ')'),
        map_opt(cond(eager_brace, group('{', '}')), |o| o),
        group('[', ']'),
        group('<', '>'),
    ))))
}

pub fn expr<'a>(eager_brace: bool) -> impl FnMut(&'a str) -> NomResult<&str> {
    move |i| expr_impl(eager_brace, true)(i).map(|(_, o)| i.take_split(o.trim_end().len()))
}
