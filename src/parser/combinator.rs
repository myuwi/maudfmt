use nom::{
    character::complete::multispace0,
    combinator::{cond, map_opt},
    sequence::delimited,
    IResult,
};

pub fn ws<I, O, E, F>(f: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: nom::InputTakeAtPosition,
    <I as nom::InputTakeAtPosition>::Item: nom::AsChar + Clone,
    E: nom::error::ParseError<I>,
    F: nom::Parser<I, O, E>,
{
    delimited(multispace0, f, multispace0)
}

pub fn cond_err<I, O, E, F>(condition: bool, f: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: Clone,
    E: nom::error::ParseError<I>,
    F: nom::Parser<I, O, E>,
{
    map_opt(cond(condition, f), |o| o)
}
