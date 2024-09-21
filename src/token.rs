use std::ops::Range;

use crate::kind::SyntaxKind;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Token<'a> {
    pub kind: SyntaxKind,
    pub text: &'a str,
    pub span: Range<usize>,
}
