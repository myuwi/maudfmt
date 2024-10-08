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

    #[test]
    fn too_many_comments() {
        let input = r#"{ // line 1
// line 2
/* block 1 */h1 /* block 2 */ // line 3
            /* block 3 */ {  // line 4
                /* block 4 */ "Hello world" // line 5
            /* block 5 */            /* block 6 */
            // line 6
            } // line 7
        /* block 7 */        /* block 8 */
        /* block 9 */ }"#;

        let markup = Parser::new(input, 0).parse().unwrap();
        let formatted = pretty_print(&markup, 0, 100);

        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn comments_fold() {
        let input = r#"{ /* 1 */ h1
            /* 2 */
             {          
                 /* 3 */
                 "Hello world" /* 4 */
                 /* 5 */
            }
            /* 6 */
        }"#;

        let markup = Parser::new(input, 0).parse().unwrap();
        let formatted = pretty_print(&markup, 0, 100);

        insta::assert_snapshot!(formatted);
    }
}
