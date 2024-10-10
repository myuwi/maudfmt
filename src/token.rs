use ecow::EcoString;

use crate::{kind::TokenKind, span::Span};

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

#[derive(Debug)]
pub struct TokenWithTrivia {
    pub leading_trivia: Vec<Token>,
    pub token: Token,
    pub trailing_trivia: Vec<Token>,
}
