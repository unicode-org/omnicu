// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

extern crate unicode_width;

use crate::lb_define::*;
use crate::lstm::*;
use crate::property_table::*;
use crate::rule_table::*;

use core::char;
use core::str::CharIndices;
use unicode_width::UnicodeWidthChar;

#[derive(Copy, Clone, PartialEq)]
pub enum LineBreakRule {
    /// Use `line-break: normal;` line break rule
    Normal,
    /// Use `line-break: strict;` line break rule
    Strict,
    /// Use `line-break: loose;` line break rule
    Loose,
    /// Use `line-break: anywhere;` line break rule
    Anywhere,
}

#[derive(Copy, Clone, PartialEq)]
pub enum WordBreakRule {
    /// Use `word-break: normal;` line break rule
    Normal,
    /// Use `word-break: break-all;` line break rule
    BreakAll,
    /// Use `word-break: keep-all;` line break rule
    KeepAll,
}

fn get_linebreak_property_utf32_with_rule(
    codepoint: u32,
    line_break_rule: LineBreakRule,
    word_break_rule: WordBreakRule,
) -> u8 {
    if codepoint < 0x20000 {
        let codepoint = codepoint as usize;
        let prop = UAX14_PROPERTY_TABLE[codepoint / 1024][(codepoint & 0x3ff)];

        if word_break_rule == WordBreakRule::BreakAll
            || line_break_rule == LineBreakRule::Loose
            || line_break_rule == LineBreakRule::Normal
        {
            return match prop {
                CJ => ID, // All CJ's General_Category is Other_Letter (Lo).
                _ => prop,
            };
        }

        // CJ is treated as NS by default, yielding strict line breaking.
        // https://www.unicode.org/reports/tr14/#CJ
        return prop;
    }

    match codepoint {
        0x20000..=0x2fffd => ID,
        0x30000..=0x3fffd => ID,
        0xe0001 => CM,
        0xe0020..=0xe007f => CM,
        0xe0100..=0xe01ef => CM,
        _ => XX,
    }
}

#[inline]
fn get_linebreak_property_latin1(codepoint: u8) -> u8 {
    let codepoint = codepoint as usize;
    UAX14_PROPERTY_TABLE[codepoint / 1024][(codepoint & 0x3ff)]
}

#[inline]
fn get_linebreak_property_with_rule(
    codepoint: char,
    linebreak_rule: LineBreakRule,
    wordbreak_rule: WordBreakRule,
) -> u8 {
    get_linebreak_property_utf32_with_rule(codepoint as u32, linebreak_rule, wordbreak_rule)
}

#[inline]
fn is_break_utf32_by_normal(codepoint: u32, ja_zh: bool) -> bool {
    match codepoint as u32 {
        0x301C => ja_zh,
        0x30A0 => ja_zh,
        _ => false,
    }
}

#[inline]
fn is_break_utf32_by_loose(
    left_codepoint: u32,
    right_codepoint: u32,
    left_prop: u8,
    right_prop: u8,
    ja_zh: bool,
) -> Option<bool> {
    // breaks before hyphens
    if right_prop == BA {
        if left_prop == ID && (right_codepoint == 0x2010 || right_codepoint == 0x2013) {
            return Some(true);
        }
    } else if right_prop == NS {
        // breaks before certain CJK hyphen-like characters
        if right_codepoint == 0x301C || right_codepoint == 0x30A0 {
            return Some(ja_zh);
        }

        // breaks before iteration marks
        if right_codepoint == 0x3005
            || right_codepoint == 0x303B
            || right_codepoint == 0x309D
            || right_codepoint == 0x309E
            || right_codepoint == 0x30FD
            || right_codepoint == 0x30FE
        {
            return Some(true);
        }

        // breaks before certain centered punctuation marks:
        if right_codepoint == 0x30FB
            || right_codepoint == 0xFF1A
            || right_codepoint == 0xFF1B
            || right_codepoint == 0xFF65
            || right_codepoint == 0x203C
            || (0x2047..=0x2049).contains(&right_codepoint)
        {
            return Some(ja_zh);
        }
    } else if right_prop == IN {
        // breaks between inseparable characters such as U+2025, U+2026 i.e. characters with the Unicode Line Break property IN
        return Some(true);
    } else if right_prop == EX {
        // breaks before certain centered punctuation marks:
        if right_codepoint == 0xFF01 || right_codepoint == 0xFF1F {
            return Some(ja_zh);
        }
    }

    // breaks before suffixes:
    // Characters with the Unicode Line Break property PO and the East Asian Width property
    if right_prop == PO
        && UnicodeWidthChar::width_cjk(char::from_u32(right_codepoint).unwrap()).unwrap() == 2
    {
        return Some(ja_zh);
    }
    // breaks after prefixes:
    // Characters with the Unicode Line Break property PR and the East Asian Width property
    if left_prop == PR
        && UnicodeWidthChar::width_cjk(char::from_u32(left_codepoint).unwrap()).unwrap() == 2
    {
        return Some(ja_zh);
    }
    None
}

