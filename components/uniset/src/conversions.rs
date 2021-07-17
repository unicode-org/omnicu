// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use std::{
    convert::TryFrom,
    ops::{Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

use super::UnicodeSetError;
use crate::utils::deconstruct_range;
use crate::UnicodeSet;

fn try_from_range(range: &impl RangeBounds<char>) -> Result<UnicodeSet, UnicodeSetError> {
    let (from, till) = deconstruct_range(range);
    if from < till {
        let set = vec![from, till];
        Ok(UnicodeSet::from_inversion_list(set).unwrap())
    } else {
        Err(UnicodeSetError::InvalidRange(from, till))
    }
}

impl TryFrom<&Range<char>> for UnicodeSet {
    type Error = UnicodeSetError;

    fn try_from(range: &Range<char>) -> Result<Self, Self::Error> {
        try_from_range(range)
    }
}

impl TryFrom<&RangeFrom<char>> for UnicodeSet {
    type Error = UnicodeSetError;

    fn try_from(range: &RangeFrom<char>) -> Result<Self, Self::Error> {
        try_from_range(range)
    }
}

impl TryFrom<&RangeFull> for UnicodeSet {
    type Error = UnicodeSetError;

    fn try_from(_: &RangeFull) -> Result<Self, Self::Error> {
        Ok(Self::all())
    }
}

impl TryFrom<&RangeInclusive<char>> for UnicodeSet {
    type Error = UnicodeSetError;

    fn try_from(range: &RangeInclusive<char>) -> Result<Self, Self::Error> {
        try_from_range(range)
    }
}

impl TryFrom<&RangeTo<char>> for UnicodeSet {
    type Error = UnicodeSetError;

    fn try_from(range: &RangeTo<char>) -> Result<Self, Self::Error> {
        try_from_range(range)
    }
}

impl TryFrom<&RangeToInclusive<char>> for UnicodeSet {
    type Error = UnicodeSetError;

    fn try_from(range: &RangeToInclusive<char>) -> Result<Self, Self::Error> {
        try_from_range(range)
    }
}

#[cfg(test)]
mod tests {
    use super::UnicodeSetError;
    use crate::UnicodeSet;
    use std::{char, convert::TryFrom};

    #[test]
    fn test_try_from_range() {
        let check: Vec<char> = UnicodeSet::try_from(&('A'..'B')).unwrap().iter_chars().collect();
        assert_eq!(vec!['A'], check);
    }
    #[test]
    fn test_try_from_range_error() {
        let check = UnicodeSet::try_from(&('A'..'A'));
        assert!(matches!(check, Err(UnicodeSetError::InvalidRange(65, 65))));
    }
    #[test]
    fn test_try_from_range_inclusive() {
        let check: Vec<char> = UnicodeSet::try_from(&('A'..='A')).unwrap().iter_chars().collect();
        assert_eq!(vec!['A'], check);
    }
    #[test]
    fn test_try_from_range_inclusive_err() {
        let check = UnicodeSet::try_from(&('B'..'A'));
        assert!(matches!(check, Err(UnicodeSetError::InvalidRange(66, 65))));
    }
    #[test]
    fn test_try_from_range_from() {
        let uset = UnicodeSet::try_from(&('A'..)).unwrap();
        let check: usize = uset.size();
        let expected: usize = (char::MAX as usize) + 1 - 65;
        assert_eq!(expected, check);
    }
    #[test]
    fn test_try_from_range_to() {
        let uset = UnicodeSet::try_from(&(..'A')).unwrap();
        let check: usize = uset.size();
        let expected: usize = 65;
        assert_eq!(expected, check);
    }
    #[test]
    fn test_try_from_range_to_err() {
        let check = UnicodeSet::try_from(&(..(0x0 as char)));
        assert!(matches!(check, Err(UnicodeSetError::InvalidRange(0, 0))));
    }
    #[test]
    fn test_try_from_range_to_inclusive() {
        let uset = UnicodeSet::try_from(&(..='A')).unwrap();
        let check: usize = uset.size();
        let expected: usize = 66;
        assert_eq!(expected, check);
    }
    #[test]
    fn test_try_from_range_full() {
        let uset = UnicodeSet::try_from(&(..)).unwrap();
        let check: usize = uset.size();
        let expected: usize = (char::MAX as usize) + 1;
        assert_eq!(expected, check);
    }
}
