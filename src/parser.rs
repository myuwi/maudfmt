use std::ops::Range;

use miette::SourceSpan;
use nom::{
    branch::alt,
    bytes::complete::{escaped, tag},
    character::complete::{
        alpha1, alphanumeric1, anychar, char, multispace0, multispace1, none_of,
    },
    combinator::{all_consuming, cut, fail, map, opt, recognize, value},
    error::{ErrorKind, VerboseError},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
    Finish, InputTake,
};

use crate::error::ParseError;

type NomResult<'a, O> = Result<(&'a str, O), nom::Err<VerboseError<&'a str>>>;

#[derive(Clone, Debug)]
pub struct Splice<'a> {
    pub expr: &'a str,
}

#[derive(Clone, Debug)]
pub struct Block<'a> {
    pub newline: bool,
    pub nodes: Vec<Node<'a>>,
}

#[derive(Clone, Debug)]
pub enum ElementBody<'a> {
    Void,
    Block(Block<'a>),
}

#[derive(Clone, Debug)]
pub enum AttributeValue<'a> {
    String(&'a str),
    Empty,
}

#[derive(Clone, Debug)]
pub struct Attribute<'a> {
    pub name: &'a str,
    pub value: AttributeValue<'a>,
}

#[derive(Clone, Debug)]
pub struct Element<'a> {
    pub name: &'a str,
    pub attrs: Vec<Attribute<'a>>,
    pub body: ElementBody<'a>,
}

