//! Indexed pulls and output filling, shared by ordinary and resumed realizations.
use super::PullBudget;
use super::source::pull_source;
use super::state::{AwaitCursor, PullContext, PullMachine, PullStage};
use rum_core::Expr;
use rum_runtime::kernel_api::{EvalError, Session, truthiness_symbol};

pub(super) fn drive_machine(
    machine: &mut PullMachine,
    session: &mut Session,
    ctx: &mut PullContext,
    items: &mut Vec<Expr>,
    budget: PullBudget,
    mut boundary: usize,
    mut carried: Option<Expr>,
) -> Result<(), EvalError> {
    if budget.reached(items.len()) {
        return Ok(());
    }
    'pull: loop {
        // Keep the value in one local during ascent, rather than moving a
        // value-bearing cursor for every callback-free stage transition.
        let mut value = match carried.take() {
            Some(value) => value,
            None => loop {
                if boundary == machine.stages.len() {
                    match pull_source(&mut machine.source, session, ctx)? {
                        Some(value) => break value,
                        None => return exhausted(machine, boundary),
                    }
                }
                match &mut machine.stages[boundary] {
                    PullStage::Take { remaining } => {
                        if *remaining == 0 {
                            return exhausted(machine, boundary);
                        }
                        // Consume the allowance before entering the child, exactly once.
                        *remaining -= 1;
                    }
                    PullStage::FoldList {
                        accumulator,
                        emitted_init,
                        ..
                    } if !*emitted_init => {
                        *emitted_init = true;
                        break accumulator.clone();
                    }
                    _ => {}
                }
                boundary += 1;
            },
        };
        while boundary > 0 {
            boundary -= 1;
            let stage = boundary;
            match &mut machine.stages[stage] {
                PullStage::Take { .. } => {}
                PullStage::Drop { remaining } => {
                    if *remaining > 0 {
                        *remaining -= 1;
                        drop(value);
                        boundary += 1;
                        continue 'pull;
                    }
                }
                PullStage::Map { function } => {
                    let call = Expr::call(function.clone(), vec![value]);
                    let answer = match session.evaluate(call) {
                        Ok(answer) => answer,
                        Err(error) => {
                            return Err(ctx.await_callback(error, AwaitCursor::Map { stage }));
                        }
                    };
                    ctx.observe_cancellation()?;
                    value = answer;
                }
                PullStage::Select { predicate } => {
                    let call = Expr::call(predicate.clone(), vec![value.clone()]);
                    let selected = match session.evaluate(call) {
                        Ok(answer) => answer,
                        Err(error) => {
                            return Err(ctx.await_callback(
                                error,
                                AwaitCursor::Select {
                                    stage,
                                    candidate: value,
                                },
                            ));
                        }
                    };
                    ctx.observe_cancellation()?;
                    let keep = truthiness_symbol(&selected) == Some(true);
                    drop(selected);
                    if !keep {
                        drop(value);
                        // Retry only this child; outer Take allowances stay consumed.
                        boundary += 1;
                        continue 'pull;
                    }
                }
                PullStage::FoldList {
                    function,
                    accumulator,
                    ..
                } => {
                    let call = Expr::call(function.clone(), vec![accumulator.clone(), value]);
                    let answer = match session.evaluate(call) {
                        Ok(answer) => answer,
                        Err(error) => {
                            return Err(ctx.await_callback(error, AwaitCursor::Fold { stage }));
                        }
                    };
                    ctx.observe_cancellation()?;
                    // Reborrow after guest evaluation; no stage reference crosses it.
                    if let PullStage::FoldList { accumulator, .. } = &mut machine.stages[stage] {
                        *accumulator = answer;
                        value = accumulator.clone();
                    } else {
                        unreachable!("the machine's stage variants do not change")
                    }
                }
            }
        }
        items.push(value);
        if budget.reached(items.len()) {
            return Ok(());
        }
        boundary = 0;
    }
}

pub(super) fn exhausted(machine: &mut PullMachine, boundary: usize) -> Result<(), EvalError> {
    for stage in machine.stages[..boundary].iter_mut().rev() {
        if let PullStage::Drop { remaining } = stage {
            *remaining = 0;
        }
    }
    Ok(())
}
