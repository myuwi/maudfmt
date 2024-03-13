use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while, take_while1},
    character::complete::{anychar, char, digit1, none_of, one_of},
    combinator::{map, opt, peek, recognize, verify},
    sequence::{delimited, pair, preceded, terminated, tuple},
    AsChar,
};

use super::{ident::keyword, NomResult};

pub fn bool_lit(input: &str) -> NomResult<&str> {
    alt((keyword("true"), keyword("false")))(input)
}

pub fn char_lit(input: &str) -> NomResult<&str> {
    map(
        delimited(
            char('\''),
            opt(escaped(none_of("\\'"), '\\', anychar)),
            char('\''),
        ),
        |s| s.unwrap_or_default(),
    )(input)
}

pub fn byte_lit(input: &str) -> NomResult<&str> {
    preceded(char('b'), char_lit)(input)
}

pub fn str_lit(input: &str) -> NomResult<&str> {
    map(
        delimited(
            char('"'),
            opt(escaped(none_of("\\\""), '\\', anychar)),
            char('"'),
        ),
        |s| s.unwrap_or_default(),
    )(input)
}

pub fn byte_str_lit(input: &str) -> NomResult<&str> {
    preceded(char('b'), str_lit)(input)
}

pub fn int_digits(radix: u32) -> impl FnMut(&str) -> NomResult<&str> {
    move |i| {
        recognize(tuple((
            take_while(|c: char| c.is_digit(radix) || c == '_'),
            take_while1(|c: char| c.is_digit(radix)),
            take_while(|c: char| c.is_digit(radix) || c == '_'),
        )))(i)
    }
}

fn dec_lit(input: &str) -> NomResult<&str> {
    recognize(pair(
        digit1,
        take_while(|c: char| c.is_dec_digit() || c == '_'),
    ))(input)
}

pub fn int_lit(input: &str) -> NomResult<&str> {
    let int_suffix = alt((
        tag("i8"),
        tag("i16"),
        tag("i32"),
        tag("i64"),
        tag("i128"),
        tag("isize"),
        tag("u8"),
        tag("u16"),
        tag("u32"),
        tag("u64"),
        tag("u128"),
        tag("usize"),
    ));

    recognize(pair(
        alt((
            dec_lit,
            recognize(pair(
                char('0'),
                alt((
                    pair(char('b'), int_digits(2)),
                    pair(char('0'), int_digits(8)),
                    pair(char('x'), int_digits(16)),
                )),
            )),
        )),
        opt(int_suffix),
    ))(input)
}

pub fn float_lit(input: &str) -> NomResult<&str> {
    let float_suffix = |i| alt((tag("f32"), tag("f64")))(i);
    let float_exp = |i| recognize(tuple((one_of("eE"), opt(one_of("+-")), int_digits(10))))(i);

    recognize(tuple((
        dec_lit,
        alt((
            recognize(terminated(
                char('.'),
                verify(peek(anychar), |c| {
                    !(c.is_alphanum() || *c == '_' || *c == '.' || *c >= '\u{0080}')
                }),
            )),
            recognize(tuple((
                opt(pair(char('.'), dec_lit)),
                opt(float_exp),
                opt(float_suffix),
            ))),
        )),
    )))(input)
}
