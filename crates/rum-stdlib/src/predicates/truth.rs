use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
enum TruthAggregateKind {
    All,
    Any,
    None,
}

pub(crate) fn eval_true_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("TrueQ", args, 1));
    };

    Ok(predicate_bool_expr(is_system_symbol_name(
        expr,
        heads::structure::TRUE,
    )))
}

pub(crate) fn eval_boolean_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("BooleanQ", args, 1));
    };

    Ok(predicate_bool_expr(truthiness_symbol(expr).is_some()))
}

pub(crate) fn eval_boole(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Boole", args, 1));
    };

    Ok(match truthiness_symbol(expr) {
        Some(true) => Expr::integer(BigInt::from(1_i64)),
        Some(false) => Expr::integer(BigInt::from(0_i64)),
        None => Expr::call(head.clone(), args.to_vec()),
    })
}

pub(crate) fn eval_all_true(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    eval_truth_aggregate(session, head, args, TruthAggregateKind::All)
}

pub(crate) fn eval_any_true(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    eval_truth_aggregate(session, head, args, TruthAggregateKind::Any)
}

pub(crate) fn eval_none_true(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    eval_truth_aggregate(session, head, args, TruthAggregateKind::None)
}

fn eval_truth_aggregate(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
    kind: TruthAggregateKind,
) -> Result<Expr, EvalError> {
    let args_slice = args.as_ref();
    if args_slice.len() == 1 {
        return Ok(Expr::call_with_shared_args(head.clone(), args.clone()));
    }

    if !(2..=3).contains(&args_slice.len()) {
        let symbol = truth_aggregate_name(kind);
        return Ok(session.invalid_call_with_arg_range(symbol, args_slice, 2, 3));
    }

    let traversal = if let Some(levelspec) = args_slice.get(2) {
        let Some(levels) = parse_standard_level_spec(levelspec) else {
            return Ok(session.invalid_call_with_arg_form(
                truth_aggregate_name(kind),
                args_slice,
                "expected a standard levelspec as the third argument",
            ));
        };
        CasesTraversalConfig::default_cases().with_levels(levels)
    } else {
        CasesTraversalConfig::default_cases()
    };

    let parts = collect_level_parts_owned(&args_slice[0], traversal, 0);

    resume_truth_aggregate_from_index(
        session,
        kind,
        head.clone(),
        args.clone(),
        parts,
        args_slice[1].clone(),
        0,
        false,
    )
}

fn resume_truth_aggregate_from_index(
    session: &mut Session,
    kind: TruthAggregateKind,
    head: Expr,
    call_args: SharedArgs,
    parts: Vec<Expr>,
    criterion: Expr,
    mut index: usize,
    mut saw_unknown: bool,
) -> Result<Expr, EvalError> {
    while let Some(part) = parts.get(index).cloned() {
        let call = Expr::call(criterion.clone(), vec![part]);
        let value = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let parts = FrozenPredicateSequence::freeze(parts);
                ResumeTruthAggregateFromIndexSuspensionFrame { kind, head, call_args, parts, criterion, index, saw_unknown }
                },
                );
            }
            Err(error) => return Err(error),
        };

        match truthiness_symbol(&value) {
            Some(true) => match kind {
                TruthAggregateKind::All => {}
                TruthAggregateKind::Any => return Ok(system_expr(heads::structure::TRUE)),
                TruthAggregateKind::None => return Ok(system_expr(heads::structure::FALSE)),
            },
            Some(false) => match kind {
                TruthAggregateKind::All => return Ok(system_expr(heads::structure::FALSE)),
                TruthAggregateKind::Any | TruthAggregateKind::None => {}
            },
            None => saw_unknown = true,
        }
        index += 1;
    }

    Ok(finalize_truth_aggregate(kind, head, call_args, saw_unknown))
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_truth_aggregate_from_index(
    session: &mut Session,
    kind: TruthAggregateKind,
    head: Expr,
    call_args: SharedArgs,
    parts: FrozenPredicateSequence<Expr>,
    criterion: Expr,
    mut index: usize,
    mut saw_unknown: bool,
) -> Result<Expr, EvalError> {
    while let Some(part) = parts.get(index).cloned() {
        let value = match session.evaluate(Expr::call(criterion.clone(), vec![part])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedTruthAggregateFromIndexSuspensionFrame { kind, head, call_args, parts, criterion, index, saw_unknown },
                );
            }
            Err(error) => return Err(error),
        };
        match truthiness_symbol(&value) {
            Some(true) => match kind {
                TruthAggregateKind::All => {}
                TruthAggregateKind::Any => return Ok(system_expr(heads::structure::TRUE)),
                TruthAggregateKind::None => return Ok(system_expr(heads::structure::FALSE)),
            },
            Some(false) => match kind {
                TruthAggregateKind::All => return Ok(system_expr(heads::structure::FALSE)),
                TruthAggregateKind::Any | TruthAggregateKind::None => {}
            },
            None => saw_unknown = true,
        }
        index += 1;
    }
    Ok(finalize_truth_aggregate(kind, head, call_args, saw_unknown))
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_truth_aggregate_with_value(
    session: &mut Session,
    kind: TruthAggregateKind,
    head: Expr,
    call_args: SharedArgs,
    parts: FrozenPredicateSequence<Expr>,
    criterion: Expr,
    index: usize,
    saw_unknown: bool,
    value: Expr,
) -> Result<Expr, EvalError> {
    match truthiness_symbol(&value) {
        Some(true) => match kind {
            TruthAggregateKind::All => resume_suspended_truth_aggregate_from_index(
                session,
                kind,
                head,
                call_args,
                parts,
                criterion,
                index + 1,
                saw_unknown,
            ),
            TruthAggregateKind::Any => Ok(system_expr(heads::structure::TRUE)),
            TruthAggregateKind::None => Ok(system_expr(heads::structure::FALSE)),
        },
        Some(false) => match kind {
            TruthAggregateKind::All => Ok(system_expr(heads::structure::FALSE)),
            TruthAggregateKind::Any | TruthAggregateKind::None => {
                resume_suspended_truth_aggregate_from_index(
                    session,
                    kind,
                    head,
                    call_args,
                    parts,
                    criterion,
                    index + 1,
                    saw_unknown,
                )
            }
        },
        None => resume_suspended_truth_aggregate_from_index(
            session,
            kind,
            head,
            call_args,
            parts,
            criterion,
            index + 1,
            true,
        ),
    }
}

