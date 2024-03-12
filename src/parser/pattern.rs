use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0, multispace1},
    combinator::{opt, recognize},
    multi::many0,
    sequence::{pair, preceded, terminated, tuple},
};

use super::{
    combinator::ws,
    expr::group,
    ident::{identifier, keyword},
    literal::{bool_lit, char_lit, float_lit, int_lit, str_lit},
    path::{path_expression, path_in_expression},
    NomResult,
};

// TODO: or-patterns
// TODO: range pattern
pub fn pattern(input: &str) -> NomResult<&str> {
    pattern_without_range(input)
}

pub fn pattern_without_range(input: &str) -> NomResult<&str> {
    alt((
        wildcard_pattern,
        rest_pattern,
        literal_pattern,
        struct_pattern,
        tuple_struct_pattern,
        path_pattern,
        identifier_pattern,
        reference_pattern,
        tuple_pattern,
        grouped_pattern,
        slice_pattern,
    ))(input)
}

// TODO: missing byte and raw variants
fn literal_pattern(input: &str) -> NomResult<&str> {
    recognize(alt((
        char_lit,
        str_lit,
        bool_lit,
        recognize(pair(opt(char('-')), alt((float_lit, int_lit)))),
    )))(input)
}

// TODO: missing (@ PatternNoTopAlt)?
fn identifier_pattern(input: &str) -> NomResult<&str> {
    recognize(pair(
        many0(terminated(
            alt((keyword("ref"), keyword("mut"))),
            multispace1,
        )),
        identifier,
    ))(input)
}

fn wildcard_pattern(input: &str) -> NomResult<&str> {
    tag("_")(input)
}

fn rest_pattern(input: &str) -> NomResult<&str> {
    tag("..")(input)
}

fn reference_pattern(input: &str) -> NomResult<&str> {
    recognize(tuple((
        alt((tag("&&"), tag("&"))),
        ws(opt(keyword("mut"))),
        pattern_without_range,
    )))(input)
}

fn struct_pattern(input: &str) -> NomResult<&str> {
    recognize(pair(
        path_in_expression,
        preceded(multispace0, group('{', '}')),
    ))(input)
}

fn tuple_struct_pattern(input: &str) -> NomResult<&str> {
    recognize(pair(
        path_in_expression,
        preceded(multispace0, group('(', ')')),
    ))(input)
}

fn tuple_pattern(input: &str) -> NomResult<&str> {
    group('(', ')')(input)
}

fn grouped_pattern(input: &str) -> NomResult<&str> {
    group('(', ')')(input)
}

fn slice_pattern(input: &str) -> NomResult<&str> {
    group('[', ']')(input)
}

fn path_pattern(input: &str) -> NomResult<&str> {
    path_expression(input)
}
