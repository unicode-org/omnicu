Zones of Information in Data Provider
=====================================

## Background

This document discusses the point in the app lifecycle at which specific inputs to i18n libraries are known, with implications on data layout and fallback behavior.

## Three Zones of Information

There are three points at which information may be known to the data provider:

1. **Compile Time:** Based on the *programmer*. Can be determined by static analysis of code.
2. **Construction Time:** Based on the *end user*. Can be determined from the locale, including Unicode extensions.
3. **Format Time:** Based on the *runtime*. Can only be determined from the specific item being formatted (e.g., date or number).

### Example: Duration Formatting

Suppose you are formatting a time duration, such as a countdown timer for a video game. The breakdown of information known at each stage is:

1. **Compile Time**
    - Width (long, short, narrow, digital, …)
2. **Construction Time**
    - Locale
    - Numbering system\*
3. **Format Time**
    - Specific set of time units (hour, minute, second, …)
    - Plural form of the time units

\* The numbering system determines the choice of digits. It does not currently affect the choice of unit pattern, but it may in the future.

### Example: Date Formatting

Date formatting is more complicated, but suppose you are formatting a date without a time based on a components bag. Information breakdown:

1. **Compile Time**
    - Coarse fields (months, weekdays, time zones, …)
2. **Construction Time**
    - Locale
    - Calendar system\*
    - Width\*\* (numeric, abbreviated, wide, …)
3. **Format Time**
    - Specific month, weekday, time zone, …

\* The calendar system is special, because the program may or may not choose to support all calendar systems due to the amount of code and data that they require. Therefore, the calendar system may be known at compile time for certain applications.

\*\* The width of fields can't be known until construction time because that might depend on the locale. For a given components bag, some locales may prefer wider month names whereas others may prefer narrower ones.

## Resource Paths

The point at which information is known should influence whether that information ends up in the resource key, resource options, or data struct. As a general rule of thumb:

- If information is known at *compile time*, it should be in the *resource key*.
- If information is known at *construction time*, it should be in the *resource options*.
- If information is not known until *format time*, it should be in the data struct itself.

Exceptions to the above rules of thumb could include:

- If the amount of data is small, the pivot could be moved into the data struct. For example, different widths of currency symbols are often the same, so they could be combined into a single resource key, rather than having different resource keys for each width.

### Example: Duration Formatting

Let's take duration formatting as an example. If we consider the width to be known at compile time, the resource keys might look like this:

- `duration-long@1`
- `duration-short@1`
- `duration-narrow@1`
- `duration-digital@1`

Having separate data keys means that you can strip out all the keys you definitely don't need from your data bundle via static code analysis.

The resource options might look like this ("arab" and "latn" are examples of numbering systems):

- `ar/arab`
- `ar/latn`
- `ar-EG/arab`
- `en/latn`
- `und`

All resource options within a resource key must be included in your data bundle, since they are user-dependent, but they need not be loaded into memory until requested.

The data struct then looks something like this:

```rust
// This is as INCOMPLETE EXAMPLE struct for illustrative purposes.
// Real duration formatting data will be more detailed than this.

struct SymbolsByPlural<'s> {
    zero: Option<Cow<'s, str>>,
    one: Option<Cow<'s, str>>,
    two: Option<Cow<'s, str>>,
    few: Option<Cow<'s, str>>,
    many: Option<Cow<'s, str>>,
    other: Cow<'s, str>,
}

struct DurationSymbolsV1<'s> {
    hour: SymbolsByPlural<'s>,
    minute: SymbolsByPlural<'s>,
    second: SymbolsByPlural<'s>,
    glue: Cow<'s, str>,
}
```

This means that if you were to load `duration-long@1/en/latn.json`, you might get:

```json
{
    "hour": {
        "one": "{0} hour",
        "other": "{0} hours",
    },
    "minute": {
        "one": "{0} minute",
        "other": "{0} minutes",
    },
    "second": {
        "one": "{0} second",
        "other": "{0} seconds",
    },
    "glue": ", "
}
```

And that's the data loaded into an instance of DurationFormat. It should contain everything needed to format any arbitrary duration input, given the choices for width, locale, and numbering system given in the constructor.

## Fallbacking within Zone 2 (Resource Options)

Resource keys are hard-coded at compile time, and data structs are transparent to the data provider.  All fallbacking that the data provider is responsible for, therefore, happens in Zone 2, the Resource Options.

Resource options contain two types of fields: the language identifier, and what I'm calling a "variant", which may be the numbering system, for example.

Fallbacking within the langauge identifier field is its own discussion, which is out of scope of this doc.  *For simplicity of illustration,* this doc assumes we were to implement the "en-GB, en-001, en" fallback chain found in ICU.

Suppose we had a single variant, the numbering system.  Suppose the numbering system is `arab`.  The following resource options would need to be visited, in some order:

- `en-GB/arab`
- `en-001/arab`
- `en/arab`
- `und/arab`
- `en-GB`
- `en-001`
- `en`
- `und`

The rows that don't have an explicit numbering system should contain the default numbering system for the locale, such that if a numbering system isn't provided, we can still access the symbols on a single lookup.

The question is then: in what order do we visit these locale/variant combos?

1. Fall back on language first, all the way to root (shown above)
2. Fall back on langauge first, but stop before getting to root:
    - `en-GB/arab`
    - `en-001/arab`
    - `en/arab`
    - `en-GB`
    - `en-001`
    - `en`
    - `und/arab`
    - `und`
3. Fall back on variant first:
    - `en-GB/arab`
    - `en-GB`
    - `en-001/arab`
    - `en-001`
    - `en/arab`
    - `en`
    - `und/arab`
    - `und`

If more than one of these strategies is useful, we could consider making multiple variant fields on ResourceOptions with different fallback behavior.
