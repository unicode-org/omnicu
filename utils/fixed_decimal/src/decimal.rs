// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use smallvec::SmallVec;

use std::cmp;
use std::cmp::Ordering;
use std::fmt;
use std::ops::RangeInclusive;

use std::str::FromStr;

#[cfg(feature = "ryu_decimal")]
use std::convert::TryInto;

use static_assertions::const_assert;

use crate::signum::Signum;
use crate::uint_iterator::IntIterator;

use crate::Error;

// FixedDecimal assumes usize (digits.len()) is at least as big as a u16
const_assert!(std::mem::size_of::<usize>() >= std::mem::size_of::<u16>());

/// A struct containing decimal digits with efficient iteration and manipulation by magnitude
/// (power of 10). Supports a mantissa of non-zero digits and a number of leading and trailing
/// zeros, used for formatting and plural selection.
///
/// You can create a `FixedDecimal` from a standard integer type. To represent fraction digits,
/// call `.multiply_pow10()` after creating your `FixedDecimal`.
///
/// # Examples
///
/// ```
/// use fixed_decimal::FixedDecimal;
///
/// let mut dec = FixedDecimal::from(250);
/// assert_eq!("250", dec.to_string());
///
/// dec.multiply_pow10(-2);
/// assert_eq!("2.50", dec.to_string());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FixedDecimal {
    /// List of digits; digits\[0\] is the most significant.
    ///
    /// Invariants:
    /// - Must not include leading or trailing zeros
    /// - Length must not exceed (magnitude - lower_magnitude + 1)
    // TODO: Consider using a nibble array
    digits: SmallVec<[u8; 8]>,

    /// Power of 10 of digits\[0\].
    ///
    /// Invariants:
    /// - <= upper_magnitude
    /// - >= lower_magnitude
    magnitude: i16,

    /// Power of 10 of the most significant digit, which may be zero.
    ///
    /// Invariants:
    /// - >= 0
    /// - >= magnitude
    upper_magnitude: i16,

    /// Power of 10 of the least significant digit, which may be zero.
    ///
    /// Invariants:
    /// - <= 0
    /// - <= magnitude
    lower_magnitude: i16,

    /// Whether the number is negative. Negative zero is supported.
    is_negative: bool,
}

impl Default for FixedDecimal {
    /// Returns a `FixedDecimal` representing zero.
    fn default() -> Self {
        Self {
            digits: SmallVec::new(),
            magnitude: 0,
            upper_magnitude: 0,
            lower_magnitude: 0,
            is_negative: false,
        }
    }
}

macro_rules! impl_from_signed_integer_type {
    ($itype:ident, $utype: ident) => {
        impl From<$itype> for FixedDecimal {
            fn from(value: $itype) -> Self {
                let int_iterator: IntIterator<$utype> = value.into();
                let is_negative = int_iterator.is_negative;
                let mut result = Self::from_ascending(int_iterator)
                    .expect("All built-in integer types should fit");
                result.is_negative = is_negative;
                result
            }
        }
    };
}

macro_rules! impl_from_unsigned_integer_type {
    ($utype: ident) => {
        impl From<$utype> for FixedDecimal {
            fn from(value: $utype) -> Self {
                let int_iterator: IntIterator<$utype> = value.into();
                Self::from_ascending(int_iterator).expect("All built-in integer types should fit")
            }
        }
    };
}

impl_from_signed_integer_type!(isize, usize);
impl_from_signed_integer_type!(i128, u128);
impl_from_signed_integer_type!(i64, u64);
impl_from_signed_integer_type!(i32, u32);
impl_from_signed_integer_type!(i16, u16);
impl_from_signed_integer_type!(i8, u8);

impl_from_unsigned_integer_type!(usize);
impl_from_unsigned_integer_type!(u128);
impl_from_unsigned_integer_type!(u64);
impl_from_unsigned_integer_type!(u32);
impl_from_unsigned_integer_type!(u16);
impl_from_unsigned_integer_type!(u8);

