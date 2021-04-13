`icu_string` is a utility crate of the [`ICU4X`] project.

It provides a collection of helper utilities for handling Rust strings.

## Example

```rust
use icu_string::Slice;

/// Parse an input string slice and return a name parsed out of it.
///
/// # Type parameters
///
/// - `S`: A type of a slice provided as an input: `&str`, `String` or `Cow<str>`.
/// - `RS`: Return slice: `&str`, `String` or `Cow<str>`.
///
/// # Lifetimes
///
/// - `slice`: The life time of the slice used for scenarios where the input slice has a lifetime,
///   or when the output slice has to live for as long as the input slice.
fn parse_name<'slice, S, RS>(input: &'slice S) -> RS
where
    S: Slice<'slice, RS> + ?Sized,
{
    let len = input.length();
    input.get_slice(6..len)
}

let input = "name: Example Name";
let name: &str = parse_name(&input);

assert_eq!(name, "Example Name");
```
