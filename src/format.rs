use pretty::RcDoc;

use crate::{
    doc::Doc,
    parser::{parse, ParseResult},
};

pub fn format_input(input: &str, width: usize) -> ParseResult<String> {
    format_with_indent(input, 0, width)
}

pub fn format_with_indent(input: &str, indent: usize, width: usize) -> ParseResult<String> {
    let markup = parse(input)?;
    let mut doc = markup.to_doc();

    if indent > 0 {
        doc = RcDoc::text(" ".repeat(indent))
            .append(doc)
            .nest(indent as isize);
    }

    let formatted = format_doc(doc, width);

    Ok(formatted)
}

fn format_doc(doc: RcDoc, width: usize) -> String {
    trim_trailing_whitespace(&doc.pretty(width).to_string())
}

fn trim_trailing_whitespace(s: &str) -> String {
    s.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let formatted = format_input(input, 100).unwrap();
        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn attributes() {
        let input = r#"{
            h1
                class
                =
                "title"
                {}

            button type="button"
                class="btn btn-small btn-primary"
                x-lorem="lorem ipsum dolor sit amet, consectetur adipiscing elit" {
                    "Small" }

            input type="text" id="small-input" class="input input-small";

            input type/**/="text" id // small input
                ="small-input" // line comment
                               class="input input-small";
            }"#;

        let formatted = format_input(input, 100).unwrap();
        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn empty_lines() {
        let input = r#"{


h1 {


    "Hello world"


    "Lorem ipsum"


}


}"#;

        let formatted = format_input(input, 100).unwrap();
        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn empty_block() {
        let input = r#"{
            h1 {

            }
            p {           }
            p { /* 1 */
            /* 2 */ }
            p {  /* 3 */         }
        }"#;

        let formatted = format_input(input, 100).unwrap();
        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn void_element() {
        let input = r#"{
            input /**/ ; // a
            input /**/
// asd
                ; // b
            input
                /**/;
        }"#;

        let formatted = format_input(input, 100).unwrap();
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

        let formatted = format_input(input, 100).unwrap();
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

        let formatted = format_input(input, 100).unwrap();
        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn trailing_line_comment() {
        let input = r#"{ h1 // 2
            {// 3
                "Hello world" // 4
            } // 5
        }"#;

        let formatted = format_input(input, 100).unwrap();
        insta::assert_snapshot!(formatted);
    }
}