impl FixedDecimal {
    /// Initialize a `FixedDecimal` with an iterator of digits in ascending
    /// order of magnitude, starting with the digit at magnitude 0.
    ///
    /// This method is not public; use `TryFrom::<isize>` instead.
    fn from_ascending<T>(digits_iter: T) -> Result<Self, Error>
    where
        T: Iterator<Item = u8>,
    {
        // TODO: make X a usize generic to customize the size of this array
        // https://github.com/rust-lang/rust/issues/44580
        // NOTE: 39 is the size required for u128: ceil(log10(u128::MAX)) == 39.
        const X: usize = 39;
        // A temporary structure to allow the digits in the iterator to be reversed.
        // The digits are inserted started from the end, and then a slice is copied
        // into its final destination (result.digits).
        let mut mem: [u8; X] = [0u8; X];
        let mut trailing_zeros: usize = 0;
        let mut i: usize = 0;
        for (x, d) in digits_iter.enumerate() {
            // Take only up to std::i16::MAX values so that we have enough capacity
            if x > std::i16::MAX as usize {
                return Err(Error::Limit);
            }
            // TODO: Should we check here that `d` is between 0 and 9?
            // That should always be the case if IntIterator is used.
            if i != 0 || d != 0 {
                i += 1;
                match X.checked_sub(i) {
                    Some(v) => mem[v] = d,
                    // This error should be obsolete after X is made generic
                    None => return Err(Error::Limit),
                }
            } else {
                trailing_zeros += 1;
            }
        }
        let mut result: Self = Default::default();
        if i != 0 {
            let magnitude = trailing_zeros + i - 1;
            debug_assert!(magnitude <= std::i16::MAX as usize);
            result.magnitude = magnitude as i16;
            result.upper_magnitude = result.magnitude;
            debug_assert!(i <= X);
            result.digits.extend_from_slice(&mem[(X - i)..]);
        }
        #[cfg(debug_assertions)]
        result.check_invariants();
        Ok(result)
    }

    /// Initialize a `FixedDecimal` with a float value, using Ryu under the hood
    /// to get digits from the value. The `post_decimal_places` parameter fixes
    /// the number of digits after the decimal point. This API requires the
    /// `ryu_decimal` feature.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_decimal::FixedDecimal;
    ///
    /// let dec = FixedDecimal::from_float_ryu(12.345, 2).unwrap();
    /// assert_eq!("12.34", dec.to_string());
    /// ```
    #[cfg(feature = "ryu_decimal")]
    pub fn from_float_ryu(value: f32, post_decimal_places: u16) -> Result<Self, Error> {
        let i16_post_decimal =
            TryInto::<i16>::try_into(post_decimal_places).map_err(|_| Error::Limit)?;
        let ryu_decimal = ryu_floating_decimal::f2d(value);
        let mut limited_mantissa = ryu_decimal.mantissa;
        let mut limited_exponent: i16 =
            ryu_decimal.exponent.try_into().map_err(|_| Error::Limit)?;

        while -limited_exponent > i16_post_decimal {
            limited_exponent += 1;
            limited_mantissa /= 10;
        }

        let mut res = FixedDecimal::from(limited_mantissa);
        res.multiply_pow10(limited_exponent)?;

        res.lower_magnitude = res
            .lower_magnitude
            .checked_sub(
                // add because exponent is negative of post decimal
                i16_post_decimal
                    .checked_add(limited_exponent)
                    .ok_or(Error::Limit)?,
            )
            .ok_or(Error::Limit)?;

        res.is_negative = value.is_sign_negative();

        Ok(res)
    }

    /// Initialize a `FixedDecimal` with a double value, using Ryu under the hood
    /// to get digits from the value. The `post_decimal_places` parameter fixes
    /// the number of digits after the decimal point. This API requires the
    /// `ryu_decimal` feature.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_decimal::FixedDecimal;
    ///
    /// let dec = FixedDecimal::from_double_ryu(12.345, 2).unwrap();
    /// assert_eq!("12.34", dec.to_string());
    /// ```
    #[cfg(feature = "ryu_decimal")]
    pub fn from_double_ryu(value: f64, post_decimal_places: u16) -> Result<Self, Error> {
        let i16_post_decimal =
            TryInto::<i16>::try_into(post_decimal_places).map_err(|_| Error::Limit)?;
        let ryu_decimal = ryu_floating_decimal::d2d(value);
        let mut limited_mantissa = ryu_decimal.mantissa;
        let mut limited_exponent: i16 =
            ryu_decimal.exponent.try_into().map_err(|_| Error::Limit)?;

        while -limited_exponent > i16_post_decimal {
            limited_exponent += 1;
            limited_mantissa /= 10;
        }

        let mut res = FixedDecimal::from(limited_mantissa);
        res.multiply_pow10(limited_exponent)?;

        res.lower_magnitude = res
            .lower_magnitude
            .checked_sub(
                // add because exponent is negative of post decimal
                i16_post_decimal
                    .checked_add(limited_exponent)
                    .ok_or(Error::Limit)?,
            )
            .ok_or(Error::Limit)?;

        res.is_negative = value.is_sign_negative();

        Ok(res)
    }

