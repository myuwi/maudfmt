use std::ops::Range;

use miette::SourceSpan;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, multispace0, multispace1, not_line_ending},
    combinator::{all_consuming, cut, map, opt, recognize, value},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    Finish,
};

mod combinator;
mod error;
mod expr;
mod ident;
mod literal;
mod path;
mod pattern;

use combinator::ws;
use error::ParserError;
use expr::{expr, group};
use ident::keyword;
use literal::str_lit;
use pattern::pattern;

use crate::error::ParseError;

pub type NomResult<'a, O> = Result<(&'a str, O), nom::Err<ParserError<&'a str>>>;

#[derive(Clone, Debug)]
pub struct MatchArm<'a> {
    pub pattern: &'a str,
    pub body: Node<'a>,
}

#[derive(Clone, Debug)]
pub struct Match<'a> {
    pub scrut: &'a str,
    pub arms: Vec<MatchArm<'a>>,
}

#[derive(Clone, Debug)]
pub struct Let<'a> {
    pub expr: &'a str,
}

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
    Let(Let<'a>),
    Match(Match<'a>),
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
    Comment(&'a str),
    Splice(Splice<'a>),
    ControlStructure(ControlStructure<'a>),
}

#[derive(Clone, Debug)]
pub struct Markup<'a> {
    pub nodes: Vec<Node<'a>>,
}

fn match_arms(input: &str) -> NomResult<Vec<MatchArm>> {
    many0(map(
        ws(separated_pair(
            expr(true),
            ws(tag("=>")),
            terminated(node, opt(char(','))),
        )),
        |(pattern, body)| MatchArm { pattern, body },
    ))(input)
}

fn match_expr(input: &str) -> NomResult<Match> {
    preceded(
        keyword("match"),
        map(
            pair(
                ws(expr(false)),
                delimited(char('{'), ws(match_arms), char('}')),
            ),
            |(scrut, arms)| Match { scrut, arms },
        ),
    )(input)
}

fn let_expr(input: &str) -> NomResult<Let> {
    preceded(
        keyword("let"),
        map(terminated(ws(expr(true)), char(';')), |expr| Let { expr }),
    )(input)
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
        cut(alt((
            map(if_expr, ControlStructure::If),
            map(for_expr, ControlStructure::For),
            map(let_expr, ControlStructure::Let),
            map(match_expr, ControlStructure::Match),
        ))),
    )(input)
}

fn splice(input: &str) -> NomResult<Splice> {
    map(group('(', ')'), |expr| Splice {
        expr: expr[1..expr.len() - 1].trim(),
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

fn comment(input: &str) -> NomResult<&str> {
    delimited(tag("//"), not_line_ending, multispace0)(input)
}

fn void(input: &str) -> NomResult<()> {
    value((), char(';'))(input)
}

fn body(input: &str) -> NomResult<ElementBody> {
    cut(alt((
        value(ElementBody::Void, void),
        map(block, ElementBody::Block),
    )))(input)
}

fn non_empty_attribute(input: &str) -> NomResult<Attribute> {
    map(separated_pair(tag_name, ws(char('=')), str_lit), |a| {
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

fn node(input: &str) -> NomResult<Node> {
    alt((
        map(element, Node::Element),
        map(str_lit, Node::StrLit),
        map(comment, Node::Comment),
        map(block, Node::Block),
        map(splice, Node::Splice),
        map(control_structure, Node::ControlStructure),
    ))(input)
}

fn nodes(input: &str) -> NomResult<Vec<Node>> {
    many0(preceded(multispace0, node))(input)
}

fn markup(input: &str) -> Result<Markup, ParserError<&str>> {
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
