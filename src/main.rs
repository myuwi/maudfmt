use std::{
    io::{self, Read},
    ops::Range,
};

use miette::{miette, Report};
use syn::{parse_file, spanned::Spanned, visit::Visit, Macro};

mod error;
mod formatter;
mod kind;
mod lexer;
mod parser;

use crate::{formatter::format, parser::parse_range};

const TAB_SIZE: usize = 4;

struct MacroLocation {
    start_line: usize,
    byte_range: Range<usize>,
}

struct MacroVisitor {
    locations: Vec<MacroLocation>,
}

impl<'ast> Visit<'ast> for MacroVisitor {
    fn visit_macro(&mut self, macro_item: &'ast Macro) {
        if let Some(ident) = macro_item.path.segments.span().source_text() {
            if ident.ends_with("html") {
                let span = macro_item.span();
                let start = span.start().line;
                let range = span.byte_range();

                self.locations.push(MacroLocation {
                    start_line: start,
                    byte_range: (range.start + ident.len() + 1)..range.end,
                });
            }
        }
    }
}

fn format_code(input: &str, location: Vec<MacroLocation>) -> Result<String, Report> {
    let mut out = input.to_string();

    for location in location.iter().rev() {
        let whitespace: usize = out
            .lines()
            .nth(location.start_line - 1)
            .unwrap()
            .chars()
            .take_while(|ch| ch.is_whitespace() && *ch != '\n')
            .map(|ch| ch.len_utf8())
            .sum();

        let indent_level = whitespace / TAB_SIZE;

        let markup = parse_range(input, location.byte_range.clone())?;

        let formatted = format(markup, indent_level);

        out.replace_range(location.byte_range.clone(), &format!(" {}", &formatted));
    }

    Ok(out)
}

fn main() -> Result<(), Report> {
    let mut code = String::new();

    io::stdin()
        .read_to_string(&mut code)
        .map_err(|e| miette!("Error reading input: {}", e))?;

    let ast = parse_file(&code).unwrap();

    let mut visitor = MacroVisitor {
        locations: Vec::new(),
    };

    visitor.visit_file(&ast);

    let code = format_code(&code, visitor.locations)?;
    print!("{}", code);

    Ok(())
}
