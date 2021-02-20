Zones of Information in Data Provider
=====================================

## Background

Fallbacking in i18n is a complex subject. This document focuses on one specific piece: splitting information into zones, with a discussion of how fallbacking should occur within each zone.

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

Separate resource keys must be included in your data bundle, since they are user-dependent, but they need not be loaded into memory until requested.

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
    other: Option<Cow<'s, str>>,
}

struct DurationSymbolsV1<'s> {
    hour: SymbolsByPlural<'s>,
    minute: SymbolsByPlural<'s>,
    second: SymbolsByPlural<'s>,
    glue: Cow<'s, str>,
}
```

This means that if you were to load `duration=long@1/en/latn.json`, you might get:

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

CLDR specifies many different pieces of data that have one or more *variants*. Within each variant, the structure of data is the same. Examples include:

1. Month names: format and standalone
2. Decimal symbols: numbering system
3. Display widths: long, short, narrow
4. Currency symbols: formality
