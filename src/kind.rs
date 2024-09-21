#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SyntaxKind {
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

    /// End of file
    Eof,
}

impl SyntaxKind {
    pub fn is_eof(self) -> bool {
        self == Self::Eof
    }
}
