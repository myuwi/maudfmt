use nom::{
    character::complete::{multispace0, multispace1},
    sequence::preceded,
};

pub fn ws0<'a, O, E, F>(f: F) -> impl nom::Parser<&'a str, O, E>
where
    E: nom::error::ParseError<&'a str>,
    F: nom::Parser<&'a str, O, E>,
{
    preceded(multispace0, f)
}

pub fn ws1<'a, O, E, F>(f: F) -> impl nom::Parser<&'a str, O, E>
where
    E: nom::error::ParseError<&'a str>,
    F: nom::Parser<&'a str, O, E>,
{
    preceded(multispace1, f)
}
