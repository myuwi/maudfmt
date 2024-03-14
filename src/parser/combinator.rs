use nom::{character::complete::multispace0, sequence::delimited};

pub fn ws<I, O, E, F>(f: F) -> impl nom::Parser<I, O, E>
where
    I: nom::InputTakeAtPosition,
    <I as nom::InputTakeAtPosition>::Item: nom::AsChar + Clone,
    E: nom::error::ParseError<I>,
    F: nom::Parser<I, O, E>,
{
    delimited(multispace0, f, multispace0)
}
