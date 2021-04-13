//! This example shows a very trivial parser that can take one of the supported
//! implementations of `Slice` and return one of the supported outputs. The example
//! is very abstract, but exemplifies ability to write parsers that operate on wide
//! range of ownership/lifetime models of the input slice and can produce a range of
//! outputs that maintain lifetimes and ownership.
use icu_string::Slice;
use std::borrow::Cow;

fn parse_name<'s, S, RS>(input: &'s S) -> RS
where
    S: Slice<'s, RS> + ?Sized,
{
    let len = input.length();
    input.get_slice(6..len)
}

fn main() {
    // Input types: `&str`, `String`, owned and borrowed `Cow<str>`.
    let str_input = "name: Example Name";
    let string_input = str_input.to_string();
    let owned_cow: Cow<str> = string_input.clone().into();
    let borrowed_cow: Cow<str> = str_input.into();

    // Use the input types to produce a `&str`.
    let name: &str = parse_name(&str_input);
    assert_eq!(name, "Example Name");

    let name: &str = parse_name(&string_input);
    assert_eq!(name, "Example Name");

    let name: &str = parse_name(&owned_cow);
    assert_eq!(name, "Example Name");

    let name: &str = parse_name(&borrowed_cow);
    assert_eq!(name, "Example Name");

    // Use the input types to produce a `String`.
    let name: String = parse_name(&str_input);
    assert_eq!(name, "Example Name");

    let name: String = parse_name(&string_input);
    assert_eq!(name, "Example Name");

    let name: String = parse_name(&owned_cow);
    assert_eq!(name, "Example Name");

    let name: String = parse_name(&borrowed_cow);
    assert_eq!(name, "Example Name");

    // Use the input types to produce a `Cow<str>`.
    let name: Cow<str> = parse_name(&str_input);
    assert_eq!(name, "Example Name");

    let name: Cow<str> = parse_name(&string_input);
    assert_eq!(name, "Example Name");

    let name: Cow<str> = parse_name(&owned_cow);
    assert_eq!(name, "Example Name");

    let name: Cow<str> = parse_name(&borrowed_cow);
    assert_eq!(name, "Example Name");
}
