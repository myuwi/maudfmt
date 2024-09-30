use crate::token::Token;

#[allow(dead_code)]
#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(Token),
    UnexpectedEndOfInput,
}