#[inline]
fn is_break_from_table(rule_table: &[i8], property_count: usize, left: u8, right: u8) -> bool {
    let rule = get_break_state_from_table(rule_table, property_count, left, right);
    if rule == KEEP_RULE {
        return false;
    }
    if rule >= 0 {
        // need additional next characters to get break rule.
        return false;
    }
    true
}

#[inline]
fn is_break(left: u8, right: u8) -> bool {
    is_break_from_table(&UAX14_RULE_TABLE, PROP_COUNT, left, right)
}

#[inline]
fn is_non_break_by_keepall(left: u8, right: u8) -> bool {
    //  typographic letter units shouldn't be break
    (left == AI
        || left == AL
        || left == ID
        || left == NU
        || left == HY
        || left == H2
        || left == H3
        || left == JL
        || left == JV
        || left == JT
        || left == CJ)
        && (right == AI
            || right == AL
            || right == ID
            || right == NU
            || right == HY
            || right == H2
            || right == H3
            || right == JL
            || right == JV
            || right == JT
            || right == CJ)
}

#[inline]
fn get_break_state_from_table(rule_table: &[i8], property_count: usize, left: u8, right: u8) -> i8 {
    rule_table[((left as usize) - 1) * property_count + (right as usize) - 1]
}

#[inline]
fn get_break_state(left: u8, right: u8) -> i8 {
    get_break_state_from_table(&UAX14_RULE_TABLE, PROP_COUNT, left, right)
}

#[inline]
fn use_complex_breaking_utf32(codepoint: u32) -> bool {
    // Thai
    (0xe01..=0xe7f).contains(&codepoint)
}

/*
#[inline]
fn use_complex_breaking_utf32(codepoint: u32) -> bool {
    // Thai, Lao and Khmer
    (codepoint >= 0xe01 && codepoint <= 0xeff) || (codepoint >= 0x1780 && codepoint <= 0x17ff)
}
*/

