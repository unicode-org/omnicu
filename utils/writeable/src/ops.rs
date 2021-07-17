// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::LengthHint;

impl core::ops::Add<LengthHint> for LengthHint {
    type Output = Self;

    fn add(self, other: LengthHint) -> Self {
        match self {
            LengthHint::Undefined => LengthHint::Undefined,
            LengthHint::Exact(len1) => match other {
                LengthHint::Undefined => LengthHint::Undefined,
                LengthHint::Exact(len2) => LengthHint::Exact(len1 + len2),
            },
        }
    }
}

impl core::ops::AddAssign<LengthHint> for LengthHint {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl core::iter::Sum<LengthHint> for LengthHint {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = LengthHint>,
    {
        iter.fold(LengthHint::Exact(0), core::ops::Add::add)
    }
}

impl core::ops::Add<usize> for LengthHint {
    type Output = Self;

    fn add(self, other: usize) -> Self {
        match self {
            LengthHint::Undefined => LengthHint::Undefined,
            LengthHint::Exact(len) => LengthHint::Exact(len + other),
        }
    }
}

impl core::ops::AddAssign<usize> for LengthHint {
    fn add_assign(&mut self, other: usize) {
        *self = *self + other;
    }
}

impl core::iter::Sum<usize> for LengthHint {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = usize>,
    {
        LengthHint::Exact(iter.sum::<usize>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(LengthHint::Exact(5), LengthHint::Exact(3) + 2);
        assert_eq!(
            LengthHint::Exact(5),
            LengthHint::Exact(3) + LengthHint::Exact(2)
        );
        assert_eq!(
            LengthHint::Undefined,
            LengthHint::Exact(3) + LengthHint::Undefined
        );

        assert_eq!(LengthHint::Undefined, LengthHint::Undefined + 2);
        assert_eq!(
            LengthHint::Undefined,
            LengthHint::Undefined + LengthHint::Exact(2)
        );
        assert_eq!(
            LengthHint::Undefined,
            LengthHint::Undefined + LengthHint::Undefined
        );

        let mut len = LengthHint::Exact(5);
        len += LengthHint::Exact(3);
        assert_eq!(LengthHint::Exact(8), len);
        len += 2;
        assert_eq!(LengthHint::Exact(10), len);
        len += LengthHint::Undefined;
        assert_eq!(LengthHint::Undefined, len);

        len += LengthHint::Exact(3);
        assert_eq!(LengthHint::Undefined, len);
        len += 2;
        assert_eq!(LengthHint::Undefined, len);
        len += LengthHint::Undefined;
        assert_eq!(LengthHint::Undefined, len);
    }

    #[test]
    fn test_sum() {
        let lens = vec![
            LengthHint::Exact(4),
            LengthHint::Exact(1),
            LengthHint::Exact(1),
        ];
        assert_eq!(
            LengthHint::Exact(6),
            lens.iter().copied().sum::<LengthHint>()
        );

        let lens = vec![
            LengthHint::Exact(4),
            LengthHint::Undefined,
            LengthHint::Exact(1),
        ];
        assert_eq!(
            LengthHint::Undefined,
            lens.iter().copied().sum::<LengthHint>()
        );

        let lens = vec![4, 1, 1];
        assert_eq!(
            LengthHint::Exact(6),
            lens.iter().copied().sum::<LengthHint>()
        );
    }
}