#[derive(Clone, Debug)]
pub enum Node<'a> {
    Element(Element<'a>),
    Block(Block<'a>),
    StrLit(&'a str),
    Splice(Splice<'a>),
}

#[derive(Clone, Debug)]
pub struct Markup<'a> {
    pub nodes: Vec<Node<'a>>,
}

fn splice_inner(input: &str) -> NomResult<&str> {
    let mut paren_count = 0;

    let mut it = input.chars().enumerate();
    while let Some((i, char)) = it.next() {
        match char {
            '(' => paren_count += 1,
            ')' => paren_count -= 1,
            '"' | '\'' => {
                let parser = if char == '"' { str_lit } else { char_lit };

                let (_, str) = parser(&input[i..])?;
                let char_count = str.chars().count();

                it.nth(char_count);
                continue;
            }
            _ => (),
        }

        if paren_count == -1 {
            return Ok(input.take_split(i));
        }
    }

    use nom::error::ParseError;
    Err(nom::Err::Error(VerboseError::from_error_kind(
        input,
        ErrorKind::TakeUntil,
    )))
}

fn splice(input: &str) -> NomResult<Splice> {
    map(delimited(char('('), splice_inner, char(')')), |s| Splice {
        expr: s.trim(),
    })(input)
}

fn char_lit(input: &str) -> NomResult<&str> {
    map(
        delimited(
            char('\''),
            opt(escaped(none_of("\\'"), '\\', anychar)),
            char('\''),
        ),
        |s| s.unwrap_or_default(),
    )(input)
}

fn str_lit(input: &str) -> NomResult<&str> {
    map(
        delimited(
            char('"'),
            opt(escaped(none_of("\\\""), '\\', anychar)),
            char('"'),
        ),
        |s| s.unwrap_or_default(),
    )(input)
}

fn block(input: &str) -> NomResult<Block> {
    delimited(
        preceded(multispace0, char('{')),
        map(tuple((multispace0, nodes)), |(whitespace, nodes)| Block {
            newline: whitespace.contains('\n'),
            nodes,
        }),
        preceded(multispace0, cut(char('}'))),
    )(input)
}

fn void(input: &str) -> NomResult<()> {
    value((), preceded(multispace0, char(';')))(input)
}

fn body(input: &str) -> NomResult<ElementBody> {
    alt((
        value(ElementBody::Void, void),
        map(block, ElementBody::Block),
        cut(fail),
    ))(input)
}

fn non_empty_attribute(input: &str) -> NomResult<Attribute> {
    map(separated_pair(tag_name, char('='), str_lit), |a| {
        Attribute {
            name: a.0,
            value: AttributeValue::String(a.1),
        }
    })(input)
}

fn empty_attribute(input: &str) -> NomResult<Attribute> {
    map(tag_name, |a| Attribute {
        name: a,
        value: AttributeValue::Empty,
    })(input)
}

fn attrs(input: &str) -> NomResult<Vec<Attribute>> {
    separated_list0(multispace1, alt((non_empty_attribute, empty_attribute)))(input)
}

fn tag_name(input: &str) -> NomResult<&str> {
    recognize(pair(
        alpha1,
        many0(alt((alphanumeric1, tag("_"), tag("-")))),
    ))(input)
}

fn element(input: &str) -> NomResult<Element> {
    map(
        tuple((
            tag_name,
            preceded(multispace0, attrs),
            preceded(multispace0, body),
        )),
        |(name, attrs, body)| Element { name, attrs, body },
    )(input)
}

fn nodes(input: &str) -> NomResult<Vec<Node>> {
    many0(preceded(
        multispace0,
        alt((
            map(element, Node::Element),
            map(str_lit, Node::StrLit),
            map(block, Node::Block),
            map(splice, Node::Splice),
        )),
    ))(input)
}

fn markup(input: &str) -> Result<Markup, VerboseError<&str>> {
    all_consuming(map(nodes, |n| Markup { nodes: n }))(input)
        .finish()
        .map(|(_, markup)| markup)
}

pub fn parse_range(src: &str, range: Range<usize>) -> Result<Markup, ParseError> {
    let content = src[range].trim();

    markup(content).map_err(|e| {
        let (remaining_input, _) = e.errors.first().unwrap();
        let offset = src.find(remaining_input).unwrap_or_default();

        ParseError::UnexpectedToken {
            src: src.to_string(),
            err_span: SourceSpan::from((offset, 0)),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_block_element() {
        let expected = r#"Ok(("", Element { name: "h1", attrs: [], body: Block(Block { newline: false, markup: Markup([Str("Poem")]) }) }))"#;

        assert_eq!(format!("{:?}", element(r#"h1 { "Poem" }"#)), expected);
        assert_eq!(format!("{:?}", element(r#"h1{"Poem"}"#)), expected);
        assert_eq!(
            format!(
                "{:?}",
                element(
                    r#"h1
                    {"Poem"
                    }"#
                )
            ),
            expected
        );
    }

    #[test]
    fn test_parse_void_element() {
        assert_eq!(
            format!("{:?}", element(r#"input;"#)),
            r#"Ok(("", Element { name: "input", attrs: [], body: Void }))"#
        );
    }

    #[test]
    fn test_parse_attrs() {
        assert_eq!(
            format!("{:?}", element(r#"input type="checkbox" checked;"#)),
            r#"Ok(("", Element { name: "input", attrs: [Attribute { name: "type", value: String("checkbox") }, Attribute { name: "checked", value: Empty }], body: Void }))"#
        );
    }

    #[test]
    fn test_parse_block() {
        assert_eq!(
            format!("{:?}", nodes(r#"{ input; }"#)),
            r#"Ok(("", Markup([Block(Block { newline: false, markup: Markup([Element(Element { name: "input", attrs: [], body: Void })]) })])))"#
        );

        assert_eq!(
            format!(
                "{:?}",
                block(
                    r#"{
                    input;
                }"#
                )
            ),
            r#"Ok(("", Block { newline: true, markup: Markup([Element(Element { name: "input", attrs: [], body: Void })]) }))"#
        );
    }

    #[test]
    fn test_parse_multiroot() {
        assert_eq!(
            format!("{:?}", nodes(r#""a" "b""#)),
            r#"Ok(("", Markup([Str("a"), Str("b")])))"#
        );
    }
}
