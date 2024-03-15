use crate::parser::{
    AttributeValue, Block, ControlStructure, Element, ElementBody, Else, For, If, Let, Markup,
    Match, MatchArm, Node, Splice,
};

fn format_match_arm(out: &mut String, match_arm: &MatchArm, depth: usize, inline: bool) {
    out.push_str(&" ".repeat(depth * 4));
    out.push_str(match_arm.pattern);
    out.push_str(" => ");
    format_node(out, &match_arm.body, depth, inline);
}

fn format_match_arms(out: &mut String, match_arms: &Vec<MatchArm>, depth: usize, inline: bool) {
    for match_arm in match_arms {
        format_match_arm(out, match_arm, depth + 1, inline);

        if !matches!(match_arm.body, Node::Block(_)) {
            out.push(',');
        }

        out.push('\n');
    }
}

fn format_match(out: &mut String, match_: &Match, depth: usize, inline: bool) {
    out.push_str("match ");
    out.push_str(match_.scrut);
    out.push_str(" {\n");

    format_match_arms(out, &match_.arms, depth, inline);

    out.push_str(&" ".repeat(depth * 4));
    out.push('}');
}

fn format_let(out: &mut String, let_: &Let, _depth: usize, _inline: bool) {
    out.push_str("let ");
    out.push_str(let_.expr);
    out.push(';');
}

fn format_for(out: &mut String, for_: &For, depth: usize, inline: bool) {
    out.push_str("for ");
    out.push_str(for_.pattern);
    out.push_str(" in ");
    out.push_str(for_.expr);
    out.push(' ');

    format_block(out, &for_.body, depth, inline);
}

fn format_else(out: &mut String, else_: &Else, depth: usize, inline: bool) {
    out.push_str("@else ");

    match else_ {
        Else::If(if_) => format_if(out, if_, depth, inline),
        Else::Then(block) => format_block(out, block, depth, inline),
    }
}

fn format_if(out: &mut String, if_: &If, depth: usize, inline: bool) {
    out.push_str("if ");
    out.push_str(if_.cond);
    out.push(' ');

    format_block(out, &if_.body, depth, inline);

    if let Some(ref else_) = if_.else_clause {
        out.push(' ');
        format_else(out, else_, depth, inline)
    }
}

fn format_string(out: &mut String, string: &str) {
    out.push_str(&format!("\"{}\"", string));
}

fn format_splice(out: &mut String, splice: &Splice) {
    out.push_str(&format!("({})", splice.expr));
}

fn format_block(out: &mut String, block: &Block, depth: usize, inline: bool) {
    out.push('{');

    if !block.nodes.is_empty() {
        let inline = !block.newline || inline;
        format_nodes(out, &block.nodes, depth + 1, inline);

        if !inline {
            out.push_str(&" ".repeat(depth * 4));
        }
    }

    out.push('}');
}

fn format_element(out: &mut String, element: &Element, depth: usize, inline: bool) {
    out.push_str(element.name);

    for attr in &element.attrs {
        out.push(' ');
        out.push_str(attr.name);

        if let AttributeValue::String(s) = &attr.value {
            out.push('=');
            format_string(out, s)
        }
    }

    match &element.body {
        ElementBody::Void => out.push(';'),
        ElementBody::Block(b) => {
            out.push(' ');
            format_block(out, b, depth, inline);
        }
    }
}

fn format_node(out: &mut String, node: &Node, depth: usize, inline: bool) {
    match node {
        Node::Element(e) => format_element(out, e, depth, inline),
        Node::Block(b) => format_block(out, b, depth, inline),
        Node::StrLit(s) => format_string(out, s),
        Node::Splice(s) => format_splice(out, s),
        Node::ControlStructure(s) => {
            out.push('@');
            match s {
                ControlStructure::If(i) => format_if(out, i, depth, inline),
                ControlStructure::For(f) => format_for(out, f, depth, inline),
                ControlStructure::Let(l) => format_let(out, l, depth, inline),
                ControlStructure::Match(m) => format_match(out, m, depth, inline),
            }
        }
    }
}

fn format_nodes(out: &mut String, nodes: &Vec<Node>, depth: usize, inline: bool) {
    for node in nodes {
        if !out.is_empty() {
            if inline {
                out.push(' ');
            } else {
                out.push('\n');
                out.push_str(&" ".repeat(depth * 4));
            }
        }

        format_node(out, node, depth, inline);
    }

    if inline {
        out.push(' ');
    } else {
        out.push('\n');
    }
}

pub fn format(markup: Markup, depth: usize) -> String {
    let mut out = String::new();

    format_nodes(&mut out, &markup.nodes, depth, false);

    out[..out.len() - 1].to_string()
}