    /// Gets the digit at the specified order of magnitude. Returns 0 if the magnitude is out of
    /// range of the currently visible digits.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_decimal::FixedDecimal;
    ///
    /// let dec = FixedDecimal::from(945);
    /// assert_eq!(0, dec.digit_at(-1));
    /// assert_eq!(5, dec.digit_at(0));
    /// assert_eq!(4, dec.digit_at(1));
    /// assert_eq!(9, dec.digit_at(2));
    /// assert_eq!(0, dec.digit_at(3));
    /// ```
    pub fn digit_at(&self, magnitude: i16) -> u8 {
        if magnitude > self.magnitude {
            0 // Leading zero
        } else {
            // The following line can't fail: magnitude <= self.magnitude, by
            // the if statement above, and u16::MAX == i16::MAX - i16::MIN, and
            // usize is asserted to be at least as big as u16.
            let j = (self.magnitude as i32 - magnitude as i32) as usize;
            match self.digits.get(j) {
                Some(v) => *v,
                None => 0, // Trailing zero
            }
        }
    }

    /// Gets the visible range of digit magnitudes, in ascending order of magnitude. Call `.rev()`
    /// on the return value to get the range in descending order. Magnitude 0 is always included,
    /// even if the number has leading or trailing zeros.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_decimal::FixedDecimal;
    ///
    /// let mut dec = FixedDecimal::from(120);
    /// assert_eq!(0..=2, dec.magnitude_range());
    /// ```
    pub const fn magnitude_range(&self) -> RangeInclusive<i16> {
        self.lower_magnitude..=self.upper_magnitude
    }

    /// Shift the digits by a power of 10, modifying self.
    ///
    /// Leading or trailing zeros may be added to keep the digit at magnitude 0 (the last digit
    /// before the decimal separator) visible.
    ///
    /// Can fail if the change in magnitude pushes the digits out of bounds; the magnitudes of all
    /// digits should fit in an i16.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_decimal::FixedDecimal;
    ///
    /// let mut dec = FixedDecimal::from(42);
    /// assert_eq!("42", dec.to_string());
    ///
    /// dec.multiply_pow10(3).expect("Bounds are small");
    /// assert_eq!("42000", dec.to_string());
    /// ```
    pub fn multiply_pow10(&mut self, delta: i16) -> Result<(), Error> {
        match delta.cmp(&0) {
            Ordering::Greater => {
                self.upper_magnitude = self
                    .upper_magnitude
                    .checked_add(delta)
                    .ok_or(Error::Limit)?;
                // If we get here, then the magnitude change is in-bounds.
                let lower_magnitude = self.lower_magnitude + delta;
                self.lower_magnitude = cmp::min(0, lower_magnitude);
            }
            Ordering::Less => {
                self.lower_magnitude = self
                    .lower_magnitude
                    .checked_add(delta)
                    .ok_or(Error::Limit)?;
                // If we get here, then the magnitude change is in-bounds.
                let upper_magnitude = self.upper_magnitude + delta;
                self.upper_magnitude = cmp::max(0, upper_magnitude);
            }
            Ordering::Equal => {}
        };
        self.magnitude += delta;
        #[cfg(debug_assertions)]
        self.check_invariants();
        Ok(())
    }

    /// Shift the digits by a power of 10, consuming self and returning a new object if successful.
    ///
    /// Leading or trailing zeros may be added to keep the digit at magnitude 0 (the last digit
    /// before the decimal separator) visible.
    ///
    /// Can fail if the change in magnitude pushes the digits out of bounds; the magnitudes of all
    /// digits should fit in an i16.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_decimal::FixedDecimal;
    ///
    /// let dec = FixedDecimal::from(42).multiplied_pow10(3).expect("Bounds are small");
    /// assert_eq!("42000", dec.to_string());
    /// ```
    pub fn multiplied_pow10(mut self, delta: i16) -> Result<Self, Error> {
        match self.multiply_pow10(delta) {
            Ok(()) => Ok(self),
            Err(err) => Err(err),
        }
    }

    /// Change the value from negative to positive or from positive to negative, modifying self.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_decimal::FixedDecimal;
    ///
    /// let mut dec = FixedDecimal::from(42);
    /// assert_eq!("42", dec.to_string());
    ///
    /// dec.negate();
    /// assert_eq!("-42", dec.to_string());
    ///
    /// dec.negate();
    /// assert_eq!("42", dec.to_string());
    /// ```
    pub fn negate(&mut self) {
        self.is_negative = !self.is_negative;
    }

    /// Change the value from negative to positive or from positive to negative, consuming self
    /// and returning a new object.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_decimal::FixedDecimal;
    ///
    /// assert_eq!(FixedDecimal::from(-42), FixedDecimal::from(42).negated());
    /// ```
    pub fn negated(mut self) -> Self {
        self.negate();
        self
    }

