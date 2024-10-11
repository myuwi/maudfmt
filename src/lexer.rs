use ecow::EcoString;
use unicode_ident::{is_xid_continue, is_xid_start};
use unscanny::Scanner;

use crate::{
    error::LexError,
    kind::TokenKind,
    token::{Token, TokenWithTrivia},
};

pub type LexResult<T> = Result<T, LexError>;

pub struct Lexer<'a> {
    s: Scanner<'a>,
    peeked: Option<TokenWithTrivia>,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            s: Scanner::new(text),
            peeked: None,
        }
    }

    pub fn next_token(&mut self) -> LexResult<Option<TokenWithTrivia>> {
        if self.peeked.is_some() {
            return Ok(self.peeked.take());
        }
        self.process_next_token_with_trivia()
    }

    pub fn peek_token(&mut self) -> LexResult<&Option<TokenWithTrivia>> {
        if self.peeked.is_none() {
            self.peeked = self.process_next_token_with_trivia()?;
        }
        Ok(&self.peeked)
    }

    pub fn collect(&mut self) -> LexResult<Vec<TokenWithTrivia>> {
        std::iter::from_fn(move || self.next_token().transpose()).collect::<Result<Vec<_>, _>>()
    }

    fn process_next_token_with_trivia(&mut self) -> LexResult<Option<TokenWithTrivia>> {
        let mut leading_trivia = Vec::new();

        let token = loop {
            match self.process_next_token()? {
                Some(t) if t.kind.is_trivia() => {
                    leading_trivia.push(t);
                    continue;
                }
                Some(t) => break t,
                None => return Ok(None),
            }
        };

        let mut trailing_trivia = Vec::new();
        loop {
            if let Some(next) = self.s.peek() {
                if is_trivia_start(next) {
                    trailing_trivia.push(self.process_next_token()?.expect("should get trivia"));

                    // Trailing trivia should end at the first newline
                    if !is_newline(next) {
                        continue;
                    }
                }
            }
            break;
        }

        Ok(Some(TokenWithTrivia {
            leading_trivia,
            token,
            trailing_trivia,
        }))
    }

    fn process_next_token(&mut self) -> LexResult<Option<Token>> {
        let start = self.s.cursor();

        let Some(c) = self.s.eat() else {
            return Ok(None);
        };
        let kind = match c {
            c if is_space(c) => self.whitespace(),
            c if is_newline(c) => self.newline(),
            c if is_ident_start(c) => self.ident(),
            '/' if self.s.eat_if('/') => self.line_comment(),
            '/' if self.s.eat_if('*') => self.block_comment()?,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '"' => self.string()?,
            _ => return Err(LexError::UnexpectedCharacter { span: start..start }),
        };

        let end = self.s.cursor();
        let span = start..end;
        let text = EcoString::from(self.s.get(span.clone()));

        Ok(Some(Token { kind, text, span }))
    }

    fn whitespace(&mut self) -> TokenKind {
        self.s.eat_while(is_space);
        TokenKind::Whitespace
    }

    fn newline(&mut self) -> TokenKind {
        if self.s.before().ends_with('\r') {
            self.s.eat_if('\n');
        }
        TokenKind::Newline
    }

    fn line_comment(&mut self) -> TokenKind {
        self.s.eat_until(is_newline);
        TokenKind::LineComment
    }

    // TODO: nested block comment
    fn block_comment(&mut self) -> LexResult<TokenKind> {
        self.s.eat_until("*/");
        if !self.s.eat_if("*/") {
            return Err(LexError::UnterminatedBlockComment);
        }
        Ok(TokenKind::BlockComment)
    }

    fn ident(&mut self) -> TokenKind {
        self.s.eat_while(is_ident_continue);
        TokenKind::Ident
    }

    fn string(&mut self) -> LexResult<TokenKind> {
        loop {
            match self.s.eat() {
                Some('\\') => self.s.eat(),
                Some('"') => break,
                None => return Err(LexError::UnterminatedString),
                _ => None,
            };
        }
        Ok(TokenKind::Str)
    }
}

/// Any non-newline whitespace characters
fn is_space(c: char) -> bool {
    c.is_whitespace() && !is_newline(c)
}

/// '\n' or '\r'
fn is_newline(c: char) -> bool {
    c == '\n' || c == '\r'
}

fn is_trivia_start(c: char) -> bool {
    is_space(c) || is_newline(c) || c == '/'
}

fn is_ident_start(c: char) -> bool {
    is_xid_start(c)
}

fn is_ident_continue(c: char) -> bool {
    is_xid_continue(c) || c == '-'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let input = r#"{
            h1 { "Hello world" }
            p { "\"This string contains escaped quotes \"" }
        }"#;

        let tokens = Lexer::new(input).collect().unwrap();
        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn comments() {
        let input = r#"{
            // line comment
            h1 { /* block comment */ "Hello world" }
        }"#;

        let tokens = Lexer::new(input).collect().unwrap();
        insta::assert_debug_snapshot!(tokens);
    }
}
