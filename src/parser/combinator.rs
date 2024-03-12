use nom::{character::complete::multispace0, sequence::preceded};

pub fn ws<'a, O, E, F>(f: F) -> impl nom::Parser<&'a str, O, E>
where
    E: nom::error::ParseError<&'a str>,
    F: nom::Parser<&'a str, O, E>,
{
    preceded(multispace0, f)
}
