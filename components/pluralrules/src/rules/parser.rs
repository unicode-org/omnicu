use super::ast;
use super::lexer::{Lexer, Token};
use smallvec::smallvec;
use std::iter::Peekable;

#[derive(Debug, PartialEq, Eq)]
pub enum ParserError {
    ExpectedAndCondition,
    ExpectedRelation,
    ExpectedOperator,
    ExpectedOperand,
    ExpectedValue,
    ExpectedSampleType,
}

/// Unicode Plural Rule parser converts an
/// input string into a Rule [`AST`].
///
/// A single [`Rule`] contains a [`Condition`] and optionally a set of
/// [`Samples`].
///
/// A [`Condition`] can be then used by the [`resolver`] to test
/// against [`PluralOperands`], to find the appropriate [`PluralCategory`].
///
/// [`Samples`] are useful for tooling to help translators understand examples of numbers
/// covered by the given [`Rule`].
///
/// At runtime, only the [`Condition`] is used and for that, consider using [`parse_condition`].
///
/// # Examples
///
/// ```
/// use icu_pluralrules::rules::parse;
///
/// let input = b"i = 0 or n = 1 @integer 0, 1 @decimal 0.0~1.0, 0.00~0.04";
/// assert_eq!(parse(input).is_ok(), true);
/// ```
///
/// [`AST`]: ../rules/ast/index.html
/// [`resolver`]: ../rules/resolver/index.html
/// [`PluralOperands`]: ../struct.PluralOperands.html
/// [`PluralCategory`]: ../enum.PluralCategory.html
/// [`Rule`]: ../rules/ast/struct.Rule.html
/// [`Samples`]: ../rules/ast/struct.Samples.html
/// [`Condition`]:  ../rules/ast/struct.Condition.html
/// [`parse_condition`]: ./fn.parse_condition.html
pub fn parse(input: &[u8]) -> Result<ast::Rule, ParserError> {
    let parser = Parser::new(input);
    parser.parse()
}

/// Unicode Plural Rule parser converts an
/// input string into an [`AST`].
///
/// That [`AST`] can be then used by the [`resolver`] to test
/// against [`PluralOperands`], to find the appropriate [`PluralCategory`].
///
/// # Examples
///
/// ```
/// use icu_pluralrules::rules::parse_condition;
///
/// let input = b"i = 0 or n = 1";
/// assert_eq!(parse_condition(input).is_ok(), true);
/// ```
///
/// [`AST`]: ../rules/ast/index.html
/// [`resolver`]: ../rules/resolver/index.html
/// [`PluralOperands`]: ../struct.PluralOperands.html
/// [`PluralCategory`]: ../enum.PluralCategory.html
pub fn parse_condition(input: &[u8]) -> Result<ast::Condition, ParserError> {
    let parser = Parser::new(input);
    parser.parse_condition()
}

struct Parser<'p> {
    lexer: Peekable<Lexer<'p>>,
}

impl<'p> Parser<'p> {
    fn new(input: &'p [u8]) -> Self {
        Self {
            lexer: Lexer::new(input).peekable(),
        }
    }

    pub fn parse(mut self) -> Result<ast::Rule, ParserError> {
        self.get_rule()
    }

    pub fn parse_condition(mut self) -> Result<ast::Condition, ParserError> {
        self.get_condition()
    }

    fn get_rule(&mut self) -> Result<ast::Rule, ParserError> {
        Ok(ast::Rule {
            condition: self.get_condition()?,
            samples: self.get_samples()?,
        })
    }

    fn get_condition(&mut self) -> Result<ast::Condition, ParserError> {
        let mut result = smallvec![];

        if let Some(cond) = self.get_and_condition()? {
            result.push(cond);
        } else {
            return Ok(ast::Condition(result));
        }

        while self.take_if(Token::Or) {
            if let Some(cond) = self.get_and_condition()? {
                result.push(cond);
            } else {
                return Err(ParserError::ExpectedAndCondition);
            }
        }
        // If lexer is not done, error?
        Ok(ast::Condition(result))
    }

    fn get_and_condition(&mut self) -> Result<Option<ast::AndCondition>, ParserError> {
        if let Some(relation) = self.get_relation()? {
            let mut rel = smallvec![relation];

            while self.take_if(Token::And) {
                if let Some(relation) = self.get_relation()? {
                    rel.push(relation);
                } else {
                    return Err(ParserError::ExpectedRelation);
                }
            }
            Ok(Some(ast::AndCondition(rel)))
        } else {
            Ok(None)
        }
    }

