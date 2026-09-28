use super::*;
use crate::runtime::options::{MergedOptionsSuspensionFrame, OptionReentry};

pub(in crate::runtime) fn drive<M: Mode, R: OptionReentry>(
    session: &mut Session,
    mut state: OptionState<M>,
    symbol: &Symbol,
    explicit: &[Expr],
    mut index: usize,
    reentry: R,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    while index < state.defaults + explicit.len() {
        let expr = if index < state.defaults {
            &state.values[index]
        } else {
            &explicit[index - state.defaults]
        };
        let outcome = match expr {
            Expr::Call { head, args }
                if args.len() == 2 && is_system_symbol_name(head, "RuleDelayed") =>
            {
                state.guest(session, |session| {
                    if M::ARMED {
                        session.evaluate_runtime_option_expr(expr)
                    } else {
                        // The driver reselects immediately after a guest leaves
                        // a limit active. Carry its existing unarmed proof into
                        // C02; adoption still checks for a newly armed result.
                        super::super::rule::evaluate_unarmed(
                            session,
                            &args[0],
                            &args[1],
                            state.values.get(state.defaults),
                        )
                    }
                })
            }
            _ => {
                // These C02 branches cannot run guest code or arm a limit.
                if M::ARMED {
                    session.evaluate_runtime_option_expr(expr)
                } else {
                    Ok(expr.clone())
                }
            }
        };
        if !M::ARMED && session.current_memory_limit.is_some() {
            let mut state = state.arm(session)?;
            match outcome {
                Ok(value) => state.adopt(session, value)?,
                Err(error) => {
                    return Err(freeze(
                        state, session, error, symbol, explicit, index, reentry, head, args,
                    ));
                }
            }
            return drive(
                session,
                state,
                symbol,
                explicit,
                index + 1,
                reentry,
                head,
                args,
            );
        }
        match outcome {
            Ok(value) => state.adopt(session, value)?,
            Err(error) => {
                return Err(freeze(
                    state, session, error, symbol, explicit, index, reentry, head, args,
                ));
            }
        }
        index += 1;
    }
    handoff(session, state, reentry, head, args)
}

fn freeze<M: Mode, R: OptionReentry>(
    state: OptionState<M>,
    session: &mut Session,
    error: EvalError,
    symbol: &Symbol,
    explicit: &[Expr],
    index: usize,
    reentry: R,
    head: &Expr,
    args: &[Expr],
) -> EvalError {
    let defaults = state.defaults;
    let domain_generation = state.generation;
    state.freeze(session, error, || MergedOptionsSuspensionFrame {
        domain_generation,
        reentry,
        head: head.clone(),
        args: args.to_vec().into(),
        target: symbol.clone(),
        explicit: explicit.to_vec().into(),
        next_index: index + 1,
        defaults,
    })
}

fn handoff<M: Mode, R: OptionReentry>(
    session: &mut Session,
    state: OptionState<M>,
    reentry: R,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if !M::option_handoff::<R>(session, state.generation, state.domain) {
        return Ok(system_expr("$Failed"));
    }
    state.guest(session, |session| {
        reentry.resume(
            session,
            head,
            args,
            RetainedOptions {
                values: &state.values[state.defaults..],
            },
        )
    })
}

pub(in crate::runtime) fn resume<M: Mode, R: OptionReentry>(
    session: &mut Session,
    frame: &MergedOptionsSuspensionFrame<R>,
    frozen: &FrozenValues,
    outcome: Result<Expr, EvalError>,
) -> Result<Expr, EvalError> {
    if !session.check_option_resume::<R>(frame.domain_generation) {
        return Ok(system_expr("$Failed"));
    }
    let value = outcome?;
    // Charge the callback output before importing a separate working copy.
    let mut incoming = M::receipt(session);
    M::resize(&mut incoming, || Some(expr_bytes(&value)));
    let mut cx = RetentionContext::<M> {
        session,
        brand: PhantomData,
        mode: PhantomData,
    };
    let empty = RetainedValues {
        values: Vec::new(),
        receipt: M::receipt(cx.session),
        brand: PhantomData,
    };
    let working = cx.resume_values(empty, frozen)?;
    let mut state = OptionState::<M> {
        values: M::Options::working(working.values, frame.defaults * 2 + frame.explicit.len()),
        receipt: working.receipt,
        defaults: frame.defaults,
        generation: frame.domain_generation,
        domain: M::option_domain(cx.session),
    }
    .charge(cx.session)?;
    state.adopt(cx.session, value)?;
    drop(incoming);
    drive(
        cx.session,
        state,
        &frame.target,
        &frame.explicit,
        frame.next_index,
        frame.reentry,
        &frame.head,
        &frame.args,
    )
}
