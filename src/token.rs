use std::ops::Range;

use crate::kind::TokenKind;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
    pub span: Range<usize>,
}
