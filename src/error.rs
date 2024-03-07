use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
#[error("Unable to parse markup")]
pub enum ParseError {
    #[diagnostic(code("maudfmt::parser::unexpected_token"))]
    #[error("Found unexpected token")]
    UnexpectedToken {
        #[source_code]
        src: String,
        #[label = "here"]
        err_span: SourceSpan,
    },
}
