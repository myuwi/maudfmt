use ecow::EcoString;
use unicode_ident::{is_xid_continue, is_xid_start};
use unscanny::Scanner;

use crate::{
    kind::TokenKind,
    token::{Token, TokenWithTrivia},
};

pub struct Lexer<'a> {
    s: Scanner<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            s: Scanner::new(text),
        }
    }

    fn process_next_token(&mut self) -> Option<Token> {
        let start = self.s.cursor();

        let kind = match self.s.eat()? {
            c if is_space(c) => self.whitespace(),
            c if is_newline(c) => self.newline(),
            c if is_ident_start(c) => self.ident(),
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '"' => self.string(),
            _ => panic!("Invalid character"),
        };

        let end = self.s.cursor();
        let span = start..end;
        let text = EcoString::from(self.s.get(span.clone()));

        Some(Token { kind, text, span })
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

    fn ident(&mut self) -> TokenKind {
        self.s.eat_while(is_ident_continue);
        TokenKind::Ident
    }

    fn string(&mut self) -> TokenKind {
        loop {
            match self.s.eat() {
                Some('\\') => self.s.eat(),
                Some('"') => break,
                None => panic!("Unterminated string"),
                _ => None,
            };
        }
        TokenKind::Str
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = TokenWithTrivia;

    fn next(&mut self) -> Option<Self::Item> {
        let mut leading_trivia = Vec::new();

        let token = loop {
            let t = self.process_next_token()?;
            if t.kind.is_trivia() {
                leading_trivia.push(t);
                continue;
            };
            break t;
        };

        let mut trailing_trivia = Vec::new();
        loop {
            if let Some(next) = self.s.peek() {
                if is_trivia_start(next) {
                    trailing_trivia.push(self.process_next_token().expect("should get trivia"));

                    // Trailing trivia should end at the first newline
                    if !is_newline(next) {
                        continue;
                    }
                }
            }
            break;
        }

        Some(TokenWithTrivia {
            leading_trivia,
            token,
            trailing_trivia,
        })
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
    is_space(c) || is_newline(c)
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

        let lexer = Lexer::new(input);
        let tokens = lexer.collect::<Vec<_>>();

        insta::assert_debug_snapshot!(tokens);
    }
}
