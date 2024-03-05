use nom::{
    branch::alt,
    bytes::complete::{escaped, tag},
    character::complete::{alpha1, alphanumeric1, char, multispace0, multispace1, none_of, one_of},
    combinator::{map, recognize, value},
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
// TODO: Preserve newlines

#[derive(Clone, Debug)]
struct Block(Markup);

#[derive(Clone, Debug)]
enum ElementBody {
    Void,
    Block(Block),
}

#[derive(Clone, Debug)]
enum AttributeValue {
    String(String),
    Empty,
}

#[derive(Clone, Debug)]
struct Attribute {
    name: String,
    value: AttributeValue,
}

#[derive(Clone, Debug)]
struct Element {
    name: String,
    attrs: Vec<Attribute>,
    body: ElementBody,
}

#[derive(Clone, Debug)]
enum Node {
    Element(Element),
    Block(Block),
    Str(String),
}

#[derive(Clone, Debug)]
struct Markup(Vec<Node>);

fn string(input: &str) -> IResult<&str, String> {
    map(
        preceded(
            char('"'),
            terminated(escaped(none_of(r#"\""#), '\\', one_of(r#""n\"#)), char('"')),
        ),
        String::from,
    )(input)
}

fn block(input: &str) -> IResult<&str, Block> {
    delimited(
        preceded(multispace0, char('{')),
        map(markup, Block),
        preceded(multispace0, char('}')),
    )(input)
}

fn void(input: &str) -> IResult<&str, ()> {
    value((), preceded(multispace0, char(';')))(input)
}

fn body(input: &str) -> IResult<&str, ElementBody> {
    alt((
        value(ElementBody::Void, void),
        map(block, ElementBody::Block),
    ))(input)
}

fn non_empty_attribute(input: &str) -> IResult<&str, Attribute> {
    map(separated_pair(tag_name, char('='), string), |a| Attribute {
        name: a.0.to_string(),
        value: AttributeValue::String(a.1),
    })(input)
}

fn empty_attribute(input: &str) -> IResult<&str, Attribute> {
    map(tag_name, |a| Attribute {
        name: a.to_string(),
        value: AttributeValue::Empty,
    })(input)
}

fn attrs(input: &str) -> IResult<&str, Vec<Attribute>> {
    separated_list0(multispace1, alt((non_empty_attribute, empty_attribute)))(input)
}

fn tag_name(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alpha1,
        many0(alt((alphanumeric1, tag("_"), tag("-")))),
    ))(input)
}

fn element(input: &str) -> IResult<&str, Element> {
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

fn markup(input: &str) -> IResult<&str, Markup> {
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

pub fn parse(src: &str) -> Vec<String> {
    let result = markup(src);

    match result {
        Ok(res) => {
            dbg!(res);
        }
        Err(err) => eprintln!("{}", err),
    }

    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_block_element() {
        let expected = r#"Ok(("", Element { name: "h1", attrs: [], body: Block(Block(Markup([Str("Poem")]))) }))"#;

        assert_eq!(format!("{:?}", element(r#"h1 { "Poem" }"#)), expected);
        assert_eq!(format!("{:?}", element(r#"h1{"Poem"}"#)), expected);
        assert_eq!(
            format!(
                "{:?}",
                element(
                    r#"h1
                    {
                    "Poem"
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
            r#"Ok(("", Markup([Block(Block(Markup([Element(Element { name: "input", attrs: [], body: Void })])))])))"#
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