    fn get_relation(&mut self) -> Result<Option<ast::Relation>, ParserError> {
        if let Some(expression) = self.get_expression()? {
            let operator = match self.lexer.next() {
                Some(Token::Operator(op)) => op,
                _ => return Err(ParserError::ExpectedOperator),
            };
            let range_list = self.get_range_list()?;
            Ok(Some(ast::Relation {
                expression,
                operator,
                range_list,
            }))
        } else {
            Ok(None)
        }
    }

    fn get_expression(&mut self) -> Result<Option<ast::Expression>, ParserError> {
        let operand = match self.lexer.peek() {
            Some(Token::Operand(op)) => *op,
            Some(Token::At) | None => return Ok(None),
            _ => return Err(ParserError::ExpectedOperand),
        };
        self.lexer.next();
        let modulus = if self.take_if(Token::Modulo) {
            Some(self.get_value()?)
        } else {
            None
        };
        Ok(Some(ast::Expression { operand, modulus }))
    }

    fn get_range_list(&mut self) -> Result<ast::RangeList, ParserError> {
        let mut range_list = smallvec![];
        loop {
            range_list.push(self.get_range_list_item()?);
            if !self.take_if(Token::Comma) {
                break;
            }
        }
        Ok(ast::RangeList(range_list))
    }

    fn take_if(&mut self, token: Token) -> bool {
        if self.lexer.peek() == Some(&token) {
            self.lexer.next();
            true
        } else {
            false
        }
    }

    fn get_range_list_item(&mut self) -> Result<ast::RangeListItem, ParserError> {
        let value = self.get_value()?;
        if self.take_if(Token::DotDot) {
            let value2 = self.get_value()?;
            Ok(ast::RangeListItem::Range(value..=value2))
        } else {
            Ok(ast::RangeListItem::Value(value))
        }
    }

    fn get_value(&mut self) -> Result<ast::Value, ParserError> {
        match self.lexer.next() {
            Some(Token::Number(v)) => Ok(ast::Value(v as u64)),
            Some(Token::Zero) => Ok(ast::Value(0)),
            _ => Err(ParserError::ExpectedValue),
        }
    }

    fn get_samples(&mut self) -> Result<Option<ast::Samples>, ParserError> {
        let mut integer = None;
        let mut decimal = None;

        while self.take_if(Token::At) {
            match self.lexer.next() {
                Some(Token::Integer) => integer = Some(self.get_sample_list()?),
                Some(Token::Decimal) => decimal = Some(self.get_sample_list()?),
                _ => return Err(ParserError::ExpectedSampleType),
            };
        }
        if integer.is_some() || decimal.is_some() {
            Ok(Some(ast::Samples { integer, decimal }))
        } else {
            Ok(None)
        }
    }

    fn get_sample_list(&mut self) -> Result<ast::SampleList, ParserError> {
        let mut ranges = vec![self.get_sample_range()?];
        let mut ellipsis = false;

        while self.take_if(Token::Comma) {
            if self.take_if(Token::Ellipsis) {
                ellipsis = true;
                break;
            }
            ranges.push(self.get_sample_range()?);
        }
        Ok(ast::SampleList {
            sample_ranges: ranges.into_boxed_slice(),
            ellipsis,
        })
    }

    fn get_sample_range(&mut self) -> Result<ast::SampleRange, ParserError> {
        let lower_val = self.get_decimal_value()?;
        let upper_val = if self.take_if(Token::Tilde) {
            Some(self.get_decimal_value()?)
        } else {
            None
        };
        Ok(ast::SampleRange {
            lower_val,
            upper_val,
        })
    }

    fn get_decimal_value(&mut self) -> Result<ast::DecimalValue, ParserError> {
        let mut s = String::new();
        loop {
            match self.lexer.peek() {
                Some(Token::Zero) => s.push('0'),
                Some(Token::Number(v)) => {
                    s.push_str(&v.to_string());
                }
                _ => {
                    break;
                }
            }
            self.lexer.next();
        }
        if self.take_if(Token::Dot) {
            s.push('.');
            loop {
                match self.lexer.peek() {
                    Some(Token::Zero) => s.push('0'),
                    Some(Token::Number(v)) => {
                        s.push_str(&v.to_string());
                    }
                    _ => {
                        break;
                    }
                }
                self.lexer.next();
            }
        }
        if s.is_empty() {
            Err(ParserError::ExpectedValue)
        } else {
            Ok(ast::DecimalValue(s))
        }
    }
}
