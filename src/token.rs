use std::ops::Range;

use ecow::EcoString;

use crate::kind::TokenKind;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub text: EcoString,
    pub span: Range<usize>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct TokenWithTrivia {
    pub leading_trivia: Vec<Token>,
    pub token: Token,
    pub trailing_trivia: Vec<Token>,
}
