use super::super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewriteDataBuiltin {
    Cases,
    DeleteCases,
    MatchQ,
    Replace,
    ReplaceAll,
    ReplaceAllUnheld,
    ReplaceList,
    ReplaceRepeated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DeleteCasesResult {
    Keep(Expr),
    Delete,
    Splice(Vec<Expr>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DeleteCasesOutcome {
    result: DeleteCasesResult,
    depth: usize,
    deleted_count: usize,
    exhausted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct ReplaceOutcome {
    expr: Expr,
    depth: usize,
    changed: bool,
}

impl ContinuationValue for DeleteCasesOutcome {
    fn into_continuation_expr(self) -> Expr {
        let result_expr = match self.result {
            DeleteCasesResult::Keep(expr) => Expr::call(
                system_expr("InternalContinuationDeleteCasesKeep"),
                vec![expr],
            ),
            DeleteCasesResult::Delete => Expr::call(
                system_expr("InternalContinuationDeleteCasesDelete"),
                Vec::new(),
            ),
            DeleteCasesResult::Splice(items) => Expr::call(
                system_expr("InternalContinuationDeleteCasesSplice"),
                vec![Expr::call(system_expr("List"), items)],
            ),
        };

        Expr::call(
            system_expr("InternalContinuationDeleteCasesOutcome"),
            vec![
                result_expr,
                Expr::integer(BigInt::from(self.depth)),
                Expr::integer(BigInt::from(self.deleted_count)),
                system_expr(if self.exhausted { "True" } else { "False" }),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args, .. }
                if is_system_symbol_name(&head, "InternalContinuationDeleteCasesOutcome")
                    && args.len() == 4 =>
            {
                let mut args = Expr::unwrap_shared_args(args);
                let exhausted_expr = args.pop().expect("delete-cases payload stores exhausted");
                let deleted_count_expr = args
                    .pop()
                    .expect("delete-cases payload stores deleted count");
                let depth_expr = args.pop().expect("delete-cases payload stores depth");
                let result_expr = args.pop().expect("delete-cases payload stores result");

                let result = match result_expr {
                    Expr::Call { head, args, .. }
                        if is_system_symbol_name(&head, "InternalContinuationDeleteCasesKeep")
                            && args.len() == 1 =>
                    {
                        let mut args = Expr::unwrap_shared_args(args);
                        DeleteCasesResult::Keep(
                            args.pop().expect("delete-cases keep payload stores expr"),
                        )
                    }
                    Expr::Call { head, args, .. }
                        if is_system_symbol_name(
                            &head,
                            "InternalContinuationDeleteCasesDelete",
                        ) && args.is_empty() =>
                    {
                        DeleteCasesResult::Delete
                    }
                    Expr::Call { head, args, .. }
                        if is_system_symbol_name(
                            &head,
                            "InternalContinuationDeleteCasesSplice",
                        ) && args.len() == 1 =>
                    {
                        let mut args = Expr::unwrap_shared_args(args);
                        let items_expr = args
                            .pop()
                            .expect("delete-cases splice payload stores items");
                        let items = match items_expr {
                            Expr::Call { head, args, .. }
                                if is_system_symbol_name(&head, "List") =>
                            {
                                args.into_vec()
                            }
                            other => vec![other],
                        };
                        DeleteCasesResult::Splice(items)
                    }
                    other => panic!("unexpected delete-cases result payload: {other:?}"),
                };

                Self {
                    result,
                    depth: expr_as_non_negative_usize(&depth_expr)
                        .expect("delete-cases depth should be non-negative"),
                    deleted_count: expr_as_non_negative_usize(&deleted_count_expr)
                        .expect("delete-cases deleted count should be non-negative"),
                    exhausted: is_system_symbol_name(&exhausted_expr, "True"),
                }
            }
            other => panic!("unexpected continuation payload for DeleteCasesOutcome: {other:?}"),
        }
    }
}

impl ContinuationValue for ReplaceOutcome {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationReplaceOutcome"),
            vec![
                self.expr,
                Expr::integer(BigInt::from(self.depth)),
                system_expr(if self.changed { "True" } else { "False" }),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args, .. }
                if is_system_symbol_name(&head, "InternalContinuationReplaceOutcome")
                    && args.len() == 3 =>
            {
                let mut args = Expr::unwrap_shared_args(args);
                let changed_expr = args.pop().expect("replace outcome payload stores changed");
                let depth_expr = args.pop().expect("replace outcome payload stores depth");
                let expr = args.pop().expect("replace outcome payload stores expr");

                Self {
                    expr,
                    depth: expr_as_non_negative_usize(&depth_expr)
                        .expect("replace outcome depth should be non-negative"),
                    changed: is_system_symbol_name(&changed_expr, "True"),
                }
            }
            other => panic!("unexpected continuation payload for ReplaceOutcome: {other:?}"),
        }
    }
}

mod cases;
mod replace;
mod suspended_state;

use suspended_state::{
    FrozenRewriteQueue, FrozenRewriteValue, PersistentRewriteExprs, PersistentRewriteQueue,
};

impl Session {
    /// Evaluates the internal two-argument MatchQ bridge without constructing a call handle.
    pub fn eval_match_q_data_builtin(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_match_q(args)
    }

    /// Dispatches rewrite-family builtin handlers whose bodies remain kernel-side.
    pub fn eval_rewrite_data_builtin(
        &mut self,
        builtin: RewriteDataBuiltin,
        _head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        match builtin {
            RewriteDataBuiltin::Cases => self.eval_cases(args),
            RewriteDataBuiltin::DeleteCases => self.eval_delete_cases(args),
            RewriteDataBuiltin::MatchQ => self.eval_match_q(args),
            RewriteDataBuiltin::Replace => self.eval_replace(args),
            RewriteDataBuiltin::ReplaceAll => self.eval_replace_all(args),
            RewriteDataBuiltin::ReplaceAllUnheld => self.eval_replace_all_unheld(args),
            RewriteDataBuiltin::ReplaceList => self.eval_replace_list(args),
            RewriteDataBuiltin::ReplaceRepeated => self.eval_replace_repeated(args),
        }
    }
}
