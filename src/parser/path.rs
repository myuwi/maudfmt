use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0},
    combinator::{opt, recognize},
    multi::many0,
    sequence::{pair, preceded, terminated, tuple},
};

use super::{combinator::ws, ident::identifier, NomResult};

pub fn path_expression(input: &str) -> NomResult<&str> {
    // TODO: qualitied path expression
    path_in_expression(input)
}

pub fn path_in_expression(input: &str) -> NomResult<&str> {
    recognize(tuple((
        opt(terminated(tag("::"), multispace0)),
        path_expr_segment,
        many0(preceded(ws(tag("::")), path_expr_segment)),
    )))(input)
}

pub fn path_expr_segment(input: &str) -> NomResult<&str> {
    recognize(pair(
        path_ident_segment,
        opt(preceded(ws(tag("::")), generic_args)),
    ))(input)
}

pub fn path_ident_segment(input: &str) -> NomResult<&str> {
    alt((identifier, tag("$crate")))(input)
}

// TODO: implement generics
pub fn generic_args(input: &str) -> NomResult<&str> {
    recognize(tuple((char('<'), multispace0, char('>'))))(input)
}
