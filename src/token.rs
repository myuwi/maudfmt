use std::ops::Range;

use crate::kind::TokenKind;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
    pub span: Range<usize>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct TokenWithTrivia<'a> {
    pub token: Token<'a>,
    pub leading_trivia: Vec<Token<'a>>,
    pub trailing_trivia: Vec<Token<'a>>,
}
