use rum_core::Expr;
use rum_runtime::kernel_api::{
    EvalError, FrameRootTrace, FrameRootVisitor, OptionReentry, Session,
};

/// Every merged-option consumer has one typed re-entry. The required inventory
/// test reconciles this enum, its live arms, and the suspension certificates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(crate) enum OptionConsumer {
    AlignColumns,
    Complement,
    ContainsAll,
    ContainsAny,
    ContainsExactly,
    ContainsNone,
    ContainsOnly,
    DisjointQ,
    DuplicateFreeQ,
    HtmlUnescape,
    IntersectingQ,
    Intersection,
    ParseArgs,
    StringCases,
    StringContainsQ,
    StringCount,
    StringDelete,
    StringEndsQ,
    StringFreeQ,
    StringIndent,
    StringMatchQ,
    StringPosition,
    StringReplace,
    StringSplit,
    StringStartsQ,
    StringWrap,
    SubsetQ,
    TemplateApply,
    Uncompress,
    Union,
    UrlDecode,
    UrlEncode,
    UrlParse,
}

impl OptionConsumer {
    pub(crate) fn symbol_name(self) -> &'static str {
        match self {
            Self::AlignColumns => "AlignColumns",
            Self::Complement => "Complement",
            Self::ContainsAll => "ContainsAll",
            Self::ContainsAny => "ContainsAny",
            Self::ContainsExactly => "ContainsExactly",
            Self::ContainsNone => "ContainsNone",
            Self::ContainsOnly => "ContainsOnly",
            Self::DisjointQ => "DisjointQ",
            Self::DuplicateFreeQ => "DuplicateFreeQ",
            Self::HtmlUnescape => "HtmlUnescape",
            Self::IntersectingQ => "IntersectingQ",
            Self::Intersection => "Intersection",
            Self::ParseArgs => "ParseArgs",
            Self::StringCases => "StringCases",
            Self::StringContainsQ => "StringContainsQ",
            Self::StringCount => "StringCount",
            Self::StringDelete => "StringDelete",
            Self::StringEndsQ => "StringEndsQ",
            Self::StringFreeQ => "StringFreeQ",
            Self::StringIndent => "StringIndent",
            Self::StringMatchQ => "StringMatchQ",
            Self::StringPosition => "StringPosition",
            Self::StringReplace => "StringReplace",
            Self::StringSplit => "StringSplit",
            Self::StringStartsQ => "StringStartsQ",
            Self::StringWrap => "StringWrap",
            Self::SubsetQ => "SubsetQ",
            Self::TemplateApply => "TemplateApply",
            Self::Uncompress => "Uncompress",
            Self::Union => "Union",
            Self::UrlDecode => "UrlDecode",
            Self::UrlEncode => "UrlEncode",
            Self::UrlParse => "UrlParse",
        }
    }
}

impl OptionReentry for OptionConsumer {
    fn symbol_name(self) -> &'static str {
        self.symbol_name()
    }

    fn resume(
        self,
        session: &mut Session,
        head: &Expr,
        args: &[Expr],
        options: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        match self {
            Self::AlignColumns => {
                crate::strings::eval_align_columns_with_options(session, head, args, Some(options))
            }
            Self::Complement => crate::structure_ops::eval_complement_with_options(
                session,
                head,
                args,
                Some(options),
            ),
            Self::ContainsAll => crate::predicates::eval_contains_all_with_options(
                session,
                head,
                args,
                Some(options),
            ),
            Self::ContainsAny => crate::predicates::eval_contains_any_with_options(
                session,
                head,
                args,
                Some(options),
            ),
            Self::ContainsExactly => crate::predicates::eval_contains_exactly_with_options(
                session,
                head,
                args,
                Some(options),
            ),
            Self::ContainsNone => crate::predicates::eval_contains_none_with_options(
                session,
                head,
                args,
                Some(options),
            ),
            Self::ContainsOnly => crate::predicates::eval_contains_only_with_options(
                session,
                head,
                args,
                Some(options),
            ),
            Self::DisjointQ => {
                crate::predicates::eval_disjoint_q_with_options(session, head, args, Some(options))
            }
            Self::DuplicateFreeQ => crate::predicates::eval_duplicate_free_q_with_options(
                session,
                head,
                args,
                Some(options),
            ),
            Self::HtmlUnescape => {
                crate::strings::eval_html_unescape_with_options(session, head, args, Some(options))
            }
            Self::IntersectingQ => crate::predicates::eval_intersecting_q_with_options(
                session,
                head,
                args,
                Some(options),
            ),
            Self::Intersection => crate::structure_ops::eval_intersection_with_options(
                session,
                head,
                args,
                Some(options),
            ),
            Self::ParseArgs => {
                crate::strings::eval_parse_args_with_options(session, head, args, Some(options))
            }
            Self::StringCases => {
                crate::strings::eval_string_cases_with_options(session, head, args, Some(options))
            }
            Self::StringContainsQ => crate::strings::eval_string_contains_q_with_options(
                session,
                head,
                args,
                Some(options),
            ),
            Self::StringCount => {
                crate::strings::eval_string_count_with_options(session, head, args, Some(options))
            }
            Self::StringDelete => {
                crate::strings::eval_string_delete_with_options(session, head, args, Some(options))
            }
            Self::StringEndsQ => {
                crate::strings::eval_string_ends_q_with_options(session, head, args, Some(options))
            }
            Self::StringFreeQ => {
                crate::strings::eval_string_free_q_with_options(session, head, args, Some(options))
            }
            Self::StringIndent => {
                crate::strings::eval_string_indent_with_options(session, head, args, Some(options))
            }
            Self::StringMatchQ => {
                crate::strings::eval_string_match_q_with_options(session, head, args, Some(options))
            }
            Self::StringPosition => crate::strings::eval_string_position_with_options(
                session,
                head,
                args,
                Some(options),
            ),
            Self::StringReplace => {
                crate::strings::eval_string_replace_with_options(session, head, args, Some(options))
            }
            Self::StringSplit => {
                crate::strings::eval_string_split_with_options(session, head, args, Some(options))
            }
            Self::StringStartsQ => crate::strings::eval_string_starts_q_with_options(
                session,
                head,
                args,
                Some(options),
            ),
            Self::StringWrap => {
                crate::strings::eval_string_wrap_with_options(session, head, args, Some(options))
            }
            Self::SubsetQ => {
                crate::predicates::eval_subset_q_with_options(session, head, args, Some(options))
            }
            Self::TemplateApply => {
                crate::strings::eval_template_apply_with_options(session, head, args, Some(options))
            }
            Self::Uncompress => {
                crate::strings::eval_uncompress_with_options(session, head, args, Some(options))
            }
            Self::Union => {
                crate::structure_ops::eval_union_with_options(session, head, args, Some(options))
            }
            Self::UrlDecode => {
                crate::strings::eval_url_decode_with_options(session, head, args, Some(options))
            }
            Self::UrlEncode => {
                crate::strings::eval_url_encode_with_options(session, head, args, Some(options))
            }
            Self::UrlParse => {
                crate::strings::eval_url_parse_with_options(session, head, args, Some(options))
            }
        }
    }
}