    /// Returns the [Signum][Signum] of this FixedDecimal.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_decimal::FixedDecimal;
    /// use fixed_decimal::Signum;
    ///
    /// assert_eq!(Signum::AboveZero, FixedDecimal::from(42).signum());
    /// assert_eq!(Signum::PositiveZero, FixedDecimal::from(0).signum());
    /// assert_eq!(Signum::NegativeZero, FixedDecimal::from(0).negated().signum());
    /// assert_eq!(Signum::BelowZero, FixedDecimal::from(-42).signum());
    /// ```
    pub fn signum(&self) -> Signum {
        let is_zero = self.digits.is_empty();
        match (self.is_negative, is_zero) {
            (false, false) => Signum::AboveZero,
            (false, true) => Signum::PositiveZero,
            (true, false) => Signum::BelowZero,
            (true, true) => Signum::NegativeZero,
        }
    }

    /// Assert that the invariants among struct fields are enforced. Returns true if all are okay.
    /// Call this in any method that mutates the struct fields.
    ///
    /// Example: `debug_assert!(self.check_invariants())`
    #[cfg(debug_assertions)]
    fn check_invariants(&self) {
        // magnitude invariants:
        debug_assert!(self.upper_magnitude >= self.magnitude, "{:?}", self);
        debug_assert!(self.lower_magnitude <= self.magnitude, "{:?}", self);
        debug_assert!(self.upper_magnitude >= 0, "{:?}", self);
        debug_assert!(self.lower_magnitude <= 0, "{:?}", self);

        // digits invariants:
        let max_len = (self.magnitude as i32 - self.lower_magnitude as i32 + 1) as usize;
        debug_assert!(self.digits.len() <= max_len, "{:?}", self);
        if !self.digits.is_empty() {
            debug_assert_ne!(self.digits[0], 0, "{:?}", self);
            debug_assert_ne!(self.digits[self.digits.len() - 1], 0, "{:?}", self);
        }
    }
}

impl writeable::Writeable for FixedDecimal {
    /// Render the `FixedDecimal` as a string of ASCII digits with a possible decimal point.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_decimal::FixedDecimal;
    /// use writeable::Writeable;
    ///
    /// let dec = FixedDecimal::from(42);
    /// let mut result = String::with_capacity(dec.write_len().capacity());
    /// dec.write_to(&mut result).expect("write_to(String) should not fail");
    /// assert_eq!("42", result);
    /// ```
    fn write_to<W: fmt::Write + ?Sized>(&self, sink: &mut W) -> fmt::Result {
        if self.is_negative {
            sink.write_char('-')?;
        }
        for m in self.magnitude_range().rev() {
            if m == -1 {
                sink.write_char('.')?;
            }
            let d = self.digit_at(m);
            sink.write_char((b'0' + d) as char)?;
        }
        Ok(())
    }

    /// The number of bytes that will be written by `FixedDecimal::write_to`. Use this function to
    /// pre-allocate capacity in the destination buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_decimal::FixedDecimal;
    /// use writeable::Writeable;
    /// use writeable::LengthHint;
    ///
    /// let dec = FixedDecimal::from(-5000).multiplied_pow10(-2).expect("Bounds are small");
    /// let result = dec.writeable_to_string();
    /// assert_eq!(LengthHint::Exact(6), dec.write_len());
    /// ```
    fn write_len(&self) -> writeable::LengthHint {
        writeable::LengthHint::Exact(1)
            + ((self.upper_magnitude as i32 - self.lower_magnitude as i32) as usize)
            + (if self.is_negative { 1 } else { 0 })
            + (if self.lower_magnitude < 0 { 1 } else { 0 })
    }
}

/// Renders the `FixedDecimal` according to the syntax documented in `FixedDecimal::write_to`.
impl fmt::Display for FixedDecimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeable::Writeable::write_to(self, f)
    }
}

