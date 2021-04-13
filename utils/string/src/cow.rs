use crate::Slice;
use std::{borrow::Cow, ops::Range};

impl<'s> Slice<'s, &'s str> for Cow<'s, str> {
    fn get_byte(&self, idx: usize) -> Option<u8> {
        self.as_bytes().get(idx).copied()
    }

    fn get_str_slice(&self, range: Range<usize>) -> &str {
        &self[range]
    }

    fn get_slice(&'s self, range: Range<usize>) -> &'s str {
        match self {
            Self::Borrowed(b) => &b[range],
            Self::Owned(o) => &o[range],
        }
    }

    fn length(&self) -> usize {
        self.len()
    }
}

impl<'s> Slice<'s, String> for Cow<'s, str> {
    fn get_byte(&self, idx: usize) -> Option<u8> {
        self.as_bytes().get(idx).copied()
    }

    fn get_str_slice(&self, range: Range<usize>) -> &str {
        &self[range]
    }

    fn get_slice(&'s self, range: Range<usize>) -> String {
        self[range].to_string()
    }

    fn length(&self) -> usize {
        self.len()
    }
}

impl<'s> Slice<'s, Cow<'s, str>> for Cow<'s, str> {
    fn get_byte(&self, idx: usize) -> Option<u8> {
        self.as_bytes().get(idx).copied()
    }

    fn get_str_slice(&self, range: Range<usize>) -> &str {
        &self[range]
    }

    fn get_slice(&'s self, range: Range<usize>) -> Cow<'s, str> {
        match self {
            Self::Borrowed(b) => Cow::Borrowed(&b[range]),
            Self::Owned(o) => Cow::Owned(o[range].to_owned()),
        }
    }

    fn length(&self) -> usize {
        self.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_cow_owned<T>(input: &Cow<T>) -> bool
    where
        T: ToOwned + ?Sized,
    {
        match input {
            Cow::Borrowed(_) => false,
            Cow::Owned(_) => true,
        }
    }

    #[test]
    fn cow_borrowed_from_cow_borrowed() {
        let slice = Cow::Borrowed("Hello World");
        let result: Cow<str> = slice.get_slice(0..5);
        assert_eq!(result, "Hello");
        assert!(!is_cow_owned(&result));
    }

    #[test]
    fn cow_owned_from_cow_owned<'s>() {
        let slice: Cow<str> = Cow::Owned("Hello World".to_string());
        let result: Cow<str> = slice.get_slice(0..5);
        assert_eq!(result, "Hello");
        assert!(is_cow_owned(&result));
    }

    #[test]
    fn str_from_cow_borrowed() {
        let slice = Cow::Borrowed("Hello World");
        let result: &str = slice.get_slice(0..5);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn str_from_cow_owned() {
        let slice: Cow<str> = Cow::Owned("Hello World".to_string());
        let result: &str = slice.get_slice(0..5);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn string_from_cow_borrowed() {
        let slice = Cow::Borrowed("Hello World");
        let result: String = slice.get_slice(0..5);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn string_from_cow_owned() {
        let slice: Cow<str> = Cow::Owned("Hello World".to_string());
        let result: String = slice.get_slice(0..5);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn cow_get_byte() {
        let slice = "Hello World";
        assert_eq!(Slice::<Cow<str>>::get_byte(&slice, 0), Some(b'H'));
        assert_eq!(Slice::<Cow<str>>::get_byte(&slice, 12), None);
    }

    #[test]
    fn string_get_str_slice() {
        let slice = "Hello World";
        assert_eq!(Slice::<Cow<str>>::get_str_slice(&slice, 0..5), "Hello");
    }

    #[test]
    fn string_length() {
        let slice = "Hello World";
        assert_eq!(Slice::<Cow<str>>::length(&slice), 11);
    }
}
