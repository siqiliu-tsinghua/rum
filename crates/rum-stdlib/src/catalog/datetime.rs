use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec};

use super::message_templates::*;

use crate::datetime;

pub(crate) const DATETIME_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "DateList",
        "Converts an exact Unix timestamp to a UTC date list.",
        AttributeSet::empty().with(Attribute::Protected),
        datetime::eval_date_list,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "date",
            template: "Reports a date-list conversion failure.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "DateString",
        "Formats an exact timestamp or date list as an ISO 8601 UTC string.",
        AttributeSet::empty().with(Attribute::Protected),
        datetime::eval_date_string,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected between 1 and 2 argument(s), got 0",
        },
        BuiltinMessageSpec {
            tag: "date",
            template: "Reports a date-formatting or date-value failure.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "FromDateList",
        "Converts a UTC date list {y, mo, d, h, mi, s} to an exact Unix timestamp; the inverse of DateList.",
        AttributeSet::empty().with(Attribute::Protected),
        datetime::eval_from_date_list,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "date",
            template: "Reports an invalid date list.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "FromDateString",
        "Parses the documented ISO 8601 UTC/fixed-offset subset into an exact Unix timestamp.",
        AttributeSet::empty().with(Attribute::Protected),
        datetime::eval_from_date_string,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "date",
            template: "Reports an invalid date string.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "DatePlus",
        "Adds an exact fixed-duration offset to an exact timestamp or date list.",
        AttributeSet::empty().with(Attribute::Protected),
        datetime::eval_date_plus,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "date",
            template: "Reports an invalid date value or duration.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "DateDifference",
        "Returns the exact fixed-duration difference between two timestamps or date lists.",
        AttributeSet::empty().with(Attribute::Protected),
        datetime::eval_date_difference,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "date",
            template: "Reports invalid date values or date units.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "DayOfWeek",
        "Returns the ISO weekday symbol for an exact timestamp or date list.",
        AttributeSet::empty().with(Attribute::Protected),
        datetime::eval_day_of_week,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "date",
            template: "Reports an invalid date value or weekday result.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "LeapYearQ",
        "Tests whether an exact integer year is a leap year in the proleptic Gregorian calendar.",
        AttributeSet::empty().with(Attribute::Protected),
        datetime::eval_leap_year_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "date",
            template: "Reports an invalid year value.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "DaysInMonth",
        "Returns the number of days in a proleptic Gregorian month.",
        AttributeSet::empty().with(Attribute::Protected),
        datetime::eval_days_in_month,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "date",
            template: "Reports an invalid year or month value.",
        },
    ]),
];
