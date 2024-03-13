use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::multispace0,
    combinator::{opt, recognize},
    multi::many0,
    sequence::{pair, preceded, terminated, tuple},
};

use super::{combinator::ws, expr::group, ident::identifier, NomResult};

pub fn simple_path(input: &str) -> NomResult<&str> {
    recognize(tuple((
        opt(terminated(tag("::"), multispace0)),
        simple_path_segment,
        many0(preceded(ws(tag("::")), simple_path_segment)),
    )))(input)
}

fn simple_path_segment(input: &str) -> NomResult<&str> {
    alt((identifier, tag("$crate")))(input)
}

pub fn path_expression(input: &str) -> NomResult<&str> {
    alt((path_in_expression, qualified_path_in_expression))(input)
}

pub fn path_in_expression(input: &str) -> NomResult<&str> {
    recognize(tuple((
        opt(terminated(tag("::"), multispace0)),
        path_expr_segment,
        many0(preceded(ws(tag("::")), path_expr_segment)),
    )))(input)
}

fn path_expr_segment(input: &str) -> NomResult<&str> {
    recognize(pair(
        path_ident_segment,
        opt(preceded(ws(tag("::")), generic_args)),
    ))(input)
}

fn path_ident_segment(input: &str) -> NomResult<&str> {
    alt((identifier, tag("$crate")))(input)
}

fn generic_args(input: &str) -> NomResult<&str> {
    group('<', '>')(input)
}

fn qualified_path_in_expression(input: &str) -> NomResult<&str> {
    recognize(pair(
        qualified_path_type,
        many0(preceded(ws(tag("::")), path_expr_segment)),
    ))(input)
}

fn qualified_path_type(input: &str) -> NomResult<&str> {
    group('<', '>')(input)
}
