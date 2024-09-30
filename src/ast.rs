use crate::token::TokenWithTrivia;

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

#[derive(Debug)]
pub struct Element {
    pub tag: TokenWithTrivia,
    pub body: ElementBody,
}

#[derive(Debug)]
pub enum ElementBody {
    Block(Block),
}

#[derive(Debug)]
pub struct Str(pub TokenWithTrivia);
