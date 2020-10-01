//! `AST` provides a set of Syntax Tree Nodes used to store
//! the output of [`parse`] method that is used in [`test_condition`] method
//! to evaluate whether a given [`PluralCategory`] should be used.
//!
//! # Examples
//!
//! ```
//! use icu_pluralrules::rules::parse_condition;
//! use icu_pluralrules::rules::ast::*;
//! use smallvec::{SmallVec,smallvec};
//!
//! let input = "i = 1";
//!
//! let ast = parse_condition(input.as_bytes())
//!     .expect("Parsing failed.");
//!
//! assert_eq!(ast, Condition(smallvec![
//!     AndCondition(smallvec![
//!         Relation {
//!             expression: Expression {
//!                 operand: Operand::I,
//!                 modulus: None,
//!             },
//!             operator: Operator::Eq,
//!             range_list: RangeList(smallvec![
//!                 RangeListItem::Value(
//!                     Value(1)
//!                 )
//!             ])
//!         }
//!     ])
//! ]));
//! ```
//!
//! [`PluralCategory`]: ../enum.PluralCategory.html
//! [`parse`]: ../fn.parse.html
//! [`test_condition`]: ../fn.test_condition.html
use smallvec::SmallVec;
use std::ops::RangeInclusive;

/// A complete AST representation of a plural rule.
/// Comprises a vector of AndConditions and optionally a set of Samples.
///
/// # Examples
///
/// ```
/// use icu_pluralrules::rules::ast::*;
/// use icu_pluralrules::rules::{parse, parse_condition};
///
/// let condition = parse_condition(b"i = 5 or v = 2")
///     .expect("Parsing failed.");
///
/// let samples = Samples {
///     integer: Some(SampleList {
///         sample_ranges: Box::new([SampleRange {
///             lower_val: DecimalValue("2".to_string()),
///             upper_val: None,
///         }]),
///         ellipsis: true
///     }),
///     decimal: Some(SampleList {
///         sample_ranges: Box::new([SampleRange {
///             lower_val: DecimalValue("2.5".to_string()),
///             upper_val: None,
///         }]),
///         ellipsis: false
///     }),
/// };
///
/// let rule = Rule {
///     condition,
///     samples: Some(samples),
/// };
///
/// assert_eq!(
///     rule,
///     parse("i = 5 or v = 2 @integer 2, … @decimal 2.5".as_bytes())
///          .expect("Parsing failed")
/// )
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub condition: Condition,
    pub samples: Option<Samples>,
}

/// A complete AST representation of a plural rule's condition. Comprises a vector of AndConditions.
///
/// # Examples
///
/// ```
/// use icu_pluralrules::rules::ast::*;
/// use icu_pluralrules::rules::parse_condition;
/// use smallvec::{SmallVec,smallvec};
///
/// let condition = Condition(smallvec![
///     AndCondition(smallvec![Relation {
///         expression: Expression {
///             operand: Operand::I,
///             modulus: None,
///         },
///         operator: Operator::Eq,
///         range_list: RangeList(smallvec![RangeListItem::Value(Value(5))]),
///     }]),
///     AndCondition(smallvec![Relation {
///         expression: Expression {
///             operand: Operand::V,
///             modulus: None,
///         },
///         operator: Operator::Eq,
///         range_list: RangeList(smallvec![RangeListItem::Value(Value(2))]),
///     }]),
/// ]);
///
/// assert_eq!(
///     condition,
///     parse_condition(b"i = 5 or v = 2")
///          .expect("Parsing failed")
/// )
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Condition(pub SmallVec<[AndCondition; 2]>);

/// An incomplete AST representation of a plural rule. Comprises a vector of Relations.
///
/// # Examples
///
/// All AST nodes can be built explicitly, as seen in the example. However, due to its complexity, it is preferred to build the AST using the parse_plural_rule function.
///
/// ```text
/// "i = 3 and v = 0"
/// ```
///
/// Can be represented by the AST:
///
/// ```
/// use icu_pluralrules::rules::ast::*;
/// use smallvec::{SmallVec,smallvec};
///
/// AndCondition(smallvec![
///     Relation {
///         expression: Expression {
///             operand: Operand::I,
///             modulus: None,
///         },
///         operator: Operator::Eq,
///         range_list: RangeList(smallvec![RangeListItem::Value(Value(5))]),
///     },
///     Relation {
///         expression: Expression {
///             operand: Operand::V,
///             modulus: None,
///         },
///         operator: Operator::NotEq,
///         range_list: RangeList(smallvec![RangeListItem::Value(Value(2))]),
///     },
/// ]);
///
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct AndCondition(pub SmallVec<[Relation; 2]>);

/// An incomplete AST representation of a plural rule. Comprises an Expression, an Operator, and a RangeList.
///
/// # Examples
///
/// All AST nodes can be built explicitly, as seen in the example. However, due to its complexity, it is preferred to build the AST using the parse_plural_rule function.
///
/// ```text
/// "i = 3"
/// ```
///
/// Can be represented by the AST:
///
/// ```
/// use icu_pluralrules::rules::ast::*;
/// use smallvec::{SmallVec,smallvec};
///
/// Relation {
///     expression: Expression {
///         operand: Operand::I,
///         modulus: None,
///     },
///     operator: Operator::Eq,
///     range_list: RangeList(smallvec![RangeListItem::Value(Value(3))]),
/// };
///
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Relation {
    pub expression: Expression,
    pub operator: Operator,
    pub range_list: RangeList,
}

