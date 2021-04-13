use crate::Slice;
use std::borrow::Cow;
use std::ops::Range;

impl<'s> Slice<'s, &'s str> for &'s str {
    fn get_byte(&self, idx: usize) -> Option<u8> {
        self.as_bytes().get(idx).copied()
    }

    fn get_str_slice(&self, range: Range<usize>) -> &str {
        &self[range]
    }

    fn get_slice(&'s self, range: Range<usize>) -> &'s str {
        &self[range]
    }

    fn length(&self) -> usize {
        self.len()
    }
}

impl<'s> Slice<'s, String> for &'s str {
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

impl<'s> Slice<'s, Cow<'s, str>> for &'s str {
    fn get_byte(&self, idx: usize) -> Option<u8> {
        self.as_bytes().get(idx).copied()
    }

    fn get_str_slice(&self, range: Range<usize>) -> &str {
        &self[range]
    }

    fn get_slice(&'s self, range: Range<usize>) -> Cow<'s, str> {
        Cow::Borrowed(&self[range])
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
    fn str_from_str() {
        let slice = "Hello World";
        let result: &str = slice.get_slice(0..5);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn string_from_str() {
        let slice = "Hello World";
        let slice: String = slice.get_slice(0..5);
        assert_eq!(slice, "Hello");
    }

    #[test]
    fn cow_borrowed_from_str() {
        let slice = "Hello World";
        let result: Cow<str> = slice.get_slice(0..5);
        assert_eq!(result, "Hello");
        assert!(!is_cow_owned(&result));
    }

    #[test]
    fn string_get_byte() {
        let slice = "Hello World";
        assert_eq!(Slice::<&str>::get_byte(&slice, 0), Some(b'H'));
        assert_eq!(Slice::<&str>::get_byte(&slice, 12), None);
    }

    #[test]
    fn string_get_str_slice() {
        let slice = "Hello World";
        assert_eq!(Slice::<&str>::get_str_slice(&slice, 0..5), "Hello");
    }

    #[test]
    fn string_length() {
        let slice = "Hello World";
        assert_eq!(Slice::<&str>::length(&slice), 11);
    }
}
