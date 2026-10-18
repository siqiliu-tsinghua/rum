use super::super::*;

mod abort_cleanup;
mod assertions;
mod branching_loops;
mod check_quiet;
mod cleanup;
mod confirmation;
mod exceptions;
mod failsafe;
mod nonlocal;
mod status;

pub(in crate::runtime) use self::failsafe::{
    apply_callable_failsafe_wrapper_to_args, is_callable_failsafe_wrapper,
};
pub(in crate::runtime) use self::status::{
    StatusObjectProjection, classify_status_object_call, is_failure_expr, is_success_expr,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlDataBuiltin {
    Abort,
    AbortProtect,
    Assert,
    Break,
    Catch,
    CatchExceptions,
    CheckAbort,
    CompoundExpression,
    Confirm,
    ConfirmAssert,
    ConfirmBy,
    ConfirmMatch,
    ConfirmQuiet,
    Continue,
    Do,
    Enclose,
    ExceptionTypes,
    Exit,
    Failsafe,
    Failure,
    FailureQ,
    For,
    If,
    Product,
    RegisterExceptionType,
    Return,
    Success,
    Sum,
    Switch,
    Table,
    TerminatedEvaluation,
    Throw,
    ThrowException,
    Until,
    Which,
    While,
    WithCleanup,
}

impl Session {
    /// Dispatches control-family builtin handlers whose bodies remain kernel-side.
    pub fn eval_control_data_builtin(
        &mut self,
        builtin: ControlDataBuiltin,
        head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        match builtin {
            ControlDataBuiltin::Abort => self.eval_abort(head, args),
            ControlDataBuiltin::AbortProtect => self.eval_abort_protect(args),
            ControlDataBuiltin::Assert => self.eval_assert(args),
            ControlDataBuiltin::Break => self.eval_break(head, args),
            ControlDataBuiltin::Catch => self.eval_catch(args),
            ControlDataBuiltin::CatchExceptions => self.eval_catch_exceptions(args),
            ControlDataBuiltin::CheckAbort => self.eval_check_abort(args),
            ControlDataBuiltin::CompoundExpression => self.eval_compound_expression(args),
            ControlDataBuiltin::Confirm => self.eval_confirm(args),
            ControlDataBuiltin::ConfirmAssert => self.eval_confirm_assert(args),
            ControlDataBuiltin::ConfirmBy => self.eval_confirm_by(args),
            ControlDataBuiltin::ConfirmMatch => self.eval_confirm_match(args),
            ControlDataBuiltin::ConfirmQuiet => self.eval_confirm_quiet(args),
            ControlDataBuiltin::Continue => self.eval_continue(head, args),
            ControlDataBuiltin::Do => {
                self.with_iteration_product_scope(|session| session.eval_do(args))
            }
            ControlDataBuiltin::Enclose => self.eval_enclose(args),
            ControlDataBuiltin::ExceptionTypes => self.eval_exception_types(head, args),
            ControlDataBuiltin::Exit => self.eval_exit(args),
            ControlDataBuiltin::Failsafe => self.eval_failsafe(head, args),
            ControlDataBuiltin::Failure => self.eval_failure(head, args),
            ControlDataBuiltin::FailureQ => self.eval_failure_q(head, args),
            ControlDataBuiltin::For => self.eval_for(args),
            ControlDataBuiltin::If => self.eval_if(args),
            ControlDataBuiltin::Product => {
                self.with_iteration_product_scope(|session| session.eval_product(args))
            }
            ControlDataBuiltin::RegisterExceptionType => {
                self.eval_register_exception_type(head, args)
            }
            ControlDataBuiltin::Return => self.eval_return(head, args),
            ControlDataBuiltin::Success => self.eval_success(head, args),
            ControlDataBuiltin::Sum => {
                self.with_iteration_product_scope(|session| session.eval_sum(args))
            }
            ControlDataBuiltin::Switch => self.eval_switch(args),
            ControlDataBuiltin::Table => {
                self.with_iteration_product_scope(|session| session.eval_table(args))
            }
            ControlDataBuiltin::TerminatedEvaluation => self.eval_terminated_evaluation(head, args),
            ControlDataBuiltin::Throw => self.eval_throw(head, args),
            ControlDataBuiltin::ThrowException => self.eval_throw_exception(head, args),
            ControlDataBuiltin::Until => self.eval_until(args),
            ControlDataBuiltin::Which => self.eval_which(args),
            ControlDataBuiltin::While => self.eval_while(args),
            ControlDataBuiltin::WithCleanup => self.eval_with_cleanup(args),
        }
    }
}
