use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
mod options;
mod resume_frames;
use resume_frames::*;

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, rum_derive::FrameRootTrace)]
enum PropagateAbortsMode {
    Automatic,
    False,
    True,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CheckAbortPropagation {
    mode: PropagateAbortsMode,
    invalid_value: Option<Expr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CheckAbortPropagationSource {
    value: Expr,
    evaluate_rhs: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CheckAbortOptionError {
    NonOption(Expr),
    InvalidOptionName(Expr),
    UnknownOption(Symbol),
}

impl Session {
    pub(in crate::runtime) fn eval_abort(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("Abort", args, 0));
        }

        if self.abort_protect_depth > 0 {
            self.abort_protect_pending += 1;
            return Ok(system_expr("$Aborted"));
        }

        Err(EvalError::Abort)
    }

    pub(in crate::runtime) fn eval_exit(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let code = match args {
            [] => 0,
            [code] => {
                let Some(code) = expr_as_signed_isize(code) else {
                    let call = self.invalid_call("Exit", args);
                    return Ok(self.invalid_call_with_message(
                        "Exit",
                        args,
                        "intm",
                        format!("Machine-sized integer expected at position 1 in {call}."),
                    ));
                };
                code as i64
            }
            _ => {
                return Ok(self.invalid_call_with_message(
                    "Exit",
                    args,
                    "argt",
                    format!(
                        "Exit called with {} arguments; 0 or 1 arguments are expected.",
                        args.len()
                    ),
                ));
            }
        };

        Err(EvalError::Exit { code })
    }

    pub(in crate::runtime) fn eval_check_abort(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() < 2 {
            let tag = if args_slice.is_empty() {
                "argrx"
            } else {
                "argr"
            };
            return Ok(self.invalid_call_with_message(
                "CheckAbort",
                args_slice,
                tag,
                format!("expected at least 2 argument(s), got {}", args_slice.len()),
            ));
        }

        self.option_rule_expr_for_symbol(
            &Symbol::system("CheckAbort"),
            &Symbol::system("PropagateAborts"),
            options::CheckAbortOptionWorking { args },
        )
    }

    fn resume_check_abort_with_propagation(
        &mut self,
        args: &SharedArgs,
        propagation: CheckAbortPropagation,
    ) -> Result<Expr, EvalError> {
        if let Some(invalid_value) = propagation.invalid_value.as_ref() {
            self.push_message(
                "CheckAbort",
                "opttfa",
                format!(
                    "The value of the option PropagateAborts -> {invalid_value} should be True, False or Automatic."
                ),
            );
        }
        let propagate = propagation.mode;
        let pending_checkpoint = self.abort_protect_pending;

        let primary_result = self.evaluate(args[0].clone());
        let outcome = crate::resume_or_suspend!(
            self,
            primary_result,
            ok(value) => Ok(value),
            suspend => {
        let args = args.clone();
        ResumeCheckAbortWithPropagationSuspensionFrame { args, propagate, pending_checkpoint }
        },
        );

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => {
                self.resume_check_abort_after_primary(args, propagate, pending_checkpoint, outcome)
            }
        }
    }

