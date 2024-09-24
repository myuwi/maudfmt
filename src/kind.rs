#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenKind {
    /// A sequence of characters matching `Char::is_whitespace`, excluding '\n' and '\r'
    Whitespace,

    /// '\n' or '\r\n'
    Newline,

    /// '{'
    LBrace,

    /// '}'
    RBrace,

    /// A valid maud identifier
    Ident,

    /// Quote delimited string
    Str,
}

impl TokenKind {
    pub fn is_trivia(self) -> bool {
        matches!(self, TokenKind::Newline | TokenKind::Whitespace)
    }
}
