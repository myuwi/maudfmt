use crate::parser::{AttributeValue, Block, Element, ElementBody, Markup, Node};

fn format_block(out: &mut String, block: &Block, depth: usize, inline: bool) {
    let inline = !block.newline || inline;

    out.push('{');

    if !block.nodes.is_empty() {
        format_nodes(out, &block.nodes, depth + 1, inline);

        if !inline {
            out.push_str(&" ".repeat(depth * 4));
        }
    }

    out.push('}');
}

fn format_element(out: &mut String, element: &Element, depth: usize, inline: bool) {
    out.push_str(&element.name);

    for attr in &element.attrs {
        out.push(' ');
        out.push_str(&attr.name);

        if let AttributeValue::String(s) = &attr.value {
            out.push('=');
            out.push_str(&format!(r#""{}""#, s));
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

        match &node {
            Node::Str(s) => out.push_str(&format!(r#""{}""#, s)),
            Node::Element(e) => format_element(out, e, depth, inline),
            Node::Block(b) => format_block(out, b, depth, inline),
        }
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
