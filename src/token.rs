use ecow::EcoString;

use crate::{kind::TokenKind, span::Span};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub text: EcoString,
    pub span: Span,
}

impl Token {
    pub fn text(&self) -> &str {
        &self.text
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct TokenWithTrivia {
    pub leading_trivia: Vec<Token>,
    pub token: Token,
    pub trailing_trivia: Vec<Token>,
}
