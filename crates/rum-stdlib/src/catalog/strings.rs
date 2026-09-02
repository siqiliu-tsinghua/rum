use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec};

use super::message_templates::*;

use crate::strings;

const ATTR_PROTECTED: AttributeSet = AttributeSet::from_attribute(Attribute::Protected);

pub(crate) const STRING_CORE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::with_attributes(
        "Alternatives",
        "Represents the current pattern alternative head.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "Blank",
        "Represents the current single-element pattern atom; in string patterns it matches one character.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "BlankNullSequence",
        "Represents the current zero-or-more sequence pattern atom; in string patterns it matches zero or more characters.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "BlankSequence",
        "Represents the current one-or-more sequence pattern atom; in string patterns it matches one or more characters.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "DigitCharacter",
        "Represents the current string-pattern atom for one ASCII decimal digit.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "EndOfLine",
        "Represents the current string-pattern anchor for the end of a string or a position before LF.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "EndOfString",
        "Represents the current string-pattern anchor for the end of a string.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "Except",
        "Represents the current pattern exclusion head.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "LetterCharacter",
        "Represents the current string-pattern atom for one alphabetic character.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "Pattern",
        "Represents the current named pattern-binding head.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "PunctuationCharacter",
        "Represents the current string-pattern atom for one ASCII punctuation character.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "RegularExpression",
        "Represents the current regular-expression string-pattern wrapper.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "StartOfLine",
        "Represents the current string-pattern anchor for the start of a string or a position after LF.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "StartOfString",
        "Represents the current string-pattern anchor for the start of a string.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "StringExpression",
        "Represents the current string-pattern sequence head produced by `~~` syntax.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "WhitespaceCharacter",
        "Represents the current string-pattern atom for one whitespace character.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "WordBoundary",
        "Represents the current string-pattern anchor at a transition between word and non-word text.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "WordCharacter",
        "Represents the current string-pattern atom for one alphabetic, ASCII digit, or underscore character.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::ordinary(
        "StringJoin",
        "Concatenates strings or a list of strings into a single string.",
        strings::eval_string_join,
        MemoryGrowthContract::Preflight,
    ).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "StringLength",
        "Returns the Unicode code-point length of a string.",
        AttributeSet::from_attribute(Attribute::Listable),
        strings::eval_string_length,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "StringReverse",
        "Reverses the Unicode code-point characters in a string.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        strings::eval_string_reverse,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "ToLowerCase",
        "Converts a string to lowercase using the current Unicode-aware subset.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        strings::eval_to_lower_case,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "ToUpperCase",
        "Converts a string or list of strings to uppercase using the current Unicode-aware subset.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        strings::eval_to_upper_case,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "Capitalize",
        "Uppercases the first character of a string or each string in a list.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        strings::eval_capitalize,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "Decapitalize",
        "Lowercases the first character of a string or each string in a list.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        strings::eval_decapitalize,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary(
        "StringForm",
        "Formats a template string eagerly, replacing `` and `n` placeholders with ToString-rendered arguments.",
        strings::eval_string_form,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "slot",
            template: "Reports a missing or invalid template slot.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringWrap",
        "Greedily word-wraps a string to a positive character width, separating output lines with LF. With \"PreserveNewlines\" -> True, wraps each LF-delimited line independently and preserves blank or whitespace-only lines verbatim; the default False collapses all whitespace as word separators.",
        strings::eval_string_wrap,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "width must be a positive integer",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringIndent",
        "Prefixes each non-blank line of a string with a string prefix or an exact number of spaces. With \"BlankLines\" -> True, blank and whitespace-only lines are prefixed too; the default False leaves them unchanged.",
        strings::eval_string_indent,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringDedent",
        "Removes the longest common leading whitespace from the non-blank lines of a string.",
        strings::eval_string_dedent,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringExpandTabs",
        "Expands tabs in a string to the next fixed-width column stop, defaulting to width 8.",
        strings::eval_string_expand_tabs,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "tab width must be a positive integer",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "AlignColumns",
        "Formats a list of string rows as left-aligned columns separated by \"Separator\" -> \"  \" by default; any string separator can be supplied.",
        strings::eval_align_columns,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "DecimalString",
        "Renders an exact number as a plain decimal string with exactly n fractional digits.",
        strings::eval_decimal_string,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "HtmlEscape",
        "Escapes the five HTML text characters &, <, >, double quote, and single quote.",
        strings::eval_html_escape,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "HtmlUnescape",
        "Unescapes the five named HTML entities and decimal or hexadecimal numeric character references, leaving unknown and malformed entities verbatim by default; with \"Strict\" -> True, malformed entities fail closed with $Failed.",
        strings::eval_html_unescape,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "entity",
            template: "Reports an invalid HTML entity.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "ShellSplit",
        "Splits a POSIX-style shell word string with whitespace, quotes, and backslash escapes, returning $Failed on malformed quoting.",
        strings::eval_shell_split,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "syntax",
            template: "Reports invalid input syntax.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "ShellQuote",
        "Single-quotes a string for POSIX-style shell input, escaping embedded single quotes.",
        strings::eval_shell_quote,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "FileExtension",
        "Returns the final path component extension without the dot, or an empty string when no extension is present.",
        strings::eval_file_extension,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "FileBaseName",
        "Returns the final path component without its extension.",
        strings::eval_file_base_name,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "FileNameSplit",
        "Splits a slash-separated path string into components, preserving an absolute leading slash as its own component.",
        strings::eval_file_name_split,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "UrlEncode",
        "Percent-encodes a string using UTF-8 bytes and the RFC 3986 unreserved character set; with \"FormEncoding\" -> True, spaces are encoded as + for form query data.",
        strings::eval_url_encode,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "UrlDecode",
        "Decodes percent escapes in a UTF-8 URL string, returning $Failed for invalid escapes or invalid UTF-8; with \"FormEncoding\" -> True, + decodes to a space.",
        strings::eval_url_decode,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "escape",
            template: "Reports an invalid escape sequence.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "UrlParse",
        "Parses scheme://host[:port][/path][?query][#fragment] into an association of string fields; \"DefaultScheme\" can supply a scheme for schemeless inputs, and \"DecodeQuery\" decodes query text using form encoding.",
        strings::eval_url_parse,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "escape",
            template: "Reports an invalid escape sequence.",
        },
        BuiltinMessageSpec {
            tag: "syntax",
            template: "Reports invalid input syntax.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "ParseArgs",
        "Parses script arguments into positional arguments, --key=value options, and --flag names; with \"ShortOptions\" -> True, single-dash clusters become single-letter flags, without attached-value syntax.",
        strings::eval_parse_args,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Row",
        "Renders a list of expressions as a single text string in the current harness subset.",
        strings::eval_row,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
];
