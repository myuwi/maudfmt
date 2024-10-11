use miette::Diagnostic;
use thiserror::Error;

use crate::span::Span;

#[derive(Debug, Diagnostic, Error)]
pub enum LexError {
    #[error("Found unexpected character.")]
    #[diagnostic(code("maudfmt::lexer::unexpected_character"))]
    UnexpectedCharacter {
        #[label("here")]
        span: Span,
    },

    // TODO: add span
    #[error("Found unterminated string.")]
    #[diagnostic(code("maudfmt::lexer::unterminated_string"))]
    UnterminatedString,

    // TODO: add span
    #[error("Found unterminated block comment.")]
    #[diagnostic(code("maudfmt::lexer::unterminated_block_comment"))]
    UnterminatedBlockComment,
}

#[derive(Debug, Diagnostic, Error)]
pub enum ParseError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    LexError(#[from] LexError),

    #[error("Found unexpected token.")]
    #[diagnostic(code("maudfmt::parser::unexpected_token"))]
    UnexpectedToken {
        #[label("here")]
        span: Span,
    },

    #[error("Found unexpected end of input.")]
    #[diagnostic(code("maudfmt::parser::unexpected_end_of_input"))]
    UnexpectedEndOfInput,
}