impl FromStr for FixedDecimal {
    type Err = Error;
    fn from_str(input_str: &str) -> Result<Self, Self::Err> {
        // input_str: the input string
        // no_sign_str: the input string when the sign is removed from it
        // Check if the input string is "" or "-"
        if input_str.is_empty() || input_str == "-" {
            return Err(Error::Syntax);
        }
        let input_str = input_str.as_bytes();
        let is_negative = input_str[0] == b'-';
        let no_sign_str = if is_negative {
            &input_str[1..]
        } else {
            input_str
        };
        // Compute length of each string once and store it, so if you use that multiple times,
        // you don't compute it multiple times
        // has_dot: shows if your input has dot in it
        // dot_index: gives the index of dot (after removing the sign) -- equal to lenght of
        // the no_sign_str if there is no dot
        let no_sign_str_len = no_sign_str.len();
        let mut has_dot = false;
        let mut dot_index = no_sign_str_len;
        // The following loop computes has_dot, dot_index, and also checks to see if all
        // characters are digits and if you have at most one dot
        // Note: Input of format 111_123 is detected as syntax error here
        // Note: Input starting or ending with a dot is detected as syntax error here (Ex: .123, 123.)
        for (i, c) in no_sign_str.iter().enumerate() {
            if *c == b'.' {
                match has_dot {
                    false => {
                        dot_index = i;
                        has_dot = true;
                        if i == 0 || i == no_sign_str.len() - 1 {
                            return Err(Error::Syntax);
                        }
                    }
                    true => {
                        return Err(Error::Syntax);
                    }
                }
            } else if *c < b'0' || *c > b'9' {
                return Err(Error::Syntax);
            }
        }

        // defining the output dec here and set its sign
        let mut dec = Self {
            is_negative,
            ..Default::default()
        };

        // no_dot_str_len: shows length of the string after removing the dot
        let mut no_dot_str_len = no_sign_str_len;
        if has_dot {
            no_dot_str_len -= 1;
        }

        // Computing DecimalFixed.upper_magnitude
        let temp_upper_magnitude = dot_index - 1;
        if temp_upper_magnitude > i16::MAX as usize {
            return Err(Error::Limit);
        }
        dec.upper_magnitude = temp_upper_magnitude as i16;

        // Computing DecimalFixed.lower_magnitude
        // Note: ((i16::MIN as u16) as usize) == 32768
        let temp_lower_magnitude = no_dot_str_len - dot_index;
        if temp_lower_magnitude > (i16::MIN as u16) as usize {
            return Err(Error::Limit);
        }
        dec.lower_magnitude = (temp_lower_magnitude as i16).wrapping_neg();

        // leftmost_digit: index of the first non-zero digit
        // rightmost_digit: index of the first element after the last non-zero digit
        // Example:
        //     input string    leftmost_digit     rightmost_digit
        //     00123000              2                  5
        //     0.0123000             3                  6
        //     001.23000             2                  6
        //     001230.00             2                  5
        // Compute leftmost_digit
        let mut leftmost_digit = no_sign_str_len;
        for (i, c) in no_sign_str.iter().enumerate() {
            if *c == b'.' {
                continue;
            }
            if *c != b'0' {
                leftmost_digit = i;
                break;
            }
        }

        // If the input only has zeros (like 000, 00.0, -00.000) we handle the situation here
        // by returning the dec and don't running the rest of the code
        if leftmost_digit == no_sign_str_len {
            return Ok(dec);
        }

        // Else if the input is not all zeros, we compute its magnitude:
        // Note that we can cast with "as" here because lower and upper magnitude have been checked already
        let mut temp_magnitude = ((dot_index as i32) - (leftmost_digit as i32) - 1i32) as i16;
        if dot_index < leftmost_digit {
            temp_magnitude += 1;
        }
        dec.magnitude = temp_magnitude;

        // Compute rightmost_digit
        let mut rightmost_digit = no_sign_str_len;
        for (i, c) in no_sign_str.iter().rev().enumerate() {
            if *c == b'.' {
                continue;
            }
            if *c != b'0' {
                rightmost_digit = no_sign_str_len - i;
                break;
            }
        }

        // digits_str_len: shows the length of digits (Ex. 0012.8900 --> 4)
        let mut digits_str_len = rightmost_digit - leftmost_digit;
        if leftmost_digit < dot_index && dot_index < rightmost_digit {
            digits_str_len -= 1;
        }

        // Constructing DecimalFixed.digits
        let mut v: SmallVec<[u8; 8]> = SmallVec::with_capacity(digits_str_len);
        for c in no_sign_str[leftmost_digit..rightmost_digit].iter() {
            if *c == b'.' {
                continue;
            }
            v.push(c - b'0');
        }
        let v_len = v.len();
        debug_assert_eq!(v_len, digits_str_len);
        dec.digits = v;

        Ok(dec)
    }
}

#[test]
fn test_basic() {
    #[derive(Debug)]
    struct TestCase {
        pub input: isize,
        pub delta: i16,
        pub expected: &'static str,
    }
    let cases = [
        TestCase {
            input: 51423,
            delta: 0,
            expected: "51423",
        },
        TestCase {
            input: 51423,
            delta: -2,
            expected: "514.23",
        },
        TestCase {
            input: 51423,
            delta: -5,
            expected: "0.51423",
        },
        TestCase {
            input: 51423,
            delta: -8,
            expected: "0.00051423",
        },
        TestCase {
            input: 51423,
            delta: 3,
            expected: "51423000",
        },
        TestCase {
            input: 0,
            delta: 0,
            expected: "0",
        },
        TestCase {
            input: 0,
            delta: -2,
            expected: "0.00",
        },
        TestCase {
            input: 0,
            delta: 3,
            expected: "0000",
        },
        TestCase {
            input: 500,
            delta: 0,
            expected: "500",
        },
        TestCase {
            input: 500,
            delta: -1,
            expected: "50.0",
        },
        TestCase {
            input: 500,
            delta: -2,
            expected: "5.00",
        },
        TestCase {
            input: 500,
            delta: -3,
            expected: "0.500",
        },
        TestCase {
            input: 500,
            delta: -4,
            expected: "0.0500",
        },
        TestCase {
            input: 500,
            delta: 3,
            expected: "500000",
        },
        TestCase {
            input: -123,
            delta: 0,
            expected: "-123",
        },
        TestCase {
            input: -123,
            delta: -2,
            expected: "-1.23",
        },
        TestCase {
            input: -123,
            delta: -5,
            expected: "-0.00123",
        },
        TestCase {
            input: -123,
            delta: 3,
            expected: "-123000",
        },
    ];
    for cas in &cases {
        let mut dec: FixedDecimal = cas.input.into();
        // println!("{}", cas.input + 0.01);
        dec.multiply_pow10(cas.delta).unwrap();
        writeable::assert_writeable_eq!(cas.expected, dec, "{:?}", cas);
    }
}

