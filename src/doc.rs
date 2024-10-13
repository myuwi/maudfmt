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
        let mut doc = RcDoc::nil().append(self.open_brace.token.text());

        let has_children = !self.nodes.is_empty()
            || self
                .open_brace
                .trailing_trivia
                .iter()
                .chain(self.close_brace.leading_trivia.iter())
                .any(|t| t.kind.is_comment());

        if has_children {
            doc = doc
                .append(handle_trailing_trivia(
                    &self.open_brace.trailing_trivia,
                    true,
                ))
                .append(self.nodes.to_doc())
                .append(handle_leading_trivia(
                    &self.close_brace.leading_trivia,
                    TriviaSpacing::Line,
                ))
                .nest(INDENT);
        }

        doc = doc.append(self.close_brace.token.text());

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

impl Doc for Vec<Node> {
    fn to_doc(&self) -> RcDoc {
        RcDoc::concat(self.iter().enumerate().map(|(i, node)| {
            let mut doc = node.to_doc();

            let first_node = i == 0;
            let leading_trivia = node.surrounding_trivia().0;

            let leading_empty_line = !first_node
                && leading_trivia
                    .iter()
                    .find(|t| t.kind != TokenKind::Whitespace)
                    .is_some_and(|t| t.kind == TokenKind::Newline);

            if leading_empty_line {
                doc = RcDoc::line_().append(doc);
            }

            doc
        }))
    }
}

impl Doc for Node {
    fn to_doc(&self) -> RcDoc {
        let (leading_trivia, trailing_trivia) = self.surrounding_trivia();

        let doc = handle_leading_trivia(leading_trivia, TriviaSpacing::Auto);

        let doc = doc.append(match self {
            Node::Element(element) => element.to_doc(),
            Node::Block(block) => block.to_doc(),
            Node::Str(str) => str.to_doc(),
        });

        let doc = doc.append(handle_trailing_trivia(trailing_trivia, true));

        doc
    }
}

impl Doc for Element {
    fn to_doc(&self) -> RcDoc {
        let trailing_space = match &self.body {
            ElementBody::Block(_) => true,
            ElementBody::Void(t) => t.leading_trivia.iter().any(|t| t.kind.is_comment()),
        };

        let doc = RcDoc::text(self.tag.token.text())
            .append(handle_trailing_trivia(
                &self.tag.trailing_trivia,
                trailing_space,
            ))
            .group();

        let doc = doc.append(self.body.to_doc());

        doc
    }
}

impl Doc for ElementBody {
    fn to_doc(&self) -> RcDoc {
        match self {
            ElementBody::Block(block) => {
                let doc =
                    handle_leading_trivia(&block.open_brace.leading_trivia, TriviaSpacing::Auto);
                doc.append(block.to_doc())
            }
            ElementBody::Void(token) => {
                let doc = handle_leading_trivia(&token.leading_trivia, TriviaSpacing::None);
                doc.append(token.token.text())
            }
        }
    }
}

impl Doc for Str {
    fn to_doc(&self) -> RcDoc {
        RcDoc::text(self.0.token.text())
    }
}

enum TriviaSpacing {
    Line,
    Auto,
    None,
}

fn handle_leading_trivia(leading_trivia: &[Token], spacing: TriviaSpacing) -> RcDoc {
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

                let last_comment = !trivia_iter.clone().any(|t| t.kind.is_comment());

                let trailing_newline = trivia_iter
                    .peek()
                    .map_or(false, |t| t.kind == TokenKind::Newline);

                let line = if last_comment {
                    match spacing {
                        TriviaSpacing::Line => RcDoc::line(),
                        TriviaSpacing::Auto if trailing_newline => RcDoc::line(),
                        TriviaSpacing::Auto => RcDoc::softline(),
                        TriviaSpacing::None => RcDoc::softline_(),
                    }
                } else if trailing_newline {
                    RcDoc::line()
                } else {
                    RcDoc::softline()
                };

                text.append(line)
            }
            _ => continue,
        };

        doc = doc.append(comment_doc);
    }

    doc
}

fn handle_trailing_trivia(trailing_trivia: &[Token], trailing_space: bool) -> RcDoc {
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
    } else if trailing_space {
        RcDoc::line()
    } else {
        RcDoc::line_()
    });

    doc
}
