use pretty::RcDoc;

use crate::{
    ast::{Block, Element, ElementBody, Markup, Node, Str},
    kind::TokenKind,
};

const INDENT: isize = 4;

pub trait Doc {
    fn to_doc(&self) -> RcDoc<'_>;
}

impl Doc for Markup {
    fn to_doc(&self) -> RcDoc<'_> {
        self.root.to_doc()
    }
}

impl Doc for Block {
    fn to_doc(&self) -> RcDoc<'_> {
        let mut doc = RcDoc::nil()
            .append(self.open_brace.token.text())
            .append(RcDoc::line())
            .append(RcDoc::intersperse(
                self.nodes.iter().map(|node| node.to_doc()),
                RcDoc::line(),
            ))
            .nest(INDENT)
            .append(RcDoc::line())
            .append(self.close_brace.token.text());

        let prefer_fold = !self
            .open_brace
            .trailing_trivia
            .iter()
            .any(|t| t.kind == TokenKind::Newline);

        // TODO: Is this consistent?
        if prefer_fold {
            doc = doc.group();
        }

        doc
    }
}

impl Doc for Node {
    fn to_doc(&self) -> RcDoc<'_> {
        match self {
            Node::Element(element) => element.to_doc(),
            Node::Block(block) => block.to_doc(),
            Node::Str(str) => str.to_doc(),
        }
    }
}

impl Doc for Element {
    fn to_doc(&self) -> RcDoc<'_> {
        RcDoc::text(self.tag.token.text())
            .append(RcDoc::space())
            .append(self.body.to_doc())
    }
}

impl Doc for ElementBody {
    fn to_doc(&self) -> RcDoc<'_> {
        match self {
            ElementBody::Block(block) => block.to_doc(),
        }
    }
}

impl Doc for Str {
    fn to_doc(&self) -> RcDoc<'_> {
        RcDoc::text(self.0.token.text())
    }
}
