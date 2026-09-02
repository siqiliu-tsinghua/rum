use super::message_templates::*;
use super::*;

pub(super) const STRING_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::new(
        "Blank",
        "Represents the current single-element pattern atom; in string patterns it matches one character.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "BlankSequence",
        "Represents the current one-or-more sequence pattern atom; in string patterns it matches one or more characters.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "BlankNullSequence",
        "Represents the current zero-or-more sequence pattern atom; in string patterns it matches zero or more characters.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "Pattern",
        "Represents the current named pattern-binding head.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "Alternatives",
        "Represents the current pattern alternative head.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new("Except", "Represents the current pattern exclusion head.", MemoryGrowthContract::NonAmplifying).protected(),
    BuiltinSpec::new(
        "StringExpression",
        "Represents the current string-pattern sequence head produced by `~~` syntax.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "RegularExpression",
        "Represents the current regular-expression string-pattern wrapper.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "Whitespace",
        "Represents the current string-pattern atom for one or more whitespace characters.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "WhitespaceCharacter",
        "Represents the current string-pattern atom for one whitespace character.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "LetterCharacter",
        "Represents the current string-pattern atom for one alphabetic character.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "DigitCharacter",
        "Represents the current string-pattern atom for one ASCII decimal digit.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "WordCharacter",
        "Represents the current string-pattern atom for one alphabetic, ASCII digit, or underscore character.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "NumberString",
        "Represents the current string-pattern atom for decimal-style numeric text.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "PunctuationCharacter",
        "Represents the current string-pattern atom for one ASCII punctuation character.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "IgnoreCase",
        "Represents the current option symbol for case-insensitive string matching.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "Overlaps",
        "Represents the current option symbol for overlapping string-match counting.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "StartOfString",
        "Represents the current string-pattern anchor for the start of a string.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "EndOfString",
        "Represents the current string-pattern anchor for the end of a string.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "StartOfLine",
        "Represents the current string-pattern anchor for the start of a string or a position after LF.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "EndOfLine",
        "Represents the current string-pattern anchor for the end of a string or a position before LF.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "WordBoundary",
        "Represents the current string-pattern anchor at a transition between word and non-word text.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "Repeated",
        "Represents the current string-pattern repetition head for one or more matches.",
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        REPEATED_RANGE,
    ])
    .protected(),
    BuiltinSpec::new(
        "RepeatedNull",
        "Represents the current string-pattern repetition head for zero or more matches.",
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        REPEATED_RANGE,
    ])
    .protected(),
    BuiltinSpec::new(
        "Shortest",
        "Represents the current string-pattern wrapper that prefers the shortest match.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "Longest",
        "Represents the current string-pattern wrapper that prefers the longest match.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "TemplateObject",
        "Represents the current simple template object wrapper returned by StringTemplate.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "DiffObject",
        "Represents the current diff result object returned by Diff.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "ByteArray",
        "Represents raw byte payloads using base64 text in the current first-pass subset.",
        MemoryGrowthContract::Preflight,
    ).protected(),
    BuiltinSpec::new(
        "InputForm",
        "Represents the current formatting symbol accepted by the current ToExpression and ToString subsets.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "OutputForm",
        "Represents the current formatting symbol accepted by the current ToString subset.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "StandardForm",
        "Represents the current formatting symbol accepted by the current ToExpression and ToString subsets.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "TraditionalForm",
        "Represents the current formatting symbol accepted by the current ToString subset.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "Automatic",
        "Represents the current default-specifier symbol shared by several first-pass string and tooling subsets.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
];
