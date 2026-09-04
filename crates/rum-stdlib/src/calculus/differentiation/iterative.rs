use super::*;

enum DifferentiationTask {
    Visit(Expr),
    FinishPlusArg {
        args: SharedArgs,
        index: usize,
        derivatives: Vec<Expr>,
    },
    FinishTimesArg {
        args: SharedArgs,
        index: usize,
        terms: Vec<Expr>,
    },
    FinishRationalPower {
        base: Expr,
        exponent: Rational,
    },
    FinishAbstractArg {
        original: Expr,
        head: Expr,
        args: SharedArgs,
        index: usize,
        terms: Vec<Expr>,
    },
}

pub(super) fn differentiate_expr_iterative_after_prepass(
    session: &Session,
    expr: &Expr,
    variable: &Expr,
) -> Result<Expr, EvalError> {
    let mut work = Vec::new();
    let mut completed = None;
    schedule_known_differentiation(session, expr.clone(), &mut work, &mut completed)?;

    while let Some(task) = work.pop() {
        match task {
            DifferentiationTask::Visit(expr) => {
                if expr == *variable {
                    set_completed(&mut completed, one_expr());
                } else if !expr_contains_exact_subexpr(&expr, variable) {
                    set_completed(&mut completed, zero_expr());
                } else {
                    schedule_known_differentiation(session, expr, &mut work, &mut completed)?;
                }
            }
            DifferentiationTask::FinishPlusArg {
                args,
                index,
                mut derivatives,
            } => {
                let derivative = take_completed(&mut completed);
                if !expr_is_exact_zero(&derivative) {
                    derivatives.push(derivative);
                }

                let next_index = index + 1;
                if let Some(next_arg) = args.get(next_index).cloned() {
                    work.push(DifferentiationTask::FinishPlusArg {
                        args,
                        index: next_index,
                        derivatives,
                    });
                    work.push(DifferentiationTask::Visit(next_arg));
                } else {
                    set_completed(
                        &mut completed,
                        session.combine_additive_args(&system_expr("Plus"), &derivatives)?,
                    );
                }
            }
            DifferentiationTask::FinishTimesArg {
                args,
                index,
                mut terms,
            } => {
                let derivative = take_completed(&mut completed);
                if !expr_is_exact_zero(&derivative) {
                    terms.push(combine_product_rule_term(
                        session, &args, index, derivative,
                    )?);
                }

                let next_index = index + 1;
                if let Some(next_arg) = args.get(next_index).cloned() {
                    work.push(DifferentiationTask::FinishTimesArg {
                        args,
                        index: next_index,
                        terms,
                    });
                    work.push(DifferentiationTask::Visit(next_arg));
                } else {
                    set_completed(
                        &mut completed,
                        session.combine_additive_args(&system_expr("Plus"), &terms)?,
                    );
                }
            }
            DifferentiationTask::FinishRationalPower { base, exponent } => {
                let base_derivative = take_completed(&mut completed);
                set_completed(
                    &mut completed,
                    finish_rational_power_derivative(session, &base, exponent, base_derivative)?,
                );
            }
            DifferentiationTask::FinishAbstractArg {
                original,
                head,
                args,
                index,
                mut terms,
            } => {
                let derivative = take_completed(&mut completed);
                if !expr_is_exact_zero(&derivative) {
                    let Some(partial) = partial_derivative_call(&head, &args, index) else {
                        set_completed(&mut completed, unevaluated_derivative(&original, variable));
                        continue;
                    };
                    terms.push(combine_chain_rule_term(session, partial, derivative)?);
                }

                let next_index = index + 1;
                if let Some(next_arg) = args.get(next_index).cloned() {
                    work.push(DifferentiationTask::FinishAbstractArg {
                        original,
                        head,
                        args,
                        index: next_index,
                        terms,
                    });
                    work.push(DifferentiationTask::Visit(next_arg));
                } else {
                    set_completed(
                        &mut completed,
                        session.combine_additive_args(&system_expr("Plus"), &terms)?,
                    );
                }
            }
        }
    }

    Ok(take_completed(&mut completed))
}