/// An enum of Relation operators for plural rules.
///
/// Each Operator enumeration belongs to the corresponding symbolic operators:
///
/// | Enum Operator | Symbolic Operator |
/// | - | - |
/// | Eq | "=" |
/// | NotEq | "!=" |
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    Eq,
    NotEq,
}

/// An incomplete AST representation of a plural rule. Comprises an Operand and an optional Modulo.
///
/// # Examples
///
/// All AST nodes can be built explicitly, as seen in the example. However, due to its complexity, it is preferred to build the AST using the parse_plural_rule function.
///
/// ```text
/// "i % 100"
/// ```
///
/// Can be represented by the AST:
///
/// ```
/// use icu_pluralrules::rules::ast::*;
///
/// Expression {
///     operand: Operand::I,
///     modulus: Some(Value(100)),
/// };
///
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub operand: Operand,
    pub modulus: Option<Value>,
}

/// An incomplete AST representation of a plural rule. Comprises a char.
///
/// # Examples
///
/// All AST nodes can be built explicitly, as seen in the example. However, due to its complexity, it is preferred to build the AST using the parse_plural_rule function.
///
/// ```text
/// "i"
/// ```
///
/// Can be represented by the AST:
///
/// ```
/// use icu_pluralrules::rules::ast::Operand;
///
/// Operand::I;
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operand {
    /// Absolute value of input
    N,
    /// An integer value of input with the fraction part truncated off
    I,
    /// Number of visible fraction digits with trailing zeros
    V,
    /// Number of visible fraction digits without trailing zeros
    W,
    /// Visible fraction digits with trailing zeros
    F,
    /// Visible fraction digits without trailing zeros
    T,
}

/// An incomplete AST representation of a plural rule. Comprises a vector of RangeListItems.
///
/// # Examples
///
/// All AST nodes can be built explicitly, as seen in the example. However, due to its complexity, it is preferred to build the AST using the parse_plural_rule function.
///
/// ```text
/// "5, 7, 9"
/// ```
///
/// Can be represented by the AST:
///
/// ```
/// use icu_pluralrules::rules::ast::*;
/// use smallvec::{SmallVec,smallvec};
///
/// RangeList(smallvec![
///     RangeListItem::Value(Value(5)),
///     RangeListItem::Value(Value(7)),
///     RangeListItem::Value(Value(9)),
/// ]);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RangeList(pub SmallVec<[RangeListItem; 2]>);

/// An enum of items that appear in a RangeList: Range or a Value.
///
/// See Range and Value for additional details.
/// A range comprises two Values: an inclusive lower and upper limit.
///
/// # Examples
///
/// ```text
/// 5
/// 11..15
/// ```
///
/// Can be represented by the AST:
///
/// ```
/// use icu_pluralrules::rules::ast::*;
///
/// let _ = RangeListItem::Value(Value(5));
/// let _ = RangeListItem::Range(Value(11)..=Value(15));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum RangeListItem {
    Range(RangeInclusive<Value>),
    Value(Value),
}

/// An incomplete AST representation of a plural rule, representing one integer.
///
/// # Examples
///
/// All AST nodes can be built explicitly, as seen in the example. However, due to its complexity, it is preferred to build the AST using the parse_plural_rule function.
///
/// ```text
/// "99"
/// ```
///
/// Can be represented by the AST:
///
/// ```
/// use icu_pluralrules::rules::ast::*;
///
/// RangeListItem::Value(Value(99));
/// ```
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Value(pub u64);

/// A sample of example values that match the given rule.
///
/// # Examples
///
/// ```text
/// @integer 2, … @decimal 2.5
/// ```
///
/// ```
/// use icu_pluralrules::rules::ast::*;
/// Samples {
///     integer: Some(SampleList {
///         sample_ranges: Box::new([SampleRange {
///             lower_val: DecimalValue("2".to_string()),
///             upper_val: None,
///         }]),
///         ellipsis: true
///     }),
///     decimal: Some(SampleList {
///         sample_ranges: Box::new([SampleRange {
///             lower_val: DecimalValue("2.5".to_string()),
///             upper_val: None,
///         }]),
///         ellipsis: false
///     }),
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Samples {
    pub integer: Option<SampleList>,
    pub decimal: Option<SampleList>,
}

/// A list of values used in samples.
///
/// # Examples
///
/// ```text
/// 0.0~1.5, …
/// ```
///
/// ```
/// use icu_pluralrules::rules::ast::*;
/// use smallvec::{SmallVec,smallvec};
///
/// SampleList {
///     sample_ranges: Box::new([
///         SampleRange {
///             lower_val: DecimalValue("0.0".to_string()),
///             upper_val: Some(DecimalValue("1.5".to_string())),
///         }
///     ]),
///     ellipsis: true
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct SampleList {
    pub sample_ranges: Box<[SampleRange]>,
    pub ellipsis: bool,
}

/// A value range used in samples.
///
/// # Examples
///
/// ```text
/// 0.0~1.5
/// ```
///
/// ```
/// use icu_pluralrules::rules::ast::*;
/// SampleRange {
///     lower_val: DecimalValue("0.0".to_string()),
///     upper_val: Some(DecimalValue("1.5".to_string())),
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct SampleRange {
    pub lower_val: DecimalValue,
    pub upper_val: Option<DecimalValue>,
}

/// A decimal value used in samples.
///
/// # Examples
///
/// ```text
/// 1.00
/// ```
///
/// ```
/// use icu_pluralrules::rules::ast::*;
/// DecimalValue("1.00".to_string());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct DecimalValue(pub String);
