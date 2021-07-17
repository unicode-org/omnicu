// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use std::{
    char,
    ops::{Bound::*, RangeBounds},
};

/// Returns whether the vector is sorted ascending non inclusive, of even length,
/// and within the bounds of `0x0 -> 0x10FFFF` inclusive.
pub fn is_valid(v: &[u32]) -> bool {
    v.is_empty() || (v.len() % 2 == 0 && v.windows(2).all(|chunk| chunk[0] < chunk[1]) && v.last().map_or(false, |e| e <= &((char::MAX as u32) + 1)))
}

/// Returns start (inclusive) and end (exclusive) bounds of [`RangeBounds`]
pub fn deconstruct_range(range: &impl RangeBounds<char>) -> (u32, u32) {
    let from = match range.start_bound() {
        Included(b) => (*b as u32),
        Excluded(_) => unreachable!(),
        Unbounded => 0,
    };
    let till = match range.end_bound() {
        Included(b) => (*b as u32) + 1,
        Excluded(b) => (*b as u32),
        Unbounded => (char::MAX as u32) + 1,
    };
    (from, till)
}

#[cfg(test)]
mod tests {
    use super::{deconstruct_range, is_valid};
    use std::char;

    #[test]
    fn test_is_valid() {
        let check = vec![0x2, 0x3, 0x4, 0x5];
        assert!(is_valid(&check));
    }
    #[test]
    fn test_is_valid_empty() {
        let check = vec![];
        assert!(is_valid(&check));
    }
    #[test]
    fn test_is_valid_overlapping() {
        let check = vec![0x2, 0x5, 0x4, 0x6];
        assert!(!is_valid(&check));
    }
    #[test]
    fn test_is_valid_out_of_order() {
        let check = vec![0x5, 0x4, 0x5, 0x6, 0x7];
        assert!(!is_valid(&check));
    }
    #[test]
    fn test_is_valid_duplicate() {
        let check = vec![0x1, 0x2, 0x3, 0x3, 0x5];
        assert!(!is_valid(&check));
    }
    #[test]
    fn test_is_valid_odd() {
        let check = vec![0x1, 0x2, 0x3, 0x4, 0x5];
        assert!(!is_valid(&check));
    }
    #[test]
    fn test_is_valid_out_of_range() {
        let check = vec![0x1, 0x2, 0x3, 0x4, (char::MAX as u32) + 1];
        assert!(!is_valid(&check));
    }
    // deconstruct_range
    #[test]
    fn test_deconstruct_range() {
        let expected = (0x41, 0x45);
        let check = deconstruct_range(&('A'..'E')); // Range
        assert_eq!(check, expected);
        let check = deconstruct_range(&('A'..='D')); // Range Inclusive
        assert_eq!(check, expected);
        let check = deconstruct_range(&('A'..)); // Range From
        assert_eq!(check, (0x41, (char::MAX as u32) + 1));
        let check = deconstruct_range(&(..'A')); // Range To
        assert_eq!(check, (0x0, 0x41));
        let check = deconstruct_range(&(..='A')); // Range To Inclusive
        assert_eq!(check, (0x0, 0x42));
        let check = deconstruct_range(&(..)); // Range Full
        assert_eq!(check, (0x0, (char::MAX as u32) + 1));
    }
}
