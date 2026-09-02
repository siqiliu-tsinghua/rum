use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};

const ATTR_PROTECTED: AttributeSet = AttributeSet::from_attribute(Attribute::Protected);

pub(crate) const CHARACTER_CLASS_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::with_attributes(
        "Alphanumeric",
        "Evaluates to the uppercase, lowercase, and digit character-class string.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "DigitCharacters",
        "Evaluates to the ASCII digit character-class string.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "LowercaseLetters",
        "Evaluates to the lowercase ASCII letter character-class string.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "PunctuationCharacters",
        "Evaluates to the ASCII punctuation character-class string.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "UppercaseLetters",
        "Evaluates to the uppercase ASCII letter character-class string.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::with_attributes(
        "WhitespaceCharacters",
        "Evaluates to the ASCII whitespace character-class string.",
        ATTR_PROTECTED,
        MemoryGrowthContract::NonAmplifying,
    ),
];
