use std::ops::Range;

use miette::SourceSpan;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, multispace0, multispace1},
    combinator::{all_consuming, cut, fail, map, opt, recognize, value},
    error::VerboseError,
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
    Finish,
};

mod combinator;
mod expr;
mod ident;
mod literal;
mod path;
mod pattern;

use combinator::ws;
use expr::expr;
use ident::keyword;
use literal::str_lit;
use pattern::pattern;

use crate::error::ParseError;

pub type NomResult<'a, O> = Result<(&'a str, O), nom::Err<VerboseError<&'a str>>>;

#[derive(Clone, Debug)]
pub struct For<'a> {
    pub pattern: &'a str,
    pub expr: &'a str,
    pub body: Block<'a>,
}

#[derive(Clone, Debug)]
pub enum Else<'a> {
    If(If<'a>),
    Then(Block<'a>),
}

#[derive(Clone, Debug)]
pub struct If<'a> {
    pub cond: &'a str,
    pub body: Block<'a>,
    pub else_clause: Option<Box<Else<'a>>>,
}

#[derive(Clone, Debug)]
pub enum ControlStructure<'a> {
    If(If<'a>),
    For(For<'a>),
}

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
    ControlStructure(ControlStructure<'a>),
}

#[derive(Clone, Debug)]
pub struct Markup<'a> {
    pub nodes: Vec<Node<'a>>,
}

fn for_expr(input: &str) -> NomResult<For> {
    preceded(
        keyword("for"),
        map(
            tuple((ws(pattern), keyword("in"), ws(expr(false)), block)),
            |(pattern, _, expr, body)| For {
                pattern,
                expr,
                body,
            },
        ),
    )(input)
}

fn else_expr(input: &str) -> NomResult<Else> {
    preceded(
        keyword("else"),
        ws(alt((map(if_expr, Else::If), map(block, Else::Then)))),
    )(input)
}

fn if_expr(input: &str) -> NomResult<If> {
    let opt_else = opt(preceded(char('@'), map(else_expr, Box::new)));

    preceded(
        keyword("if"),
        map(
            tuple((ws(expr(false)), block, ws(opt_else))),
            |(cond, body, else_clause)| If {
                cond,
                body,
                else_clause,
            },
        ),
    )(input)
}

fn control_structure(input: &str) -> NomResult<ControlStructure> {
    preceded(
        char('@'),
        alt((
            map(if_expr, ControlStructure::If),
            map(for_expr, ControlStructure::For),
            cut(fail),
        )),
    )(input)
}

fn splice(input: &str) -> NomResult<Splice> {
    map(delimited(char('('), ws(expr(true)), char(')')), |expr| {
        Splice { expr }
    })(input)
}

fn block(input: &str) -> NomResult<Block> {
    delimited(
        char('{'),
        map(pair(multispace0, ws(nodes)), |(whitespace, nodes)| Block {
            newline: whitespace.contains('\n'),
            nodes,
        }),
        cut(char('}')),
    )(input)
}

fn void(input: &str) -> NomResult<()> {
    value((), char(';'))(input)
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
    map(tuple((tag_name, ws(attrs), body)), |(name, attrs, body)| {
        Element { name, attrs, body }
    })(input)
}

fn nodes(input: &str) -> NomResult<Vec<Node>> {
    many0(preceded(
        multispace0,
        alt((
            map(element, Node::Element),
            map(str_lit, Node::StrLit),
            map(block, Node::Block),
            map(splice, Node::Splice),
            map(control_structure, Node::ControlStructure),
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
        dbg!(&e);
        let (remaining_input, _) = e.errors.first().unwrap();
        let offset = src.find(remaining_input).unwrap_or_default();

        ParseError::UnexpectedToken {
            src: src.to_string(),
            err_span: SourceSpan::from((offset, 0)),
        }
    })
}
