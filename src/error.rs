use miette::Diagnostic;
use thiserror::Error;

use crate::span::Span;

#[derive(Debug, Diagnostic, Error)]
pub enum ParseError {
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
