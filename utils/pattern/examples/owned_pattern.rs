use icu_pattern::{Interpolator, Parser};
use std::{
    convert::TryInto,
    fmt::{Display, Write},
};

#[derive(Debug)]
enum Element<'s> {
    Token(usize),
    Literal(&'s str),
}

impl Display for Element<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Token(n) => write!(f, "{}", n),
            Self::Literal(s) => f.write_str(s),
        }
    }
}

impl<'s> From<&'s str> for Element<'s> {
    fn from(input: &'s str) -> Self {
        Self::Literal(input)
    }
}

fn main() {
    let replacements = vec![Some(Element::Token(5))];

    let pattern: Vec<_> = Parser::new("{0} days", true)
        .try_into()
        .expect("Failed to parse a pattern.");

    let mut interpolator = Interpolator::new(&pattern, replacements);

    let mut result = String::new();

    while let Some(element) = interpolator.try_next().expect("Failed to interpolate") {
        write!(result, "{}", element).expect("Failed to write to a string");
    }
    assert_eq!(result, "5 days");
}