#[test]
fn test_from_str() {
    #[derive(Debug)]
    struct TestCase {
        pub input_str: &'static str,
    }
    let cases = [
        TestCase {
            input_str: "-00123400",
        },
        TestCase {
            input_str: "0.0123400",
        },
        TestCase {
            input_str: "-00.123400",
        },
        TestCase {
            input_str: "0012.3400",
        },
        TestCase {
            input_str: "-0012340.0",
        },
        TestCase { input_str: "1234" },
        TestCase {
            input_str: "0.000000001",
        },
        TestCase {
            input_str: "0.0000000010",
        },
        TestCase {
            input_str: "1000000",
        },
        TestCase {
            input_str: "10000001",
        },
        TestCase { input_str: "123" },
        TestCase {
            input_str: "922337203685477580898230948203840239384.9823094820384023938423424",
        },
        TestCase {
            input_str: "009223372000.003685477580898230948203840239384000",
        },
        TestCase {
            input_str: "009223372000.003685477580898230948203840239384000",
        },
        TestCase { input_str: "0" },
        TestCase { input_str: "-0" },
        TestCase { input_str: "000" },
        TestCase { input_str: "-00.0" },
    ];
    for cas in &cases {
        let input_str_roundtrip = FixedDecimal::from_str(cas.input_str).unwrap().to_string();
        assert_eq!(cas.input_str, input_str_roundtrip);
    }
}

#[test]
fn test_isize_limits() {
    for num in &[std::isize::MAX, std::isize::MIN] {
        let dec: FixedDecimal = (*num).into();
        let dec_str = dec.to_string();
        assert_eq!(num.to_string(), dec_str);
        assert_eq!(dec, FixedDecimal::from_str(&dec_str).unwrap());
        writeable::assert_writeable_eq!(dec_str, dec);
    }
}

#[test]
fn test_ui128_limits() {
    for num in &[std::i128::MAX, std::i128::MIN] {
        let dec: FixedDecimal = (*num).into();
        let dec_str = dec.to_string();
        assert_eq!(num.to_string(), dec_str);
        assert_eq!(dec, FixedDecimal::from_str(&dec_str).unwrap());
        writeable::assert_writeable_eq!(dec_str, dec);
    }
    for num in &[std::u128::MAX, std::u128::MIN] {
        let dec: FixedDecimal = (*num).into();
        let dec_str = dec.to_string();
        assert_eq!(num.to_string(), dec_str);
        assert_eq!(dec, FixedDecimal::from_str(&dec_str).unwrap());
        writeable::assert_writeable_eq!(dec_str, dec);
    }
}

#[test]
fn test_upper_magnitude_bounds() {
    let mut dec: FixedDecimal = 98765.into();
    assert_eq!(dec.upper_magnitude, 4);
    dec.multiply_pow10(32763).unwrap();
    assert_eq!(dec.upper_magnitude, std::i16::MAX);
    let dec_backup = dec.clone();
    assert_eq!(Error::Limit, dec.multiply_pow10(1).unwrap_err());
    assert_eq!(dec, dec_backup, "Value should be unchanged on failure");

    // Checking from_str for dec (which is valid)
    let dec_roundtrip = FixedDecimal::from_str(&dec.to_string()).unwrap();
    assert_eq!(dec, dec_roundtrip);
}

#[test]
fn test_lower_magnitude_bounds() {
    let mut dec: FixedDecimal = 98765.into();
    assert_eq!(dec.lower_magnitude, 0);
    dec.multiply_pow10(-32768).unwrap();
    assert_eq!(dec.lower_magnitude, std::i16::MIN);
    let dec_backup = dec.clone();
    assert_eq!(Error::Limit, dec.multiply_pow10(-1).unwrap_err());
    assert_eq!(dec, dec_backup, "Value should be unchanged on failure");

    // Checking from_str for dec (which is valid)
    let dec_roundtrip = FixedDecimal::from_str(&dec.to_string()).unwrap();
    assert_eq!(dec, dec_roundtrip);
}

