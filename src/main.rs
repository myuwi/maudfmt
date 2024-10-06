use std::{
    io::{self, Read},
    ops::Range,
};

use miette::{miette, NamedSource, Report};
use syn::{parse_file, spanned::Spanned, visit::Visit, Macro};

mod ast;
mod doc;
mod error;
mod format;
mod kind;
mod lexer;
mod parser;
mod span;
mod token;

use format::pretty_print;
use parser::Parser;

struct MacroVisitor {
    ranges: Vec<Range<usize>>,
}

impl<'ast> Visit<'ast> for MacroVisitor {
    fn visit_macro(&mut self, macro_item: &'ast Macro) {
        if let Some(ident) = macro_item.path.segments.span().source_text() {
            let macro_names = ["html".to_string(), "maud::html".to_string()];

            if macro_names.contains(&ident) {
                let range = macro_item.span().byte_range();

                self.ranges.push((range.start + ident.len() + 1)..range.end);
            }
        }
    }
}

fn get_macro_ranges(code: &str) -> Result<Vec<Range<usize>>, Report> {
    let ast = parse_file(code).map_err(|e| miette!("{}", e))?;
    let mut visitor = MacroVisitor { ranges: Vec::new() };
    visitor.visit_file(&ast);

    Ok(visitor.ranges)
}

fn format_code(input: &str, ranges: &[Range<usize>]) -> Result<String, Report> {
    let mut out = input.to_string();

    for range in ranges.iter().rev() {
        let indent = input[..range.start]
            .rsplit_once('\n')
            .map(|(_, s)| {
                s.chars()
                    .take_while(|ch| ch.is_whitespace() && *ch != '\n')
                    .fold(0, |acc, c| acc + c.len_utf8())
            })
            .unwrap_or(0);

        let content = &input[range.clone()];
        let markup = Parser::new(content, range.start)
            .parse()
            .map_err(Report::new)
            .map_err(|e| e.with_source_code(NamedSource::new("stdin", input.to_string())))?;
        let pretty = pretty_print(&markup, indent, 100);

        out.replace_range(range.clone(), &format!(" {}", pretty.trim()));
    }

    Ok(out)
}

fn main() -> Result<(), Report> {
    let mut code = String::new();

    io::stdin()
        .read_to_string(&mut code)
        .map_err(|e| miette!("Error reading input: {}", e))?;

    let ranges = get_macro_ranges(&code)?;
    let code = format_code(&code, &ranges)?;
    print!("{}", code);

    Ok(())
}
