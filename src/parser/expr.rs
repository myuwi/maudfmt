use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, none_of},
    combinator::{not, peek, recognize},
    multi::many0_count,
    sequence::{delimited, terminated},
    InputTake,
};

use super::{
    combinator::cond_err,
    literal::{char_lit, str_lit},
    NomResult,
};

pub fn group<'a>(start_delim: char, end_delim: char) -> impl FnMut(&'a str) -> NomResult<&str> {
    move |i| {
        recognize(delimited(
            char(start_delim),
            expr_impl(true, true),
            char(end_delim),
        ))(i)
    }
}

// TODO: handle comments
fn expr_impl<'a>(eager_brace: bool, nested_expr: bool) -> impl FnMut(&'a str) -> NomResult<&str> {
    move |i| {
        recognize(many0_count(alt((
            str_lit,
            char_lit,
            group('(', ')'),
            cond_err(eager_brace, group('{', '}')),
            group('[', ']'),
            // TODO: probably not the optimal way...
            recognize(none_of("(){}[];=")),
            terminated(tag("="), not(peek(tag(">")))),
            cond_err(nested_expr, alt((tag(";"), tag("=>")))),
        ))))(i)
    }
}

pub fn expr<'a>(eager_brace: bool) -> impl FnMut(&'a str) -> NomResult<&str> {
    move |i| expr_impl(eager_brace, false)(i).map(|(_, o)| i.take_split(o.trim_end().len()))
}