macro_rules! break_iterator_impl {
    ($name:ident, $iter_attr:ty, $char_type:ty) => {
        #[allow(dead_code)]
        pub struct $name<'a> {
            iter: $iter_attr,
            len: usize,
            current_pos_data: Option<(usize, $char_type)>,
            result_cache: Vec<usize>,
            break_rule: LineBreakRule,
            word_break_rule: WordBreakRule,
            ja_zh: bool,
        }

        impl<'a> Iterator for $name<'a> {
            type Item = usize;

            fn next(&mut self) -> Option<Self::Item> {
                if self.is_eof() {
                    return None;
                }

                if !self.result_cache.is_empty() {
                    // We have break point cache by previous run.
                    let mut i = 0;
                    loop {
                        if i == *self.result_cache.first().unwrap() {
                            self.result_cache.remove(0);
                            self.result_cache = self.result_cache.iter().map(|r| r - i).collect();
                            return Some(self.current_pos_data.unwrap().0);
                        }
                        self.current_pos_data = self.iter.next();
                        if self.current_pos_data.is_none() {
                            // Reach EOF
                            self.result_cache.clear();
                            return Some(self.len);
                        }
                        i += 1;
                    }
                }

                loop {
                    let mut left_prop = self.get_linebreak_property();
                    let left_codepoint = self.current_pos_data;
                    self.current_pos_data = self.iter.next();
                    if self.current_pos_data.is_none() {
                        // EOF
                        return Some(self.len);
                    }
                    let right_prop = self.get_linebreak_property();

                    // CSS word-break property handling
                    match self.word_break_rule {
                        WordBreakRule::BreakAll => {
                            left_prop = match left_prop {
                                AL => ID,
                                NU => ID,
                                SA => ID,
                                _ => left_prop,
                            };
                        }
                        WordBreakRule::KeepAll => {
                            if is_non_break_by_keepall(left_prop, right_prop) {
                                continue;
                            }
                        }
                        _ => (),
                    }

                    // CSS line-break property handling
                    match self.break_rule {
                        LineBreakRule::Normal => {
                            if self.is_break_by_normal() {
                                return Some(self.current_pos_data.unwrap().0);
                            }
                        }
                        LineBreakRule::Loose => {
                            if let Some(breakable) = is_break_utf32_by_loose(
                                left_codepoint.unwrap().1 as u32,
                                self.current_pos_data.unwrap().1 as u32,
                                left_prop,
                                right_prop,
                                self.ja_zh,
                            ) {
                                if breakable {
                                    return Some(self.current_pos_data.unwrap().0);
                                }
                                continue;
                            }
                        }
                        LineBreakRule::Anywhere => {
                            return Some(self.current_pos_data.unwrap().0);
                        }
                        _ => (),
                    };

                    // UAX14 doesn't have Thai etc, so use another way.
                    if self.word_break_rule != WordBreakRule::BreakAll
                        && $name::use_complex_breaking(left_codepoint.unwrap().1)
                        && $name::use_complex_breaking(self.current_pos_data.unwrap().1)
                    {
                        let result = self.handle_complex_language(left_codepoint.unwrap().1);
                        if result.is_some() {
                            return result;
                        }
                        // I may have to fetch text until non-SA character?.
                    }

                    // If break_state is equals or grater than 0, it is alias of property.
                    let mut break_state = get_break_state(left_prop, right_prop);
                    if break_state >= 0 as i8 {
                        let mut previous_iter = self.iter.clone();
                        let mut previous_pos_data = self.current_pos_data;

                        loop {
                            self.current_pos_data = self.iter.next();
                            if self.current_pos_data.is_none() {
                                // Reached EOF. But we are analyzing multiple characters now, so next break may be previous point.
                                let break_state = get_break_state(break_state as u8, EOT);
                                if break_state == PREVIOUS_BREAK_RULE {
                                    self.iter = previous_iter;
                                    self.current_pos_data = previous_pos_data;
                                    return Some(previous_pos_data.unwrap().0);
                                }
                                // EOF
                                return Some(self.len);
                            }

                            let prop = self.get_linebreak_property();
                            break_state = get_break_state(break_state as u8, prop);
                            if break_state < 0 {
                                break;
                            }

                            previous_iter = self.iter.clone();
                            previous_pos_data = self.current_pos_data;
                        }
                        if break_state == KEEP_RULE {
                            continue;
                        }
                        if break_state == PREVIOUS_BREAK_RULE {
                            self.iter = previous_iter;
                            self.current_pos_data = previous_pos_data;
                            return Some(previous_pos_data.unwrap().0);
                        }
                        return Some(self.current_pos_data.unwrap().0);
                    }

                    if is_break(left_prop, right_prop) {
                        return Some(self.current_pos_data.unwrap().0);
                    }
                }
            }
        }

        impl<'a> $name<'a> {
            #[inline]
            fn is_eof(&mut self) -> bool {
                if self.current_pos_data.is_none() {
                    self.current_pos_data = self.iter.next();
                    if self.current_pos_data.is_none() {
                        return true;
                    }
                }
                return false;
            }

            // UAX14 doesn't define line break rules for some languages such as Thai.
            // These languages uses dictionary-based breaker, so we use OS's line breaker instead.
            fn handle_complex_language(&mut self, left_codepoint: $char_type) -> Option<usize> {
                let start_iter = self.iter.clone();
                let start_point = self.current_pos_data;
                let mut s = vec![left_codepoint as u16];
                loop {
                    s.push(self.current_pos_data.unwrap().1 as u16);
                    self.current_pos_data = self.iter.next();
                    if self.current_pos_data.is_none() {
                        break;
                    }
                    if !$name::use_complex_breaking(self.current_pos_data.unwrap().1) {
                        break;
                    }
                }
                // Restore iterator to move to head of complex string
                self.iter = start_iter;
                self.current_pos_data = start_point;
                let breaks = self.get_line_break_by_platform_fallback(&s);
                let mut i = 1;
                self.result_cache = breaks;
                // result_cache vector is utf-16 index that is in BMP.
                loop {
                    if i == *self.result_cache.first().unwrap() {
                        self.result_cache.remove(0);
                        self.result_cache = self.result_cache.iter().map(|r| r - i).collect();
                        return Some(self.current_pos_data.unwrap().0);
                    }
                    self.current_pos_data = self.iter.next();
                    if self.current_pos_data.is_none() {
                        self.result_cache.clear();
                        return Some(self.len);
                    }
                    i += 1;
                }
            }
        }
    };
}