fn schedule_known_differentiation(
    session: &Session,
    expr: Expr,
    work: &mut Vec<DifferentiationTask>,
    completed: &mut Option<Expr>,
) -> Result<(), EvalError> {
    match expr {
        Expr::Atom(_) => set_completed(completed, zero_expr()),
        Expr::Call { head, args } if is_system_symbol_name(&head, "Plus") => {
            schedule_first_plus_arg(session, args, work, completed)?;
        }
        Expr::Call { head, args } if is_system_symbol_name(&head, "Times") => {
            schedule_first_times_arg(session, args, work, completed)?;
        }
        Expr::Call { head, args } if is_system_symbol_name(&head, "Power") && args.len() == 2 => {
            if let Some(exponent) = expr_as_exact_rational(&args[1]) {
                if exponent.is_zero() {
                    set_completed(completed, zero_expr());
                } else {
                    let base = args[0].clone();
                    work.push(DifferentiationTask::FinishRationalPower {
                        base: base.clone(),
                        exponent,
                    });
                    work.push(DifferentiationTask::Visit(base));
                }
            } else {
                let original = Expr::call_shared(head, args.clone());
                schedule_first_abstract_arg(
                    session,
                    original,
                    system_expr("Power"),
                    args,
                    work,
                    completed,
                )?;
            }
        }
        Expr::Call { head, args } => {
            let original = Expr::call_shared(head.clone(), args.clone());
            schedule_first_abstract_arg(
                session,
                original,
                head.into_owned(),
                args,
                work,
                completed,
            )?;
        }
    }
    Ok(())
}

fn schedule_first_plus_arg(
    session: &Session,
    args: SharedArgs,
    work: &mut Vec<DifferentiationTask>,
    completed: &mut Option<Expr>,
) -> Result<(), EvalError> {
    if let Some(first_arg) = args.first().cloned() {
        work.push(DifferentiationTask::FinishPlusArg {
            args,
            index: 0,
            derivatives: Vec::new(),
        });
        work.push(DifferentiationTask::Visit(first_arg));
    } else {
        set_completed(
            completed,
            session.combine_additive_args(&system_expr("Plus"), &[])?,
        );
    }
    Ok(())
}

fn schedule_first_times_arg(
    session: &Session,
    args: SharedArgs,
    work: &mut Vec<DifferentiationTask>,
    completed: &mut Option<Expr>,
) -> Result<(), EvalError> {
    if let Some(first_arg) = args.first().cloned() {
        work.push(DifferentiationTask::FinishTimesArg {
            args,
            index: 0,
            terms: Vec::new(),
        });
        work.push(DifferentiationTask::Visit(first_arg));
    } else {
        set_completed(
            completed,
            session.combine_additive_args(&system_expr("Plus"), &[])?,
        );
    }
    Ok(())
}

fn schedule_first_abstract_arg(
    session: &Session,
    original: Expr,
    head: Expr,
    args: SharedArgs,
    work: &mut Vec<DifferentiationTask>,
    completed: &mut Option<Expr>,
) -> Result<(), EvalError> {
    if let Some(first_arg) = args.first().cloned() {
        work.push(DifferentiationTask::FinishAbstractArg {
            original,
            head,
            args,
            index: 0,
            terms: Vec::new(),
        });
        work.push(DifferentiationTask::Visit(first_arg));
    } else {
        set_completed(
            completed,
            session.combine_additive_args(&system_expr("Plus"), &[])?,
        );
    }
    Ok(())
}

fn set_completed(completed: &mut Option<Expr>, expr: Expr) {
    debug_assert!(completed.is_none());
    *completed = Some(expr);
}

fn take_completed(completed: &mut Option<Expr>) -> Expr {
    completed
        .take()
        .expect("each differentiation continuation consumes one completed child")
}
