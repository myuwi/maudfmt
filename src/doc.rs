use pretty::RcDoc;

use crate::{
    ast::{Block, Element, ElementBody, Markup, Node, Str},
    kind::TokenKind,
    token::Token,
};

const INDENT: isize = 4;

pub trait Doc {
    fn to_doc(&self) -> RcDoc;
}

impl Doc for Markup {
    fn to_doc(&self) -> RcDoc {
        self.root.to_doc()
    }
}

impl Doc for Block {
    fn to_doc(&self) -> RcDoc {
        let mut doc = RcDoc::nil()
            .append(self.open_brace.token.text())
            .append(handle_trailing_trivia(&self.open_brace.trailing_trivia))
            .append(self.nodes.to_doc())
            .append(handle_leading_trivia(
                &self.close_brace.leading_trivia,
                true,
            ))
            .nest(INDENT)
            .append(self.close_brace.token.text());

        let prefer_fold = !self
            .open_brace
            .trailing_trivia
            .iter()
            .any(|t| t.kind == TokenKind::Newline);

        if prefer_fold {
            doc = doc.group();
        }

        doc
    }
}

// TODO: handle blank lines between nodes
impl Doc for Vec<Node> {
    fn to_doc(&self) -> RcDoc {
        RcDoc::concat(self.iter().map(|node| node.to_doc()))
    }
}

impl Doc for Node {
    fn to_doc(&self) -> RcDoc {
        let (leading_trivia, trailing_trivia) = self.surrounding_trivia();

        let doc = handle_leading_trivia(leading_trivia, false);

        let doc = doc.append(match self {
            Node::Element(element) => element.to_doc(),
            Node::Block(block) => block.to_doc(),
            Node::Str(str) => str.to_doc(),
        });

        let doc = doc.append(handle_trailing_trivia(trailing_trivia));

        doc
    }
}

impl Doc for Element {
    fn to_doc(&self) -> RcDoc {
        let doc = RcDoc::text(self.tag.token.text())
            .append(handle_trailing_trivia(&self.tag.trailing_trivia))
            .group();

        let doc = doc.append(self.body.to_doc());

        doc
    }
}

impl Doc for ElementBody {
    fn to_doc(&self) -> RcDoc {
        match self {
            ElementBody::Block(block) => {
                let doc = handle_leading_trivia(&block.open_brace.leading_trivia, false);
                doc.append(block.to_doc())
            }
        }
    }
}

impl Doc for Str {
    fn to_doc(&self) -> RcDoc {
        RcDoc::text(self.0.token.text())
    }
}

fn handle_leading_trivia(leading_trivia: &[Token], ending_newline: bool) -> RcDoc {
    let mut doc = RcDoc::nil();

    let mut trivia_iter = leading_trivia
        .iter()
        .filter(|t| t.kind != TokenKind::Whitespace)
        .peekable();

    while let Some(token) = trivia_iter.next() {
        let comment_doc = match token.kind {
            TokenKind::LineComment => RcDoc::text(token.text()).append(RcDoc::hardline()),
            TokenKind::BlockComment => {
                let text = RcDoc::text(token.text());

                let line = match trivia_iter.peek().map(|t| t.kind) {
                    Some(TokenKind::Newline) | None if ending_newline => RcDoc::line(),
                    _ => RcDoc::softline(),
                };

                text.append(line)
            }
            _ => continue,
        };

        doc = doc.append(comment_doc);
    }

    doc
}

fn handle_trailing_trivia(trailing_trivia: &[Token]) -> RcDoc {
    let trailing_comments = trailing_trivia
        .iter()
        .filter(|t| t.kind.is_comment())
        .collect::<Vec<_>>();

    let doc = RcDoc::concat(
        trailing_comments
            .iter()
            .flat_map(|t| vec![RcDoc::space(), RcDoc::text(t.text())]),
    );

    let has_line_comment = trailing_comments
        .iter()
        .any(|t| t.kind == TokenKind::LineComment);

    let doc = doc.append(if has_line_comment {
        RcDoc::hardline()
    } else {
        RcDoc::line()
    });

    doc
}
