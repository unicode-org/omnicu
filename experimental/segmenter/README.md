# icu_segmenter [![crates.io](http://meritbadge.herokuapp.com/icu_segmenter)](https://crates.io/crates/icu_segmenter)

A line breaker that is compatible with [Unicode Standard Annex #14][UAX14] and CSS properties.

[UAX14]: http://www.unicode.org/reports/tr14/

```rust
use icu_segmenter::LineBreakIterator;

let mut iter = LineBreakIterator::new("Hello World");
let result: Vec<usize> = iter.collect();
println!("{:?}", result);
```

With CSS property.
```rust
use icu_segmenter::{LineBreakIterator, LineBreakRule, WordBreakRule};

let iter = LineBreakIterator::new_with_break_rule(
    "Hello World",
    LineBreakRule::Strict,
    WordBreakRule::BreakAll,
    false,
);
let result: Vec<usize> = iter.collect();
println!("{:?}", result);
```

Use Latin 1 string for C binding and etc.

```rust
use icu_segmenter::LineBreakIteratorLatin1;

let s = "Hello World";
let iter = LineBreakIteratorLatin1::new(s.as_bytes());
let result: Vec<usize> = iter.collect();
println!("{:?}", result);
```

## Generating property table

Copy the following files to `tools` directory. Then run `./generate_properties.py` in `tools` directory (requires Python 3.8+). Machine generated files are moved to `src` directory.
- <https://www.unicode.org/Public/UCD/latest/ucd/LineBreak.txt>
- <https://www.unicode.org/Public/UCD/latest/ucd/EastAsianWidth.txt>

## More Information

For more information on development, authorship, contributing etc. please visit [`ICU4X home page`](https://github.com/unicode-org/icu4x).
