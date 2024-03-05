use std::{
    io::{self, Read},
    ops::Range,
};

use syn::{parse_file, spanned::Spanned, visit::Visit, Macro};

mod formatter;
mod parser;

use crate::{formatter::format, parser::parse};

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

fn format_code(input: &str, location: Vec<MacroLocation>) -> String {
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

        let indentation = whitespace / TAB_SIZE;

        let content = &out[location.byte_range.clone()];

        let markup = parse(content.trim());
        let formatted = format(markup, indentation);

        out.replace_range(location.byte_range.clone(), &format!(" {}", &formatted));
    }

    out
}

fn main() {
    let mut code = String::new();

    match io::stdin().read_to_string(&mut code) {
        Ok(_) => {
            let ast = parse_file(&code).unwrap();

            let mut visitor = MacroVisitor {
                locations: Vec::new(),
            };

            visitor.visit_file(&ast);

            let formatted_code = format_code(&code, visitor.locations);
            print!("{}", formatted_code);
        }
        Err(err) => eprintln!("Error reading input: {}", err),
    }
}