fn truth_aggregate_name(kind: TruthAggregateKind) -> &'static str {
    match kind {
        TruthAggregateKind::All => "AllTrue",
        TruthAggregateKind::Any => "AnyTrue",
        TruthAggregateKind::None => "NoneTrue",
    }
}

fn finalize_truth_aggregate(
    kind: TruthAggregateKind,
    head: Expr,
    call_args: SharedArgs,
    saw_unknown: bool,
) -> Expr {
    if saw_unknown {
        Expr::call_with_shared_args(head, call_args)
    } else {
        match kind {
            TruthAggregateKind::All | TruthAggregateKind::None => {
                system_expr(heads::structure::TRUE)
            }
            TruthAggregateKind::Any => system_expr(heads::structure::FALSE),
        }
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeTruthAggregateFromIndexSuspensionFrame {
    kind: crate::predicates::truth::TruthAggregateKind,
    head: rum_core::Expr,
    call_args: rum_core::SharedArgs,
    parts: crate::predicates::suspended_state::FrozenPredicateSequence<rum_core::Expr>,
    criterion: rum_core::Expr,
    index: usize,
    saw_unknown: bool,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeTruthAggregateFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut kind = self.kind.clone();
        #[allow(unused_mut)]
        let mut head = self.head.clone();
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        #[allow(unused_mut)]
        let mut parts = self.parts.clone();
        #[allow(unused_mut)]
        let mut criterion = self.criterion.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut saw_unknown = self.saw_unknown.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_truth_aggregate_with_value(
                    session,
                    kind,
                    head,
                    call_args,
                    parts,
                    criterion,
                    index,
                    saw_unknown,
                    value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeSuspendedTruthAggregateFromIndexSuspensionFrame {
    kind: crate::predicates::truth::TruthAggregateKind,
    head: rum_core::Expr,
    call_args: rum_core::SharedArgs,
    parts: crate::predicates::suspended_state::FrozenPredicateSequence<rum_core::Expr>,
    criterion: rum_core::Expr,
    index: usize,
    saw_unknown: bool,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedTruthAggregateFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut kind = self.kind.clone();
        #[allow(unused_mut)]
        let mut head = self.head.clone();
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        #[allow(unused_mut)]
        let mut parts = self.parts.clone();
        #[allow(unused_mut)]
        let mut criterion = self.criterion.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut saw_unknown = self.saw_unknown.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_truth_aggregate_with_value(
                    session,
                    kind,
                    head,
                    call_args,
                    parts,
                    criterion,
                    index,
                    saw_unknown,
                    value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