break_iterator_impl!(LineBreakIterator, CharIndices<'a>, char);

impl<'a> LineBreakIterator<'a> {
    /// Create line break iterator
    pub fn new(input: &str) -> LineBreakIterator {
        LineBreakIterator {
            iter: input.char_indices(),
            len: input.len(),
            current_pos_data: None,
            result_cache: Vec::new(),
            break_rule: LineBreakRule::Strict,
            word_break_rule: WordBreakRule::Normal,
            ja_zh: false,
        }
    }

    /// Create line break iterator with CSS rules
    pub fn new_with_break_rule(
        input: &str,
        line_break_rule: LineBreakRule,
        word_break_rule: WordBreakRule,
        ja_zh: bool,
    ) -> LineBreakIterator {
        LineBreakIterator {
            iter: input.char_indices(),
            len: input.len(),
            current_pos_data: None,
            result_cache: Vec::new(),
            break_rule: line_break_rule,
            word_break_rule,
            ja_zh,
        }
    }

    fn get_linebreak_property(&mut self) -> u8 {
        self.get_linebreak_property_with_rule(self.current_pos_data.unwrap().1)
    }

    fn get_linebreak_property_with_rule(&mut self, c: char) -> u8 {
        get_linebreak_property_with_rule(c, self.break_rule, self.word_break_rule)
    }

    fn is_break_by_normal(&mut self) -> bool {
        is_break_utf32_by_normal(self.current_pos_data.unwrap().1 as u32, self.ja_zh)
    }

    #[inline]
    fn use_complex_breaking(c: char) -> bool {
        use_complex_breaking_utf32(c as u32)
    }

    fn get_line_break_by_platform_fallback(&mut self, input: &[u16]) -> Vec<usize> {
        if let Some(mut ret) = get_line_break_utf16(input) {
            ret.push(input.len());
            return ret;
        }
        [input.len()].to_vec()
    }

    /*
        fn handle_complex_language(&mut self, left_codepoint: char) -> Option<usize> {
            let start_iter = self.iter.clone();
            let start_point = self.current_pos_data;
            loop {
                self.current_pos_data = self.iter.next();
                if self.current_pos_data.is_none() {
                    break;
                }
                if !self.use_complex_breaking(self.current_pos_data.unwrap().1) {
                    break;
                }
            }
            if let Some(mut breaks) = get_line_break_utf8(&str) {
                breaks.push(str.len());
                return breaks;
            }
            [str.len_utf8()].to_vec()
        }
    */
}

/// Latin-1 version of line break iterator.
#[derive(Clone)]
struct Latin1Indices<'a> {
    front_offset: usize,
    iter: &'a [u8],
}

impl<'a> Iterator for Latin1Indices<'a> {
    type Item = (usize, u8);

    #[inline]
    fn next(&mut self) -> Option<(usize, u8)> {
        if self.front_offset >= self.iter.len() {
            return None;
        }
        let ch = self.iter[self.front_offset];
        let index = self.front_offset;
        self.front_offset += 1;
        Some((index, ch))
    }
}

break_iterator_impl!(LineBreakIteratorLatin1, Latin1Indices<'a>, u8);

impl<'a> LineBreakIteratorLatin1<'a> {
    /// Create line break iterator using Latin-1/8-bit string.
    pub fn new(input: &[u8]) -> LineBreakIteratorLatin1 {
        LineBreakIteratorLatin1 {
            iter: Latin1Indices {
                front_offset: 0,
                iter: input,
            },
            len: input.len(),
            current_pos_data: None,
            result_cache: Vec::new(),
            break_rule: LineBreakRule::Strict,
            word_break_rule: WordBreakRule::Normal,
            ja_zh: false,
        }
    }

    /// Create line break iterator with CSS rules using Latin-1/8-bit string.
    pub fn new_with_break_rule(
        input: &[u8],
        line_break_rule: LineBreakRule,
        word_break_rule: WordBreakRule,
        ja_zh: bool,
    ) -> LineBreakIteratorLatin1 {
        LineBreakIteratorLatin1 {
            iter: Latin1Indices {
                front_offset: 0,
                iter: input,
            },
            len: input.len(),
            current_pos_data: None,
            result_cache: Vec::new(),
            break_rule: line_break_rule,
            word_break_rule,
            ja_zh,
        }
    }

    fn get_linebreak_property(&mut self) -> u8 {
        // No CJ on Latin1
        self.get_linebreak_property_with_rule(self.current_pos_data.unwrap().1)
    }

    fn get_linebreak_property_with_rule(&mut self, c: u8) -> u8 {
        // No CJ on Latin1
        get_linebreak_property_latin1(c)
    }

    fn is_break_by_normal(&mut self) -> bool {
        is_break_utf32_by_normal(self.current_pos_data.unwrap().1 as u32, self.ja_zh)
    }

    #[inline]
    fn use_complex_breaking(_c: u8) -> bool {
        false
    }

    fn get_line_break_by_platform_fallback(&mut self, _input: &[u16]) -> Vec<usize> {
        panic!("not reachable");
    }
}

/// UTF-16 version of line break iterator.
#[derive(Clone)]
struct Utf16Indices<'a> {
    front_offset: usize,
    iter: &'a [u16],
}

