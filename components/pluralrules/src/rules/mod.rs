//! A single Plural Rule is an expression which tests the value of [`PluralOperands`]
//! against a condition. If the condition is truthful, then the [`PluralCategory`]
//! to which the Rule is assigned should be used.
//!
//! # Examples
//!
//! In this example we're going to examine the AST, parsing and resolving of a
//! set of English Cardinal Plural Rules.
//!
//! A CLDR JSON source contains the following entry:
//!
//! ```json
//! {
//!   "one": "i = 1 and v = 0 @integer 1",
//!   "other": " @integer 0, 2~16, 100, 1000, 10000, 100000, 1000000, … @decimal 0.0~1.5, 10.0, 100.0, 1000.0, 10000.0, 100000.0, 1000000.0, …"
//! }
//! ```
//!
//! When the user provides a number for which the [`PluralCategory`] is to be selected,
//! the system will examin a rule for each category in order, and stop on the first
//! category which matches.
//!
//! In our example, the user provided an input value `1`.
//! That value expanded into [`PluralOperands`] looks like this:
//!
//! ```
//! use icu_pluralrules::PluralOperands;
//! PluralOperands {
//!     i: 1,
//!     v: 0,
//!     w: 0,
//!     f: 0,
//!     t: 0
//! };
//! ```
//!
//! Now, the system will parse the first rule, assigned to [`PluralCategory::One`], and
//! test if it matches.
//!
//! The value of the rule is:
//!
//! ```text
//! i = 1 and v = 0 @integer 1
//! ```
//!
//! The [`Rule`] contains a [`Condition`] `i = 1 and v = 0` and a [`Sample`] `@integer 1`.
//!
//! When parsed, the resulting [`AST`] will look like this:
//!
//! ```
//! use smallvec::{SmallVec,smallvec};
//! use icu_pluralrules::rules::parse_condition;
//! use icu_pluralrules::rules::ast::*;
//!
//! let input = "i = 1 and v = 0 @integer 1";
//!
//! let ast = parse_condition(input.as_bytes())
//!     .expect("Parsing failed.");
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
//!         },
//!         Relation {
//!             expression: Expression {
//!                 operand: Operand::V,
//!                 modulus: None,
//!             },
//!             operator: Operator::Eq,
//!             range_list: RangeList(smallvec![
//!                 RangeListItem::Value(
//!                     Value(0)
//!                 )
//!             ])
//!         }
//!     ]),
//! ]));
//! ```
//!
//! Finally, we can pass this [`AST`] (in fact, just the [`Condition`] node),
//! to a resolver alongside the [`PluralOperands`] to test if the Rule
//! matches:
//!
//! ```
//! use icu_pluralrules::rules::{test_condition, parse_condition};
//! use icu_pluralrules::PluralOperands;
//!
//! let input = "i = 1 and v = 0 @integer 1";
//!
//! let operands = PluralOperands::from(1_u32);
//!
//! let ast = parse_condition(input.as_bytes())
//!     .expect("Parsing failed.");
//!
//! assert!(test_condition(&ast, &operands));
//! ```
//!
//! Since the rule for [`PluralCategory::One`] matches, we will return this category.
//! Otherwise, we'd test the next rule, in this case [`PluralCategory::Other`], which has an
//! empty [`Condition`], meaning that it'll match all operands.
//!
//! # Summary
//!
//! For [`PluralRuleType::Cardinal`] in English, we can summarize the logic as:
//!
//! If [`PluralOperands::i`] is `1` and [`PluralOperands::v`] is `0`, [`PluralCategory::One`]
//! should be used, otherwise [`PluralCategory::Other`] should be used.
//!
//! For other locales, there are more [`PluralCategories`] and more complicated [`Rules`].
//!
//! # Difference between Category and Number
//!
//! While in English [`PluralCategory::One`] overlaps with an integer value `1`,
//! in other languages, the category may be used for other numbers as well.
//!
//! For example, in Russian [`PluralCategory::One`] matches numbers such as `11`, `21`, `121` etc.
//!
//! [`PluralCategory`]: ../enum.PluralCategory.html
//! [`PluralCategories`]: ../enum.PluralCategory.html
//! [`PluralCategory::One`]: ../enum.PluralCategory.html#variant.One
//! [`PluralCategory::Other`]: ../enum.PluralCategory.html#variant.Other
//! [`PluralOperands`]: ../struct.PluralOperands.html
//! [`PluralOperands::i`]: ../struct.PluralOperands.html#structfield.i
//! [`PluralOperands::v`]: ../struct.PluralOperands.html#structfield.v
//! [`PluralRuleType::Cardinal`]: ../enum.PluralRuleType.html#variant.Cardinal
//! [`Rule`]: ../rules/ast/struct.Rule.html
//! [`Rules`]: ../rules/ast/struct.Rule.html
//! [`Condition`]: ../rules/ast/struct.Condition.html
//! [`Sample`]: ../rules/ast/struct.Sample.html
//! [`AST`]: ../rules/ast/index.html
pub mod ast;
pub(crate) mod lexer;
pub(crate) mod parser;
pub(crate) mod resolver;
pub(crate) mod serializer;

pub use lexer::Lexer;
pub use parser::{parse, parse_condition};
pub use resolver::test_condition;
pub use serializer::serialize;
