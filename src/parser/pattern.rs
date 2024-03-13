use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0},
    combinator::{opt, recognize},
    multi::many0,
    sequence::{pair, preceded, separated_pair, terminated, tuple},
};

use super::{
    combinator::ws,
    expr::group,
    ident::{identifier, keyword},
    literal::{bool_lit, byte_lit, byte_str_lit, char_lit, float_lit, int_lit, str_lit},
    path::{path_expression, path_in_expression, simple_path},
    NomResult,
};

pub fn pattern(input: &str) -> NomResult<&str> {
    recognize(tuple((
        opt(terminated(char('|'), multispace0)),
        pattern_no_top_alt,
        many0(pair(ws(char('|')), pattern_no_top_alt)),
    )))(input)
}

fn pattern_no_top_alt(input: &str) -> NomResult<&str> {
    alt((range_pattern, pattern_without_range))(input)
}

fn range_pattern(input: &str) -> NomResult<&str> {
    recognize(tuple((
        opt(terminated(range_pattern_bound, multispace0)),
        tag(".."),
        opt(tag("=")),
        opt(preceded(multispace0, range_pattern_bound)),
    )))(input)
}

fn range_pattern_bound(input: &str) -> NomResult<&str> {
    recognize(alt((
        char_lit,
        byte_lit,
        recognize(pair(opt(char('-')), alt((float_lit, int_lit)))),
        path_expression,
    )))(input)
}

fn pattern_without_range(input: &str) -> NomResult<&str> {
    alt((
        wildcard_pattern,
        rest_pattern,
        literal_pattern,
        struct_pattern,
        tuple_struct_pattern,
        macro_invocation,
        path_pattern,
        identifier_pattern,
        reference_pattern,
        tuple_pattern,
        grouped_pattern,
        slice_pattern,
    ))(input)
}

// TODO: missing raw string variants
fn literal_pattern(input: &str) -> NomResult<&str> {
    recognize(alt((
        char_lit,
        byte_lit,
        str_lit,
        byte_str_lit,
        bool_lit,
        recognize(pair(opt(char('-')), alt((float_lit, int_lit)))),
    )))(input)
}

fn identifier_pattern(input: &str) -> NomResult<&str> {
    recognize(tuple((
        many0(terminated(
            alt((keyword("ref"), keyword("mut"))),
            multispace0,
        )),
        identifier,
        opt(preceded(ws(char('@')), pattern_no_top_alt)),
    )))(input)
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

fn macro_invocation(input: &str) -> NomResult<&str> {
    recognize(separated_pair(
        simple_path,
        ws(char('!')),
        alt((group('(', ')'), group('[', ']'), group('{', '}'))),
    ))(input)
}