    pub(in crate::runtime) fn eval_abort_protect(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [body] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("AbortProtect", args, 1));
        };

        let pending_checkpoint = self.abort_protect_pending;
        self.abort_protect_depth += 1;
        let body_result = self.evaluate(body.clone());
        let outcome = crate::resume_or_suspend!(
            self,
            body_result,
            ok(value) => Ok(value),
            suspend => EvalAbortProtectSuspensionFrame { pending_checkpoint },
        );

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_abort_protect_after_body(pending_checkpoint, outcome),
        }
    }

    fn resume_check_abort_after_primary(
        &mut self,
        args: &SharedArgs,
        propagate: PropagateAbortsMode,
        pending_checkpoint: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(_value) if self.abort_protect_pending > pending_checkpoint => {
                let fallback_result = self.evaluate(args[1].clone());
                let outcome = crate::resume_or_suspend!(
                    self,
                    fallback_result,
                    ok(value) => Ok(value),
                    suspend => ResumeCheckAbortAfterPrimarySuspensionFrame1 { propagate, pending_checkpoint },
                );

                match outcome {
                    Err(EvalError::InputRequired { request }) => {
                        Err(EvalError::InputRequired { request })
                    }
                    outcome => self.resume_check_abort_after_fallback(
                        propagate,
                        true,
                        pending_checkpoint,
                        outcome,
                    ),
                }
            }
            Ok(value) => Ok(value),
            Err(error @ (EvalError::Abort | EvalError::ProtectedAbort)) => {
                let protected = matches!(error, EvalError::ProtectedAbort);
                let fallback_result = self.evaluate(args[1].clone());
                let outcome = crate::resume_or_suspend!(
                    self,
                    fallback_result,
                    ok(value) => Ok(value),
                    suspend => ResumeCheckAbortAfterPrimarySuspensionFrame2 { propagate, protected, pending_checkpoint },
                );

                match outcome {
                    Err(EvalError::InputRequired { request }) => {
                        Err(EvalError::InputRequired { request })
                    }
                    outcome => self.resume_check_abort_after_fallback(
                        propagate,
                        protected,
                        pending_checkpoint,
                        outcome,
                    ),
                }
            }
            Err(error) => Err(error),
        }
    }

    fn resume_check_abort_after_fallback(
        &mut self,
        propagate: PropagateAbortsMode,
        protected: bool,
        pending_checkpoint: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                if protected && propagate != PropagateAbortsMode::True {
                    self.abort_protect_pending = pending_checkpoint;
                }

                if propagate == PropagateAbortsMode::True {
                    if self.abort_protect_depth > 0 {
                        if self.abort_protect_pending <= pending_checkpoint {
                            self.abort_protect_pending = pending_checkpoint.saturating_add(1);
                        }
                        return Ok(system_expr("$Aborted"));
                    }

                    Err(if protected {
                        EvalError::ProtectedAbort
                    } else {
                        EvalError::Abort
                    })
                } else {
                    Ok(value)
                }
            }
            Err(error) => {
                if protected {
                    self.abort_protect_pending = pending_checkpoint;
                }

                Err(error)
            }
        }
    }

    fn resume_abort_protect_after_body(
        &mut self,
        pending_checkpoint: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        if self.abort_protect_depth > 0 {
            self.abort_protect_depth -= 1;
        }

        let delayed_abort = self.abort_protect_pending > pending_checkpoint
            || matches!(outcome, Err(EvalError::Abort | EvalError::ProtectedAbort));
        self.abort_protect_pending = pending_checkpoint;

        if delayed_abort && self.abort_protect_depth > 0 {
            self.abort_protect_pending = pending_checkpoint.saturating_add(1);
            return match outcome {
                Ok(value) => Ok(value),
                Err(EvalError::Abort | EvalError::ProtectedAbort) => Ok(system_expr("$Aborted")),
                Err(error) => Err(error),
            };
        }

        match outcome {
            Ok(value) => {
                if delayed_abort {
                    Err(EvalError::Abort)
                } else {
                    Ok(value)
                }
            }
            Err(EvalError::ProtectedAbort) => Err(EvalError::Abort),
            Err(EvalError::Abort) => Err(EvalError::Abort),
            Err(error) => Err(error),
        }
    }

    pub(in crate::runtime) fn abandon_abort_protect(&mut self, pending_checkpoint: usize) {
        if self.abort_protect_depth > 0 {
            self.abort_protect_depth -= 1;
        }
        self.abort_protect_pending = pending_checkpoint;
    }

    fn invalid_check_abort_option_call(
        &mut self,
        args: &[Expr],
        error: CheckAbortOptionError,
    ) -> Expr {
        let held_call = Expr::call(system_expr("CheckAbort"), args.to_vec());
        match error {
            CheckAbortOptionError::NonOption(offender) => self.invalid_call_with_message(
                "CheckAbort",
                args,
                "nonopt",
                format!(
                    "Options expected (instead of {offender}) beyond position 2 in {held_call}. An option must be a rule or a list of rules."
                ),
            ),
            CheckAbortOptionError::InvalidOptionName(offender) => self.invalid_call_with_message(
                "CheckAbort",
                args,
                "optrs",
                format!(
                    "Option specification {offender} in {held_call} is not a rule for a symbol or string."
                ),
            ),
            CheckAbortOptionError::UnknownOption(option_name) => self.invalid_call_with_message(
                "CheckAbort",
                args,
                "optx",
                format!("Unknown option {option_name} in {held_call}."),
            ),
        }
    }
}

fn parse_check_abort_propagation_source(
    option_exprs: &[Expr],
    evaluate_rule_rhs: bool,
) -> Result<Option<CheckAbortPropagationSource>, CheckAbortOptionError> {
    let mut source = None;
    for option_expr in option_exprs {
        parse_check_abort_propagation_source_expr(option_expr, evaluate_rule_rhs, &mut source)?;
    }

    Ok(source)
}

fn default_check_abort_propagation() -> CheckAbortPropagation {
    CheckAbortPropagation {
        mode: PropagateAbortsMode::Automatic,
        invalid_value: None,
    }
}

fn parse_check_abort_propagation_source_expr(
    option_expr: &Expr,
    evaluate_rule_rhs: bool,
    source: &mut Option<CheckAbortPropagationSource>,
) -> Result<(), CheckAbortOptionError> {
    let Expr::Call { head, args } = option_expr else {
        return Err(CheckAbortOptionError::NonOption(option_expr.clone()));
    };

    if is_system_symbol_name(head, heads::structure::LIST) {
        for item in args {
            if let Err(error) =
                parse_check_abort_propagation_source_expr(item, evaluate_rule_rhs, source)
            {
                return Err(match error {
                    CheckAbortOptionError::NonOption(_) => {
                        CheckAbortOptionError::NonOption(option_expr.clone())
                    }
                    other => other,
                });
            }
        }
        return Ok(());
    }

    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return Err(CheckAbortOptionError::NonOption(option_expr.clone()));
    }

    let option_name = check_abort_option_name(&args[0])
        .ok_or_else(|| CheckAbortOptionError::InvalidOptionName(option_expr.clone()))?;
    if option_name.name() != "PropagateAborts" {
        return Err(CheckAbortOptionError::UnknownOption(option_name));
    }

    if source.is_none() {
        *source = Some(CheckAbortPropagationSource {
            value: args[1].clone(),
            evaluate_rhs: evaluate_rule_rhs && is_system_symbol_name(head, "Rule"),
        });
    }

    Ok(())
}

fn check_abort_propagation_from_value(value: Expr) -> CheckAbortPropagation {
    let (mode, invalid_value) = if is_system_symbol_name(&value, "True") {
        (PropagateAbortsMode::True, None)
    } else if is_system_symbol_name(&value, "False") {
        (PropagateAbortsMode::False, None)
    } else if is_system_symbol_name(&value, "Automatic") {
        (PropagateAbortsMode::Automatic, None)
    } else {
        (PropagateAbortsMode::Automatic, Some(value))
    };

    CheckAbortPropagation {
        mode,
        invalid_value,
    }
}

fn check_abort_option_name(expr: &Expr) -> Option<Symbol> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.clone()),
        Expr::Atom(Atom::String(name)) => resolve_unqualified_symbol(name),
        _ => None,
    }
}