impl<'a> Iterator for Utf16Indices<'a> {
    type Item = (usize, u32);

    #[inline]
    fn next(&mut self) -> Option<(usize, u32)> {
        if self.front_offset >= self.iter.len() {
            return None;
        }
        let ch = self.iter[self.front_offset];
        let index = self.front_offset;
        self.front_offset += 1;

        if (ch & 0xfc00) != 0xd800 {
            return Some((index, ch as u32));
        }

        let mut ch = ch as u32;
        if self.front_offset < self.iter.len() {
            let next = self.iter[self.front_offset] as u32;
            if (next & 0xfc00) == 0xdc00 {
                ch = ((ch & 0x3ff) << 10) + (next & 0x3ff) + 0x10000;
                self.front_offset += 1;
                return Some((index, ch));
            }
        }
        Some((index, ch))
    }
}

break_iterator_impl!(LineBreakIteratorUtf16, Utf16Indices<'a>, u32);

impl<'a> LineBreakIteratorUtf16<'a> {
    /// Create line break iterator using UTF-16 string.
    pub fn new(input: &[u16]) -> LineBreakIteratorUtf16 {
        LineBreakIteratorUtf16 {
            iter: Utf16Indices {
                front_offset: 0,
                iter: input,
            },
            len: input.len(),
            current_pos_data: None,
            result_cache: Vec::new(),
            break_rule: LineBreakRule::Strict,
            word_break_rule: WordBreakRule::Normal,
            ja_zh: false,
        }
    }

    /// Create line break iterator with CSS rules using UTF-16 string.
    pub fn new_with_break_rule(
        input: &[u16],
        line_break_rule: LineBreakRule,
        word_break_rule: WordBreakRule,
        ja_zh: bool,
    ) -> LineBreakIteratorUtf16 {
        LineBreakIteratorUtf16 {
            iter: Utf16Indices {
                front_offset: 0,
                iter: input,
            },
            len: input.len(),
            current_pos_data: None,
            result_cache: Vec::new(),
            break_rule: line_break_rule,
            word_break_rule,
            ja_zh,
        }
    }

    fn get_linebreak_property(&mut self) -> u8 {
        self.get_linebreak_property_with_rule(self.current_pos_data.unwrap().1)
    }

    fn get_linebreak_property_with_rule(&mut self, c: u32) -> u8 {
        get_linebreak_property_utf32_with_rule(c, self.break_rule, self.word_break_rule)
    }

    fn is_break_by_normal(&mut self) -> bool {
        is_break_utf32_by_normal(self.current_pos_data.unwrap().1 as u32, self.ja_zh)
    }

    #[inline]
    fn use_complex_breaking(c: u32) -> bool {
        use_complex_breaking_utf32(c)
    }

    fn get_line_break_by_platform_fallback(&mut self, input: &[u16]) -> Vec<usize> {
        if let Some(mut ret) = get_line_break_utf16(input) {
            ret.push(input.len());
            return ret;
        }
        [input.len()].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use crate::lb_define::*;
    use crate::line_breaker::get_linebreak_property_with_rule;
    use crate::line_breaker::is_break;
    use crate::LineBreakRule;
    use crate::WordBreakRule;

    fn get_linebreak_property(codepoint: char) -> u8 {
        get_linebreak_property_with_rule(codepoint, LineBreakRule::Strict, WordBreakRule::Normal)
    }

    #[test]
    fn linebreak_propery() {
        assert_eq!(get_linebreak_property('\u{0020}'), SP);
        assert_eq!(get_linebreak_property('\u{0022}'), QU);
        assert_eq!(get_linebreak_property('('), OP_OP30);
        assert_eq!(get_linebreak_property('\u{0030}'), NU);
        assert_eq!(get_linebreak_property('['), OP_OP30);
        assert_eq!(get_linebreak_property('\u{1f3fb}'), EM);
        assert_eq!(get_linebreak_property('\u{20000}'), ID);
        assert_eq!(get_linebreak_property('\u{e0020}'), CM);
        assert_eq!(get_linebreak_property('\u{3041}'), CJ);
        assert_eq!(get_linebreak_property('\u{0025}'), PO);
        assert_eq!(get_linebreak_property('\u{00A7}'), AI);
        assert_eq!(get_linebreak_property('\u{50005}'), XX);
        assert_eq!(get_linebreak_property('\u{17D6}'), NS);
        assert_eq!(get_linebreak_property('\u{2014}'), B2);
    }

    #[test]
    fn break_rule() {
        // LB4
        assert_eq!(is_break(BK, AL), true);
        // LB5
        assert_eq!(is_break(CR, LF), false);
        assert_eq!(is_break(CR, AL), true);
        assert_eq!(is_break(LF, AL), true);
        assert_eq!(is_break(NL, AL), true);
        // LB6
        assert_eq!(is_break(AL, BK), false);
        assert_eq!(is_break(AL, CR), false);
        assert_eq!(is_break(AL, LF), false);
        assert_eq!(is_break(AL, NL), false);
        // LB7
        assert_eq!(is_break(AL, SP), false);
        assert_eq!(is_break(AL, ZW), false);
        // LB8
        // LB8a
        assert_eq!(is_break(ZWJ, AL), false);
        // LB9
        assert_eq!(is_break(AL, ZWJ), false);
        assert_eq!(is_break(AL, CM), false);
        assert_eq!(is_break(ID, ZWJ), false);
        // LB10
        assert_eq!(is_break(ZWJ, SP), false);
        assert_eq!(is_break(SP, CM), true);
        // LB11
        assert_eq!(is_break(AL, WJ), false);
        assert_eq!(is_break(WJ, AL), false);
        // LB12
        assert_eq!(is_break(GL, AL), false);
        // LB12a
        assert_eq!(is_break(AL, GL), false);
        assert_eq!(is_break(SP, GL), true);
        // LB13
        assert_eq!(is_break(AL, CL), false);
        assert_eq!(is_break(AL, CP), false);
        assert_eq!(is_break(AL, EX), false);
        assert_eq!(is_break(AL, IS), false);
        assert_eq!(is_break(AL, SY), false);
        // LB18
        assert_eq!(is_break(SP, AL), true);
        // LB19
        assert_eq!(is_break(AL, QU), false);
        assert_eq!(is_break(QU, AL), false);
        // LB20
        assert_eq!(is_break(AL, CB), true);
        assert_eq!(is_break(CB, AL), true);
        // LB20
        assert_eq!(is_break(AL, BA), false);
        assert_eq!(is_break(AL, HY), false);
        assert_eq!(is_break(AL, NS), false);
        assert_eq!(is_break(BB, AL), false);
        // LB21
        assert_eq!(is_break(AL, BA), false);
        assert_eq!(is_break(ID, BA), false);
        assert_eq!(is_break(ID, NS), false);
        // LB21a
        // LB21b
        assert_eq!(is_break(SY, HL), false);
        // LB22
        assert_eq!(is_break(AL, IN), false);
        // LB 23
        assert_eq!(is_break(AL, NU), false);
        assert_eq!(is_break(HL, NU), false);
        // LB 23a
        assert_eq!(is_break(PR, ID), false);
        assert_eq!(is_break(PR, EB), false);
        assert_eq!(is_break(PR, EM), false);
        assert_eq!(is_break(ID, PO), false);
        assert_eq!(is_break(EB, PO), false);
        assert_eq!(is_break(EM, PO), false);
        // LB26
        assert_eq!(is_break(JL, JL), false);
        assert_eq!(is_break(JL, JV), false);
        assert_eq!(is_break(JL, H2), false);
        // LB27
        assert_eq!(is_break(JL, IN), false);
        assert_eq!(is_break(JL, PO), false);
        assert_eq!(is_break(PR, JL), false);
        // LB28
        assert_eq!(is_break(AL, AL), false);
        assert_eq!(is_break(HL, AL), false);
        // LB29
        assert_eq!(is_break(IS, AL), false);
        assert_eq!(is_break(IS, HL), false);
        // LB30b
        assert_eq!(is_break(EB, EM), false);
        // LB31
        assert_eq!(is_break(ID, ID), true);
    }
}
