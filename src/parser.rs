use std::process::exit;

use miette::{miette, LabeledSpan, Severity};
use nom::{
    branch::alt,
    bytes::complete::{escaped, tag},
    character::complete::{alpha1, alphanumeric1, char, multispace0, multispace1, none_of, one_of},
    combinator::{cut, map, recognize, value},
    error::VerboseError,
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

// TODO: Raw strings
// TODO: Classes and IDs: .foo #bar
// TODO: Implicit div elements
// TODO: Splices
// TODO: Toggles
// TODO: Control structures

#[derive(Clone, Debug)]
pub struct Block {
    pub newline: bool,
    pub markup: Markup,
}

#[derive(Clone, Debug)]
pub enum ElementBody {
    Void,
    Block(Block),
}

#[derive(Clone, Debug)]
pub enum AttributeValue {
    String(String),
    Empty,
}

#[derive(Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub value: AttributeValue,
}

#[derive(Clone, Debug)]
pub struct Element {
    pub name: String,
    pub attrs: Vec<Attribute>,
    pub body: ElementBody,
}

#[derive(Clone, Debug)]
pub enum Node {
    Element(Element),
    Block(Block),
    Str(String),
}

#[derive(Clone, Debug)]
pub struct Markup(pub Vec<Node>);

fn string(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    map(
        preceded(
            char('"'),
            terminated(escaped(none_of(r#"\""#), '\\', one_of(r#""n\"#)), char('"')),
        ),
        String::from,
    )(input)
}

fn block(input: &str) -> IResult<&str, Block, VerboseError<&str>> {
    delimited(
        preceded(multispace0, char('{')),
        map(tuple((multispace0, markup)), |(whitespace, markup)| Block {
            newline: whitespace.contains('\n'),
            markup,
        }),
        preceded(multispace0, cut(char('}'))),
    )(input)
}

fn void(input: &str) -> IResult<&str, (), VerboseError<&str>> {
    value((), preceded(multispace0, char(';')))(input)
}

fn body(input: &str) -> IResult<&str, ElementBody, VerboseError<&str>> {
    alt((
        value(ElementBody::Void, void),
        map(block, ElementBody::Block),
    ))(input)
}

fn non_empty_attribute(input: &str) -> IResult<&str, Attribute, VerboseError<&str>> {
    map(separated_pair(tag_name, char('='), string), |a| Attribute {
        name: a.0.to_string(),
        value: AttributeValue::String(a.1),
    })(input)
}

fn empty_attribute(input: &str) -> IResult<&str, Attribute, VerboseError<&str>> {
    map(tag_name, |a| Attribute {
        name: a.to_string(),
        value: AttributeValue::Empty,
    })(input)
}

fn attrs(input: &str) -> IResult<&str, Vec<Attribute>, VerboseError<&str>> {
    separated_list0(multispace1, alt((non_empty_attribute, empty_attribute)))(input)
}

fn tag_name(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    recognize(pair(
        alpha1,
        many0(alt((alphanumeric1, tag("_"), tag("-")))),
    ))(input)
}

fn element(input: &str) -> IResult<&str, Element, VerboseError<&str>> {
    map(
        tuple((
            tag_name,
            preceded(multispace0, attrs),
            preceded(multispace0, body),
        )),
        |(name, attrs, body)| Element {
            name: name.to_string(),
            attrs,
            body,
        },
    )(input)
}

fn markup(input: &str) -> IResult<&str, Markup, VerboseError<&str>> {
    map(
        many0(preceded(
            multispace0,
            alt((
                map(element, Node::Element),
                map(string, Node::Str),
                map(block, Node::Block),
            )),
        )),
        Markup,
    )(input)
}

pub fn parse(src: &str, full_file: &str) -> Markup {
    match markup(src) {
        Ok((_, markup)) => markup,
        Err(e) => {
            let e = match e {
                nom::Err::Error(e) | nom::Err::Failure(e) => e,
                nom::Err::Incomplete(_) => {
                    eprintln!("Incomplete input");
                    exit(1);
                }
            };

            let context = e.errors.first().unwrap();
            let offset = full_file.find(context.0).unwrap_or_default();

            let text = "Unknown token";

            let report = miette!(
                severity = Severity::Error,
                labels = vec![LabeledSpan::at_offset(offset, text)],
                "Parsing error"
            )
            .with_source_code(full_file.to_string());

            eprintln!("{:?}", report);
            exit(1)
        }
    }
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
            format!("{:?}", markup(r#"{ input; }"#)),
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
            format!("{:?}", markup(r#""a" "b""#)),
            r#"Ok(("", Markup([Str("a"), Str("b")])))"#
        );
    }
}
