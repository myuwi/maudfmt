use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0, multispace1},
    combinator::{opt, recognize},
    multi::{many1, many_m_n, separated_list1},
    sequence::{delimited, pair, separated_pair, terminated, tuple},
};

use super::{
    combinator::ws,
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
        recognize(literal_pattern),
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
    alt((
        char_lit,
        str_lit,
        bool_lit,
        recognize(pair(opt(char('-')), alt((float_lit, int_lit)))),
    ))(input)
}

// TODO: missing (@ PatternNoTopAlt)?
fn identifier_pattern(input: &str) -> NomResult<&str> {
    recognize(pair(
        many_m_n(0, 2, pair(alt((tag("ref"), tag("mut"))), multispace1)),
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
        terminated(alt((tag("&&"), tag("&"))), multispace0),
        opt(keyword("mut")),
        ws(pattern_without_range),
    )))(input)
}

fn struct_pattern(input: &str) -> NomResult<&str> {
    recognize(pair(
        path_in_expression,
        ws(delimited(
            char('{'),
            ws(opt(struct_pattern_elements)),
            ws(char('}')),
        )),
    ))(input)
}

fn struct_pattern_elements(input: &str) -> NomResult<&str> {
    alt((
        recognize(pair(
            struct_pattern_fields,
            opt(pair(char(','), opt(struct_pattern_et_cetera))),
        )),
        struct_pattern_et_cetera,
    ))(input)
}

fn struct_pattern_fields(input: &str) -> NomResult<&str> {
    // TODO: OuterAttribute
    recognize(separated_list1(
        tag(","),
        alt((
            recognize(separated_pair(
                ws(alt((int_lit, identifier))),
                ws(char(':')),
                ws(pattern),
            )),
            ws(identifier_pattern),
        )),
    ))(input)
}

fn struct_pattern_et_cetera(input: &str) -> NomResult<&str> {
    // TODO: OuterAttribute
    tag("..")(input)
}

fn tuple_struct_pattern(input: &str) -> NomResult<&str> {
    recognize(pair(
        path_in_expression,
        ws(delimited(
            char('('),
            ws(opt(tuple_struct_pattern_items)),
            char(')'),
        )),
    ))(input)
}

fn tuple_struct_pattern_items(input: &str) -> NomResult<&str> {
    recognize(pair(
        separated_list1(tag(","), ws(pattern)),
        ws(opt(tag(","))),
    ))(input)
}

fn tuple_pattern(input: &str) -> NomResult<&str> {
    recognize(delimited(
        char('('),
        ws(opt(tuple_pattern_items)),
        ws(char(')')),
    ))(input)
}

fn tuple_pattern_items(input: &str) -> NomResult<&str> {
    alt((
        rest_pattern,
        recognize(tuple((
            pattern,
            many1(pair(ws(tag(",")), ws(pattern))),
            opt(ws(tag(","))),
        ))),
        recognize(pair(pattern, ws(tag(",")))),
    ))(input)
}

fn grouped_pattern(input: &str) -> NomResult<&str> {
    recognize(delimited(char('('), ws(pattern), char(')')))(input)
}

fn slice_pattern(input: &str) -> NomResult<&str> {
    recognize(delimited(
        char('['),
        ws(opt(tuple_struct_pattern_items)),
        char(']'),
    ))(input)
}

fn path_pattern(input: &str) -> NomResult<&str> {
    path_expression(input)
}
