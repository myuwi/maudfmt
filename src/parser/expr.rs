use nom::{
    error::{ErrorKind, VerboseError},
    InputTake,
};

use super::{
    literal::{char_lit, str_lit},
    NomResult,
};

// TODO: rewrite with nom combinators
pub fn expr(input: &str) -> NomResult<&str> {
    use nom::error::ParseError;
    let mut stack: Vec<char> = vec![];

    let mut it = input.chars().enumerate();
    while let Some((i, c)) = it.next() {
        match c {
            // Handle strings inside expressions
            '"' | '\'' => {
                let parser = if c == '"' { str_lit } else { char_lit };

                let (_, str) = parser(&input[i..])?;
                let char_count = str.chars().count();

                it.nth(char_count);
                continue;
            }

            '(' | '[' => stack.push(c),
            '{' => {
                if stack.is_empty() {
                    let (i, o) = input.take_split(i);
                    return Ok((i, o.trim()));
                }
                stack.push(c)
            }
            '|' => {
                if let Some('|') = stack.last() {
                    stack.pop();
                } else {
                    stack.push(c);
                }
            }
            ')' | '}' | ']' => {
                if let Some(popped) = stack.pop() {
                    match popped {
                        '(' if c == ')' => (),
                        '{' if c == '}' => (),
                        '[' if c == ']' => (),
                        // Return error if braces are invalid
                        _ => {
                            let (ctx, _) = input.take_split(i);
                            return Err(nom::Err::Error(VerboseError::from_char(ctx, c)));
                        }
                    }
                } else {
                    let (i, o) = input.take_split(i);
                    return Ok((i, o.trim()));
                }
            }
            _ => (),
        }
    }

    Err(nom::Err::Error(VerboseError::from_error_kind(
        input,
        ErrorKind::TakeUntil,
    )))
}
