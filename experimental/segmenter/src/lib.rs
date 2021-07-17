// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! A line breaker that is compatible with [Unicode Standard Annex #14][UAX14] and CSS properties.
//!
//! [UAX14]: http://www.unicode.org/reports/tr14/
//!
//!```rust
//! use icu_segmenter::LineBreakIterator;
//!
//! let mut iter = LineBreakIterator::new("Hello World");
//! let result: Vec<usize> = iter.collect();
//! println!("{:?}", result);
//! ```
//!
//! With CSS property.
//! ```rust
//! use icu_segmenter::{LineBreakIterator, LineBreakRule, WordBreakRule};
//!
//! let iter = LineBreakIterator::new_with_break_rule(
//!     "Hello World",
//!     LineBreakRule::Strict,
//!     WordBreakRule::BreakAll,
//!     false,
//! );
//! let result: Vec<usize> = iter.collect();
//! println!("{:?}", result);
//! ```
//!
//! Use Latin 1 string for C binding and etc.
//!
//! ```rust
//! use icu_segmenter::LineBreakIteratorLatin1;
//!
//! let s = "Hello World";
//! let iter = LineBreakIteratorLatin1::new(s.as_bytes());
//! let result: Vec<usize> = iter.collect();
//! println!("{:?}", result);
//! ```
//!
//! # Generating property table
//!
//! Copy the following files to `tools` directory. Then run `./generate_properties.py` in `tools` directory (requires Python 3.8+). Machine generated files are moved to `src` directory.
//! - <https://www.unicode.org/Public/UCD/latest/ucd/LineBreak.txt>
//! - <https://www.unicode.org/Public/UCD/latest/ucd/EastAsianWidth.txt>

mod lb_define;
mod line_breaker;
mod lstm;
mod properties_defines;
mod properties_other;
mod property_table;
mod rule_table;

#[macro_use]
extern crate lazy_static;

pub use crate::line_breaker::*;

#[cfg(test)]
mod tests {
    use crate::LineBreakIterator;
    use crate::LineBreakIteratorLatin1;
    use crate::LineBreakIteratorUtf16;

    #[test]
    fn linebreak() {
        let mut iter = LineBreakIterator::new("hello world");
        assert_eq!(Some(6), iter.next());
        assert_eq!(Some(11), iter.next());
        assert_eq!(None, iter.next());

        iter = LineBreakIterator::new("$10 $10");
        assert_eq!(Some(4), iter.next());
        assert_eq!(Some(7), iter.next());

        // LB10

        // LB14
        iter = LineBreakIterator::new("[  abc def");
        assert_eq!(Some(7), iter.next());
        assert_eq!(Some(10), iter.next());
        assert_eq!(None, iter.next());

        let input: [u8; 10] = [0x5B, 0x20, 0x20, 0x61, 0x62, 0x63, 0x20, 0x64, 0x65, 0x66];
        let mut iter_u8 = LineBreakIteratorLatin1::new(&input);
        assert_eq!(Some(7), iter_u8.next());
        assert_eq!(Some(10), iter_u8.next());
        assert_eq!(None, iter_u8.next());

        let input: [u16; 10] = [0x5B, 0x20, 0x20, 0x61, 0x62, 0x63, 0x20, 0x64, 0x65, 0x66];
        let mut iter_u16 = LineBreakIteratorUtf16::new(&input);
        assert_eq!(Some(7), iter_u16.next());

        // LB15
        iter = LineBreakIterator::new("abc\u{0022}  (def");
        assert_eq!(Some(10), iter.next());

        let input: [u8; 10] = [0x61, 0x62, 0x63, 0x22, 0x20, 0x20, 0x28, 0x64, 0x65, 0x66];
        let mut iter_u8 = LineBreakIteratorLatin1::new(&input);
        assert_eq!(Some(10), iter_u8.next());

        let input: [u16; 10] = [0x61, 0x62, 0x63, 0x22, 0x20, 0x20, 0x28, 0x64, 0x65, 0x66];
        let mut iter_u16 = LineBreakIteratorUtf16::new(&input);
        assert_eq!(Some(10), iter_u16.next());

        // LB16
        iter = LineBreakIterator::new("\u{0029}\u{203C}");
        assert_eq!(Some(4), iter.next());
        iter = LineBreakIterator::new("\u{0029}  \u{203C}");
        assert_eq!(Some(6), iter.next());

        let input: [u16; 4] = [0x29, 0x20, 0x20, 0x203c];
        let mut iter_u16 = LineBreakIteratorUtf16::new(&input);
        assert_eq!(Some(4), iter_u16.next());

        // LB17
        iter = LineBreakIterator::new("\u{2014}\u{2014}aa");
        assert_eq!(Some(6), iter.next());
        iter = LineBreakIterator::new("\u{2014}  \u{2014}aa");
        assert_eq!(Some(8), iter.next());

        iter = LineBreakIterator::new("\u{2014}\u{2014}  \u{2014}\u{2014}123 abc");
        assert_eq!(Some(14), iter.next());
        assert_eq!(Some(18), iter.next());
        assert_eq!(Some(21), iter.next());

        // LB25
        let mut iter = LineBreakIterator::new("(0,1)+(2,3)");
        assert_eq!(Some(11), iter.next());
        let input: [u16; 11] = [
            0x28, 0x30, 0x2C, 0x31, 0x29, 0x2B, 0x28, 0x32, 0x2C, 0x33, 0x29,
        ];
        let mut iter_u16 = LineBreakIteratorUtf16::new(&input);
        assert_eq!(Some(11), iter_u16.next());

        let input: [u16; 13] = [
            0x2014, 0x2014, 0x20, 0x20, 0x2014, 0x2014, 0x31, 0x32, 0x33, 0x20, 0x61, 0x62, 0x63,
        ];
        let mut iter_u16 = LineBreakIteratorUtf16::new(&input);
        assert_eq!(Some(6), iter_u16.next());

        iter = LineBreakIterator::new("\u{1F3FB} \u{1F3FB}");
        assert_eq!(Some(5), iter.next());
    }
}
