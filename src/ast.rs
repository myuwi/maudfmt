use crate::token::{Token, TokenWithTrivia};

#[derive(Debug)]
pub struct Markup {
    pub root: Block,
}

#[derive(Debug)]
pub struct Block {
    pub open_brace: TokenWithTrivia,
    pub nodes: Vec<Node>,
    pub close_brace: TokenWithTrivia,
}

#[derive(Debug)]
pub enum Node {
    Element(Element),
    Block(Block),
    Str(Str),
}

impl Node {
    pub fn surrounding_trivia(&self) -> (&Vec<Token>, &Vec<Token>) {
        match self {
            Node::Element(element) => {
                let leading_trivia = &element.tag.leading_trivia;
                let trailing_trivia = match &element.body {
                    ElementBody::Block(block) => &block.close_brace.trailing_trivia,
                    ElementBody::Void(semi) => &semi.trailing_trivia,
                };

                (leading_trivia, trailing_trivia)
            }
            Node::Block(block) => {
                let leading_trivia = &block.open_brace.leading_trivia;
                let trailing_trivia = &block.close_brace.trailing_trivia;

                (leading_trivia, trailing_trivia)
            }
            Node::Str(str) => {
                let leading_trivia = &str.0.leading_trivia;
                let trailing_trivia = &str.0.trailing_trivia;

                (leading_trivia, trailing_trivia)
            }
        }
    }
}

#[derive(Debug)]
pub struct Element {
    pub tag: TokenWithTrivia,
    pub attrs: Vec<Attribute>,
    pub body: ElementBody,
}

#[derive(Debug)]
pub enum ElementBody {
    Block(Block),
    Void(TokenWithTrivia),
}

#[derive(Debug)]
pub struct Attribute {
    pub name: TokenWithTrivia,
    pub eq: TokenWithTrivia,
    pub value: TokenWithTrivia,
}

#[derive(Debug)]
pub struct Str(pub TokenWithTrivia);