#[test]
fn test_zero_str_bounds() {
    #[derive(Debug)]
    struct TestCase {
        pub zeros_before_dot: usize,
        pub zeros_after_dot: usize,
        pub expected_err: Option<Error>,
    }
    // Note that std::i16::MAX = 32768
    let cases = [
        TestCase {
            zeros_before_dot: 32768,
            zeros_after_dot: 0,
            expected_err: None,
        },
        TestCase {
            zeros_before_dot: 32767,
            zeros_after_dot: 0,
            expected_err: None,
        },
        TestCase {
            zeros_before_dot: 32769,
            zeros_after_dot: 0,
            expected_err: Some(Error::Limit),
        },
        TestCase {
            zeros_before_dot: 0,
            zeros_after_dot: 32769,
            expected_err: Some(Error::Limit),
        },
        TestCase {
            zeros_before_dot: 32768,
            zeros_after_dot: 32768,
            expected_err: None,
        },
        TestCase {
            zeros_before_dot: 32769,
            zeros_after_dot: 32768,
            expected_err: Some(Error::Limit),
        },
        TestCase {
            zeros_before_dot: 32768,
            zeros_after_dot: 32769,
            expected_err: Some(Error::Limit),
        },
        TestCase {
            zeros_before_dot: 32767,
            zeros_after_dot: 32769,
            expected_err: Some(Error::Limit),
        },
        TestCase {
            zeros_before_dot: 32767,
            zeros_after_dot: 32767,
            expected_err: None,
        },
        TestCase {
            zeros_before_dot: 32768,
            zeros_after_dot: 32767,
            expected_err: None,
        },
    ];
    for cas in &cases {
        let mut input_str = format!("{:0fill$}", 0, fill = cas.zeros_before_dot);
        if cas.zeros_after_dot > 0 {
            input_str.push('.');
            input_str.push_str(&format!("{:0fill$}", 0, fill = cas.zeros_after_dot));
        }
        match FixedDecimal::from_str(&input_str) {
            Ok(dec) => {
                assert_eq!(cas.expected_err, None, "{:?}", cas);
                assert_eq!(input_str, dec.to_string(), "{:?}", cas);
            }
            Err(err) => {
                assert_eq!(cas.expected_err, Some(err), "{:?}", cas);
            }
        }
    }
}

#[test]
fn test_syntax_error() {
    #[derive(Debug)]
    struct TestCase {
        pub input_str: &'static str,
        pub expected_err: Option<Error>,
    }
    let cases = [
        TestCase {
            input_str: "-12a34",
            expected_err: Some(Error::Syntax),
        },
        TestCase {
            input_str: "0.0123√400",
            expected_err: Some(Error::Syntax),
        },
        TestCase {
            input_str: "0.012.3400",
            expected_err: Some(Error::Syntax),
        },
        TestCase {
            input_str: "-0-0123400",
            expected_err: Some(Error::Syntax),
        },
        TestCase {
            input_str: "0-0123400",
            expected_err: Some(Error::Syntax),
        },
        TestCase {
            input_str: "-.00123400",
            expected_err: Some(Error::Syntax),
        },
        TestCase {
            input_str: "-0.00123400",
            expected_err: None,
        },
        TestCase {
            input_str: ".00123400",
            expected_err: Some(Error::Syntax),
        },
        TestCase {
            input_str: "00123400.",
            expected_err: Some(Error::Syntax),
        },
        TestCase {
            input_str: "00123400.0",
            expected_err: None,
        },
        TestCase {
            input_str: "123_456",
            expected_err: Some(Error::Syntax),
        },
        TestCase {
            input_str: "",
            expected_err: Some(Error::Syntax),
        },
        TestCase {
            input_str: "-",
            expected_err: Some(Error::Syntax),
        },
        TestCase {
            input_str: "-1",
            expected_err: None,
        },
    ];
    for cas in &cases {
        match FixedDecimal::from_str(cas.input_str) {
            Ok(dec) => {
                assert_eq!(cas.expected_err, None, "{:?}", cas);
                assert_eq!(cas.input_str, dec.to_string(), "{:?}", cas);
            }
            Err(err) => {
                assert_eq!(cas.expected_err, Some(err), "{:?}", cas);
            }
        }
    }
}

