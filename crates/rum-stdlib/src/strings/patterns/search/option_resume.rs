use super::*;
use rum_runtime::kernel_api::{
    ContinuationValue, FrameAction, FrameRootTrace, FrameRootVisitor, SuspensionFrame,
};
use std::sync::Arc;

/// Resume the consumer with the resolved options, without evaluating arguments
/// or options again. Its initial pattern parsing is pure and uses frozen args.
#[derive(FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct StringOptionsSuspensionFrame {
    pub(super) symbol: &'static str,
    pub(super) head: Expr,
    pub(super) args: SharedArgs,
}

impl SuspensionFrame for StringOptionsSuspensionFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let resumed = outcome.and_then(|value| {
            let options = Some(Vec::<Expr>::from_continuation_expr(value));
            let head = &self.head;
            let args = &self.args;
            match self.symbol {
                "StringPosition" => eval_string_position_with_options(session, head, args, options),
                "StringSplit" => eval_string_split_with_options(session, head, args, options),
                "StringReplace" => eval_string_replace_with_options(session, head, args, options),
                "StringCases" => eval_string_cases_with_options(session, head, args, options),
                "StringCount" => eval_string_count_with_options(session, head, args, options),
                "StringDelete" => eval_string_delete_with_options(session, head, args, options),
                "StringContainsQ" | "StringStartsQ" | "StringEndsQ" => {
                    let predicate = match self.symbol {
                        "StringContainsQ" => string_contains_q,
                        "StringStartsQ" => string_starts_q,
                        _ => string_ends_q,
                    };
                    eval_string_pattern_predicate(
                        session,
                        head,
                        args,
                        self.symbol,
                        predicate,
                        options,
                    )
                }
                "StringMatchQ" | "StringFreeQ" => {
                    let predicate = if self.symbol == "StringMatchQ" {
                        string_match_q
                    } else {
                        string_free_q
                    };
                    eval_string_patterns_predicate(
                        session,
                        head,
                        args,
                        self.symbol,
                        predicate,
                        options,
                    )
                }
                _ => unreachable!("only string search consumers install this frame"),
            }
        });
        FrameAction::complete(resumed)
    }
}
