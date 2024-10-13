use crate::{
    ast::{Block, Element, ElementBody, Markup, Node, Str},
    error::{LexError, ParseError},
    kind::TokenKind,
    lexer::Lexer,
    token::TokenWithTrivia,
};

pub type ParseResult<T> = Result<T, ParseError>;

pub fn parse(input: &str) -> ParseResult<Markup> {
    Parser::new(input).parse()
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let lexer = Lexer::new(input);
        Self { lexer }
    }

    pub fn parse(&mut self) -> ParseResult<Markup> {
        let root = self.parse_block()?;
        Ok(Markup { root })
    }

    fn next(&mut self) -> Result<Option<TokenWithTrivia>, LexError> {
        self.lexer.next_token()
    }

    fn expect_next(&mut self, expected_kind: TokenKind) -> ParseResult<TokenWithTrivia> {
        let next = self.next()?.ok_or(ParseError::UnexpectedEndOfInput)?;

        if next.token.kind == expected_kind {
            return Ok(next);
        }

        // TODO: Better error reporting with `expected_kind`
        Err(ParseError::UnexpectedToken {
            span: next.token.span,
        })
    }

    fn peek(&mut self) -> Result<&Option<TokenWithTrivia>, LexError> {
        self.lexer.peek_token()
    }

    fn parse_block(&mut self) -> ParseResult<Block> {
        let open_brace = self.expect_next(TokenKind::LBrace)?;
        let mut nodes = Vec::new();

        while let Some(t) = self.peek()? {
            if t.token.kind == TokenKind::RBrace {
                break;
            }
            let n = self.parse_node()?;
            nodes.push(n);
        }

        let close_brace = self.expect_next(TokenKind::RBrace)?;

        Ok(Block {
            open_brace,
            nodes,
            close_brace,
        })
    }

    fn parse_node(&mut self) -> ParseResult<Node> {
        let peeked = self
            .peek()?
            .as_ref()
            .ok_or(ParseError::UnexpectedEndOfInput)?;

        match peeked.token.kind {
            TokenKind::LBrace => self.parse_block().map(Node::Block),
            TokenKind::Ident => self.parse_element().map(Node::Element),
            TokenKind::Str => self.parse_string().map(Node::Str),
            _ => Err(ParseError::UnexpectedToken {
                span: peeked.token.span.clone(),
            }),
        }
    }

    fn parse_string(&mut self) -> ParseResult<Str> {
        self.expect_next(TokenKind::Str).map(Str)
    }

    fn parse_element(&mut self) -> ParseResult<Element> {
        let tag = self.parse_ident()?;
        let body = self.parse_element_body()?;

        Ok(Element { tag, body })
    }

    fn parse_ident(&mut self) -> ParseResult<TokenWithTrivia> {
        self.expect_next(TokenKind::Ident)
    }

    fn parse_element_body(&mut self) -> ParseResult<ElementBody> {
        let peeked = self
            .peek()?
            .as_ref()
            .ok_or(ParseError::UnexpectedEndOfInput)?;

        match peeked.token.kind {
            TokenKind::LBrace => self.parse_block().map(ElementBody::Block),
            TokenKind::Semi => self.expect_next(TokenKind::Semi).map(ElementBody::Void),
            _ => Err(ParseError::UnexpectedToken {
                span: peeked.token.span.clone(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let input = r#"{
            h1 {
                "Hello world"
            }
        }"#;

        let markup = Parser::new(input).parse();
        insta::assert_debug_snapshot!(markup);
    }

    #[test]
    fn comments() {
        let input = r#"{
            // line comment
            h1 { /* block comment */ "Hello world" }
        }"#;

        let markup = Parser::new(input).parse();
        insta::assert_debug_snapshot!(markup);
    }
}