#[test]
fn test_signum_zero() {
    #[derive(Debug)]
    struct TestCase {
        pub fixed_decimal: FixedDecimal,
        pub expected_signum: Signum,
    }
    let cases = [
        TestCase {
            fixed_decimal: Default::default(),
            expected_signum: Signum::PositiveZero,
        },
        TestCase {
            fixed_decimal: FixedDecimal::from(0),
            expected_signum: Signum::PositiveZero,
        },
        TestCase {
            fixed_decimal: FixedDecimal::from(0).negated(),
            expected_signum: Signum::NegativeZero,
        },
        TestCase {
            fixed_decimal: FixedDecimal::from_str("0").unwrap(),
            expected_signum: Signum::PositiveZero,
        },
        TestCase {
            fixed_decimal: FixedDecimal::from_str("000").unwrap(),
            expected_signum: Signum::PositiveZero,
        },
        TestCase {
            fixed_decimal: FixedDecimal::from_str("-0.000").unwrap(),
            expected_signum: Signum::NegativeZero,
        },
        TestCase {
            fixed_decimal: FixedDecimal::from_str("000.000").unwrap(),
            expected_signum: Signum::PositiveZero,
        },
    ];
    for cas in &cases {
        let signum = cas.fixed_decimal.signum();
        assert_eq!(cas.expected_signum, signum, "{:?}", cas);
    }
}

#[test]
fn test_from_float_ryu() {
    #[derive(Debug)]
    struct TestCase {
        pub float: f32,
        pub post_decimal_places: u16,
        pub expected_decimal: FixedDecimal,
    }
    let cases = [
        TestCase {
            float: 12.34,
            post_decimal_places: 2,
            expected_decimal: FixedDecimal::from_str("12.34").unwrap(),
        },
        TestCase {
            float: 12.34,
            post_decimal_places: 1,
            expected_decimal: FixedDecimal::from_str("12.3").unwrap(),
        },
        TestCase {
            float: 12.34,
            post_decimal_places: 3,
            expected_decimal: FixedDecimal::from_str("12.340").unwrap(),
        },
        TestCase {
            float: 1234.0,
            post_decimal_places: 0,
            expected_decimal: FixedDecimal::from_str("1234").unwrap(),
        },
        TestCase {
            float: 1234.0,
            post_decimal_places: 1,
            expected_decimal: FixedDecimal::from_str("1234.0").unwrap(),
        },
        TestCase {
            float: 12.34,
            post_decimal_places: 100,
            expected_decimal: FixedDecimal::from_str(format!("12.34{}", "0".repeat(98)).as_str())
                .unwrap(),
        },
        TestCase {
            float: -12.34,
            post_decimal_places: 2,
            expected_decimal: FixedDecimal::from_str("-12.34").unwrap(),
        },
        TestCase {
            float: -12.34,
            post_decimal_places: 1,
            expected_decimal: FixedDecimal::from_str("-12.3").unwrap(),
        },
        TestCase {
            float: -12.34,
            post_decimal_places: 3,
            expected_decimal: FixedDecimal::from_str("-12.340").unwrap(),
        },
    ];
    for cas in &cases {
        let to_decimal = FixedDecimal::from_float_ryu(cas.float, cas.post_decimal_places).unwrap();
        assert_eq!(cas.expected_decimal, to_decimal, "{:?}", cas);
    }
}

#[test]
fn test_from_double_ryu() {
    #[derive(Debug)]
    struct TestCase {
        pub double: f64,
        pub post_decimal_places: u16,
        pub expected_decimal: FixedDecimal,
    }
    let cases = [
        TestCase {
            double: 12.34,
            post_decimal_places: 2,
            expected_decimal: FixedDecimal::from_str("12.34").unwrap(),
        },
        TestCase {
            double: 12.34,
            post_decimal_places: 1,
            expected_decimal: FixedDecimal::from_str("12.3").unwrap(),
        },
        TestCase {
            double: 12.34,
            post_decimal_places: 3,
            expected_decimal: FixedDecimal::from_str("12.340").unwrap(),
        },
        TestCase {
            double: 1234.0,
            post_decimal_places: 0,
            expected_decimal: FixedDecimal::from_str("1234").unwrap(),
        },
        TestCase {
            double: 1234.0,
            post_decimal_places: 1,
            expected_decimal: FixedDecimal::from_str("1234.0").unwrap(),
        },
        TestCase {
            double: 12.34,
            post_decimal_places: 100,
            expected_decimal: FixedDecimal::from_str(format!("12.34{}", "0".repeat(98)).as_str())
                .unwrap(),
        },
        TestCase {
            double: -12.34,
            post_decimal_places: 2,
            expected_decimal: FixedDecimal::from_str("-12.34").unwrap(),
        },
        TestCase {
            double: -12.34,
            post_decimal_places: 1,
            expected_decimal: FixedDecimal::from_str("-12.3").unwrap(),
        },
        TestCase {
            double: -12.34,
            post_decimal_places: 3,
            expected_decimal: FixedDecimal::from_str("-12.340").unwrap(),
        },
    ];
    for cas in &cases {
        let to_decimal =
            FixedDecimal::from_double_ryu(cas.double, cas.post_decimal_places).unwrap();
        assert_eq!(cas.expected_decimal, to_decimal, "{:?}", cas);
    }
}
