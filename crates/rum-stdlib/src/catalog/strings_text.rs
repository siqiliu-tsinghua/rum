use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec};

use super::message_templates::*;

use crate::strings;

pub(crate) const STRING_TEXT_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "StringRiffle",
        "Riffles a flat string list with a string separator or {left, separator, right} wrapper, and riffles nested string rows with row and item separators.",
        strings::eval_string_riffle,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringPadLeft",
        "Pads strings on the left to a target width using a string pad or a non-empty list of non-empty string pads; list pads cycle from the first emitted pad character and the final pad piece is truncated to the target character width.",
        strings::eval_string_pad_left,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringPadRight",
        "Pads strings on the right to a target width using a string pad or a non-empty list of non-empty string pads; list pads cycle from the first emitted pad character and the final pad piece is truncated to the target character width. Values at or above width are unchanged.",
        strings::eval_string_pad_right,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringLines",
        "Splits a string on LF or CRLF newlines, preserving interior empty lines.",
        strings::eval_string_lines,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringTemplate",
        "Builds a simple TemplateObject wrapper from a template string in the current first-pass subset.",
        strings::eval_string_template,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "TemplateApply",
        "Applies a simple named or positional string template in the current first-pass subset; unfilled slots leave the call inert by default, while \"Strict\" -> True reports them and returns $Failed.",
        strings::eval_template_apply,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "slot",
            template: "Reports a missing or invalid template slot.",
        },
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "TextWords",
        "Splits text into alphanumeric word runs in the current first-pass subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        strings::eval_text_words,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "ToCharacterCode",
        "Returns Unicode scalar values for strings, or UTF-8 byte values when given the \"UTF8\" or \"UTF-8\" encoding argument.",
        AttributeSet::from_attribute(Attribute::Listable),
        strings::eval_to_character_code,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "ToExpression",
        "Parses and evaluates input strings using the current InputForm/StandardForm-compatible subset.",
        strings::eval_to_expression,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringToByteArray",
        "Converts strings to ByteArray objects using UTF8/UTF-8 encoding.",
        strings::eval_string_to_byte_array,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    #[cfg(feature = "hash")]
    BuiltinSpec::ordinary(
        "Hash",
        "Computes SHA256, SHA1, SHA512, MD5, or CRC32 hashes over the current string/expression subset.",
        strings::eval_hash,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Compress",
        "Encodes expressions into the current project-local compressed string envelope.",
        strings::eval_compress,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Uncompress",
        "Recovers values from the current project-local compressed string envelope; malformed envelopes stay inert by default, while \"Strict\" -> True reports them and returns $Failed.",
        strings::eval_uncompress,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "env",
            template: "Uncompress found a malformed or corrupt compressed string envelope.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Nearest",
        "Finds nearest strings or exact numeric values in the current first-pass subset.",
        strings::eval_nearest,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Pluralize",
        "Applies a simple English pluralization subset.",
        strings::eval_pluralize,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "SyntaxQ",
        "Tests whether a string parses successfully in the current syntax subset.",
        strings::eval_syntax_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "RumTokens",
        "Tokenizes rum source text into total, trivia-preserving lexer spans.",
        strings::eval_rum_tokens,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "tok",
            template: "Reports a tokenizer failure.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "TokenizeWith",
        "Scans a string with string-pattern rules and returns total 1-based token spans.",
        strings::eval_tokenize_with,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "tok",
            template: "Reports a tokenizer failure.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "TextString",
        "Converts expressions to the current canonical text subset, leaving raw strings unquoted.",
        strings::eval_text_string,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "CharacterCounts",
        "Counts code-point occurrences in a string and returns an association in the current first-pass subset.",
        strings::eval_character_counts,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "RemoveDiacritics",
        "Removes common Latin diacritics from strings in the current first-pass subset.",
        strings::eval_remove_diacritics,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Alphabet",
        "Returns the current English lowercase alphabet subset.",
        strings::eval_alphabet,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "AlphabeticSort",
        "Sorts strings using the current English alphabet subset with simple diacritic folding.",
        strings::eval_alphabetic_sort,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "EditDistance",
        "Computes unit-cost Levenshtein distance for strings or lists in the current first-pass subset.",
        strings::eval_edit_distance,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "HammingDistance",
        "Computes mismatch counts for equal-length strings or lists in the current first-pass subset; unequal lengths emit a message and leave the call inert.",
        strings::eval_hamming_distance,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "len",
            template: "HammingDistance expects inputs with the same length in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "LongestCommonSubsequence",
        "Finds the longest common contiguous substring or sublist in the current first-pass subset.",
        strings::eval_longest_common_subsequence,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "SequenceAlignment",
        "Builds a simple global alignment summary for strings or lists in the current first-pass subset.",
        strings::eval_sequence_alignment,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Diff",
        "Builds a simple DiffObject for strings or lists in the current first-pass subset.",
        strings::eval_diff,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
];
