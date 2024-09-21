use unicode_ident::{is_xid_continue, is_xid_start};
use unscanny::Scanner;

use crate::kind::SyntaxKind;

pub struct Lexer<'a> {
    s: Scanner<'a>,
    tokens: Vec<SyntaxKind>,
}

#[allow(dead_code)]
impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            s: Scanner::new(text),
            tokens: Vec::new(),
        }
    }

    pub fn tokenize(mut self) -> Vec<SyntaxKind> {
        loop {
            let kind = self.next();
            self.tokens.push(kind);

            if kind.is_eof() {
                break;
            }
        }
        self.tokens
    }

    fn next(&mut self) -> SyntaxKind {
        match self.s.eat() {
            Some(c) if is_space(c) => self.whitespace(),
            Some(c) if is_newline(c) => self.newline(),
            Some(c) if is_ident_start(c) => self.ident(),
            Some('{') => SyntaxKind::LBrace,
            Some('}') => SyntaxKind::RBrace,
            None => SyntaxKind::Eof,
            _ => panic!("Invalid character"),
        }
    }

    fn whitespace(&mut self) -> SyntaxKind {
        self.s.eat_while(is_space);
        SyntaxKind::Whitespace
    }

    fn newline(&mut self) -> SyntaxKind {
        if self.s.before().ends_with('\r') {
            self.s.eat_if('\n');
        }
        SyntaxKind::Newline
    }

    fn ident(&mut self) -> SyntaxKind {
        self.s.eat_while(is_ident_continue);
        SyntaxKind::Ident
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
            h1 {
                p {}
            }
        }"#;

        let tokens = Lexer::new(input).tokenize();

        insta::assert_debug_snapshot!(tokens);
    }
}
