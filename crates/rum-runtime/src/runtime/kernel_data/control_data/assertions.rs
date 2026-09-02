use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

impl Session {
    pub(in crate::runtime) fn eval_assert(&mut self, args: &SharedArgs) -> Result<Expr, EvalError> {
        let test = args.first().cloned();

        let function_result = self.evaluate(system_expr("$AssertFunction"));
        let function = crate::resume_or_suspend!(
            self,
            function_result,
            ok(value) => Ok(value),
            suspend => {
        let test = test.clone();
                    let args = args.clone();
        EvalAssertSuspensionFrame { test, args }
        },
        )?;

        self.resume_assert_after_function(test, args, function)
    }

    fn resume_assert_after_test(
        &mut self,
        test: Expr,
        assert_args: &SharedArgs,
        assert_function: Option<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) if is_system_symbol_name(&value, "True") => Ok(system_expr("Null")),
            Ok(value) => {
                for (message_tag, text) in assert_test_value_messages(&test, &value) {
                    self.push_message("Assert", message_tag, text);
                }

                if let Some(assert_function) = assert_function {
                    return self.finish_assert_function(assert_function, assert_args);
                }

                for (message_tag, text) in assert_failure_messages(&test, assert_args.as_ref()) {
                    self.push_message("Assert", message_tag, text);
                }
                Ok(system_expr("Null"))
            }
            Err(error) => Err(error),
        }
    }

    fn resume_assert_after_function(
        &mut self,
        test: Option<Expr>,
        assert_args: &SharedArgs,
        function: Expr,
    ) -> Result<Expr, EvalError> {
        let assert_function = if is_system_symbol_name(&function, "$AssertFunction")
            || is_system_symbol_name(&function, "None")
        {
            None
        } else {
            Some(function)
        };
        if !self.asserts_enabled && assert_function.is_none() {
            return Ok(Expr::call_with_shared_args(
                system_expr("Assert"),
                assert_args.clone(),
            ));
        }
        let Some(test) = test else {
            self.push_message(
                "Assert",
                "asrte",
                format!(
                    "Assertion {} had no test.",
                    render_input_like_expr(&Expr::call_with_shared_args(
                        system_expr("Assert"),
                        assert_args.clone(),
                    ))
                ),
            );
            return Ok(system_expr("Null"));
        };

        let test_result = self.evaluate(test.clone());
        let outcome = crate::resume_or_suspend!(
            self,
            test_result,
            ok(value) => Ok(value),
            suspend => {
        let test = test.clone();
                    let assert_args = assert_args.clone();
                    let assert_function = assert_function.clone();
        ResumeAssertAfterFunctionSuspensionFrame { test, assert_args, assert_function }
        },
        );

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_assert_after_test(test, assert_args, assert_function, outcome),
        }
    }

    fn finish_assert_function(
        &mut self,
        assert_function: Expr,
        assert_args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let payload = Expr::call(
            system_expr("HoldComplete"),
            vec![Expr::call_with_shared_args(
                system_expr("Assert"),
                assert_args.clone(),
            )],
        );
        let result = self.evaluate(Expr::call(assert_function, vec![payload]));
        crate::resume_or_suspend!(
            self,
            result,
            ok(_) => { (|| { Ok(system_expr("Null")) })() },
            suspend => FinishAssertFunctionSuspensionFrame,
        )
    }
}

fn assert_test_value_messages(test: &Expr, value: &Expr) -> Vec<(&'static str, String)> {
    let mut messages = Vec::new();
    let test_text = render_input_like_expr(test);

    if !is_system_symbol_name(value, "False") {
        messages.push((
            "asrttf",
            format!(
                "Assertion test {test_text} evaluated to {} that is neither True nor False.",
                render_input_like_expr(value)
            ),
        ));
    }

    messages
}

fn assert_failure_messages(test: &Expr, assert_args: &[Expr]) -> Vec<(&'static str, String)> {
    let test_text = render_input_like_expr(test);
    vec![match assert_args {
        [_] => ("asrtf", format!("Assertion {test_text} failed.")),
        _ => (
            "asrtfe",
            format!(
                "Assertion {test_text} in {} failed.",
                render_input_like_expr(&Expr::call(system_expr("Assert"), assert_args.to_vec()))
            ),
        ),
    }]
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalAssertSuspensionFrame {
    test: std::option::Option<rum_core::Expr>,
    args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalAssertSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut test = self.test.clone();
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(function) => (|| session.resume_assert_after_function(test, &args, function))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeAssertAfterFunctionSuspensionFrame {
    test: rum_core::Expr,
    assert_args: rum_core::SharedArgs,
    assert_function: std::option::Option<rum_core::Expr>,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeAssertAfterFunctionSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut test = self.test.clone();
        #[allow(unused_mut)]
        let mut assert_args = self.assert_args.clone();
        #[allow(unused_mut)]
        let mut assert_function = self.assert_function.clone();
        let resumed =
            (|| session.resume_assert_after_test(test, &assert_args, assert_function, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct FinishAssertFunctionSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for FinishAssertFunctionSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let _ = <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> { Ok(system_expr("Null")) })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
