#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenKind {
    /// A sequence of characters matching `Char::is_whitespace`, excluding '\n' and '\r'
    Whitespace,

    /// '\n' or '\r\n'
    Newline,

    /// `// ...`
    LineComment,

    /// `/* ... */`
    BlockComment,

    /// '{'
    LBrace,

    /// '}'
    RBrace,

    /// '='
    Eq,

    /// ';'
    Semi,

    /// A valid maud identifier
    Ident,

    /// Quote delimited string
    Str,
}

impl TokenKind {
    pub fn is_trivia(self) -> bool {
        matches!(self, TokenKind::Newline | TokenKind::Whitespace) || self.is_comment()
    }

    pub fn is_comment(self) -> bool {
        matches!(self, TokenKind::LineComment | TokenKind::BlockComment)
    }
}
