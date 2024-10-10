use std::{
    io::{self, Read},
    ops::Range,
};

use miette::{miette, NamedSource, Report};
use syn::{parse_file, spanned::Spanned, visit::Visit, Macro};

use maudfmt::{format_with_indent, ParseError};

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

// Lexer calculates spans from the start of each macro body, adjust the
// error spans so they are offset from the start of the whole input file.
fn adjust_error_spans(e: ParseError, offset: usize) -> ParseError {
    let adjust_span = |s: Range<usize>| (s.start + offset)..(s.end + offset);

    match e {
        ParseError::UnexpectedToken { span } => ParseError::UnexpectedToken {
            span: adjust_span(span),
        },
        e => e,
    }
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
        let formatted = format_with_indent(content, indent, 100)
            .map_err(|e| adjust_error_spans(e, range.start))
            .map_err(Report::new)
            .map_err(|e| e.with_source_code(NamedSource::new("stdin", input.to_string())))?;

        out.replace_range(range.clone(), &format!(" {}", formatted.trim()));
    }

    Ok(out)
}

fn format_file(code: &str) -> Result<String, Report> {
    let ranges = get_macro_ranges(code)?;
    format_code(code, &ranges)
}

fn read_stdin() -> Result<String, io::Error> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

fn main() -> Result<(), Report> {
    let input = read_stdin().map_err(|e| miette!("Unable to read input from stdin: {}", e))?;
    let formatted = format_file(&input)?;
    print!("{}", formatted);
    Ok(())
}
