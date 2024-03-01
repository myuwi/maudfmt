use std::io::{self, Read};

use syn::{parse_file, spanned::Spanned, visit::Visit, Macro};

// TODO: Check that the html macro is actually from maud and not some other html library?

struct MacroVisitor {
    macro_lines: Vec<(usize, usize)>,
}

impl<'ast> Visit<'ast> for MacroVisitor {
    fn visit_macro(&mut self, macro_item: &'ast Macro) {
        if macro_item.path.get_ident().is_some_and(|n| n == "html") {
            let start = macro_item.span().start().line;
            let end = macro_item.span().end().line;
            self.macro_lines.push((start, end - 1));
        }
    }
}

const INDENT_AMOUNT: usize = 4;

fn format_code(input: &str, ranges: Vec<(usize, usize)>) -> String {
    let mut out = String::new();
    let mut indent_level: usize = 0;

    for (line_number, line) in input.lines().enumerate() {
        let maybe_range = ranges
            .iter()
            .find(|r| line_number >= r.0 - 1 && line_number < r.1);

        if let Some(range) = maybe_range {
            if range.0 - 1 == line_number {
                let whitespace: usize = line
                    .chars()
                    .take_while(|ch| ch.is_whitespace() && *ch != '\n')
                    .map(|ch| ch.len_utf8())
                    .sum();

                indent_level = whitespace / INDENT_AMOUNT + 1;

                out.push_str(line);
                out.push('\n');
                continue;
            }

            let trimmed_line = line.trim();

            if trimmed_line.starts_with('}') {
                indent_level = indent_level.saturating_sub(1);
            }

            out.push_str(&" ".repeat(indent_level * INDENT_AMOUNT));
            out.push_str(trimmed_line);
            out.push('\n');

            if trimmed_line.ends_with('{') {
                indent_level += 1;
            }
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }

    out
}

fn main() {
    let mut code = String::new();

    match io::stdin().read_to_string(&mut code) {
        Ok(_) => {
            let syntax_tree = parse_file(&code).unwrap();

            let mut visitor = MacroVisitor {
                macro_lines: Vec::new(),
            };

            visitor.visit_file(&syntax_tree);

            dbg!(&visitor.macro_lines);

            let formatted_code = format_code(&code, visitor.macro_lines);
            print!("{}", formatted_code);
        }
        Err(err) => eprintln!("Error reading input: {}", err),
    }
}
