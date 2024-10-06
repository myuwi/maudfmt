use pretty::RcDoc;

use crate::ast::Markup;
use crate::doc::Doc;

pub fn pretty_print(markup: &Markup, indent: usize, width: usize) -> String {
    let doc = markup.to_doc();
    let doc = RcDoc::text(" ".repeat(indent))
        .append(doc)
        .nest(indent as isize);

    doc.pretty(width).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn basic() {
        let input = r#"{
h1 {         "Hello "
   b {
       "world"}
}p {
"Lorem ipsum"
}
        }"#;

        let markup = Parser::new(input, 0).parse().unwrap();
        let formatted = pretty_print(&markup, 0, 100);

        insta::assert_snapshot!(formatted);
    }
}
