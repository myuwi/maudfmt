use nom::error::ErrorKind;

#[derive(Clone, Debug, PartialEq)]
pub struct ParserError<I> {
    pub errors: Vec<(I, ParserErrorKind)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParserErrorKind {
    Char(char),
    Nom(ErrorKind),
}

pub trait Offset {
    fn offset(&self, other: &Self) -> isize;
}

impl Offset for &str {
    fn offset(&self, other: &Self) -> isize {
        self.as_ptr() as isize - other.as_ptr() as isize
    }
}

impl<I> nom::error::ParseError<I> for ParserError<I>
where
    I: Offset,
{
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        ParserError {
            errors: vec![(input, ParserErrorKind::Nom(kind))],
        }
    }

    fn append(input: I, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.errors.push((input, ParserErrorKind::Nom(kind)));
        other
    }

    fn from_char(input: I, c: char) -> Self {
        ParserError {
            errors: vec![(input, ParserErrorKind::Char(c))],
        }
    }

    fn or(self, other: Self) -> Self {
        let self_first = self.errors.first();
        let other_first = other.errors.first();

        match (self_first, other_first) {
            (_, None) => self,
            (None, _) => other,
            (Some(s), Some(o)) => {
                let diff = s.0.offset(&o.0);

                if diff > 0 {
                    self
                } else {
                    other
                }
            }
        }
    }
}
