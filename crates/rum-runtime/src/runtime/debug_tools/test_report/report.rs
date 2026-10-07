mod resume_frames;
use resume_frames::*;

use super::model::{EvalCapturedOutcome, TestCaseSpec, TestReportOptions};
use super::parsing::{parse_test_case, parse_test_case_exprs, parse_test_report_objects};
use super::*;
use crate::runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};
use std::sync::Arc;

#[derive(Clone, FrameRootTrace)]
struct TestReportEvalState {
    head: Expr,
    raw_args: Vec<Expr>,
    tests: Vec<Expr>,
    properties: Option<Expr>,
    options: TestReportOptions,
}

#[derive(Clone, FrameRootTrace)]
struct TestReportCaseEvalState {
    report: TestReportEvalState,
    index: usize,
    successes: usize,
    failures: usize,
    rendered_results: Vec<Expr>,
    test_case: TestCaseSpec,
    expected_messages: Option<Vec<Expr>>,
}

#[derive(Clone, FrameRootTrace)]
struct TestReportResultAccumulator {
    prefix: SharedArgs,
    tail: AccumulatorChain<TestReportResultNode>,
    len: usize,
}

#[derive(Clone, FrameRootTrace, AccumulatorChainNode)]
struct TestReportResultNode {
    #[accumulator_link]
    previous: AccumulatorChain<TestReportResultNode>,
    value: Expr,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedTestReportState {
    head: Expr,
    raw_args: SharedArgs,
    tests: SharedArgs,
    properties: Option<Expr>,
    options: TestReportOptions,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedTestCaseSpec {
    input: Expr,
    expected: Expr,
    test_id: Option<Expr>,
    same_test: Option<Expr>,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedTestReportCase {
    report: SuspendedTestReportState,
    index: usize,
    successes: usize,
    failures: usize,
    rendered_results: TestReportResultAccumulator,
    test_case: SuspendedTestCaseSpec,
    expected_messages: Option<SharedArgs>,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedEvalCapturedOutcome {
    value: Expr,
    failure_type: Option<&'static str>,
    messages: Arc<[RuntimeMessage]>,
    prints: Arc<[String]>,
    timing: Expr,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedTestReportOutput {
    case: SuspendedTestReportCase,
    evaluation: SuspendedEvalCapturedOutcome,
    messages_match: bool,
}

impl TestReportResultAccumulator {
    fn freeze(values: Vec<Expr>) -> Self {
        let len = values.len();
        Self {
            prefix: SharedArgs::new(values),
            tail: AccumulatorChain::default(),
            len,
        }
    }

    fn appended(&self, value: Expr) -> Self {
        Self {
            prefix: self.prefix.clone(),
            tail: AccumulatorChain::new(Some(Arc::new(TestReportResultNode {
                previous: self.tail.clone(),
                value,
            }))),
            len: self.len + 1,
        }
    }

    fn into_vec(self) -> Vec<Expr> {
        let mut suffix = Vec::with_capacity(self.len.saturating_sub(self.prefix.len()));
        for current in self.tail.into_completion() {
            suffix.push(current.value);
        }
        suffix.reverse();

        let mut values = self.prefix.into_vec();
        values.reserve(suffix.len());
        values.extend(suffix);
        values
    }
}

impl SuspendedTestReportCase {
    fn freeze(state: TestReportCaseEvalState) -> Self {
        let TestReportCaseEvalState {
            report,
            index,
            successes,
            failures,
            rendered_results,
            test_case,
            expected_messages,
        } = state;
        Self {
            report: SuspendedTestReportState {
                head: report.head,
                raw_args: SharedArgs::new(report.raw_args),
                tests: SharedArgs::new(report.tests),
                properties: report.properties,
                options: report.options,
            },
            index,
            successes,
            failures,
            rendered_results: TestReportResultAccumulator::freeze(rendered_results),
            test_case: SuspendedTestCaseSpec {
                input: test_case.input,
                expected: test_case.expected,
                test_id: test_case.test_id,
                same_test: test_case.same_test,
            },
            expected_messages: expected_messages.map(SharedArgs::new),
        }
    }
}

impl SuspendedEvalCapturedOutcome {
    fn freeze(outcome: EvalCapturedOutcome) -> Self {
        Self {
            value: outcome.value,
            failure_type: outcome.failure_type,
            messages: Arc::from(outcome.messages),
            prints: Arc::from(outcome.prints),
            timing: outcome.timing,
        }
    }

    fn into_owned(self) -> EvalCapturedOutcome {
        EvalCapturedOutcome {
            value: self.value,
            failure_type: self.failure_type,
            messages: self.messages.iter().cloned().collect(),
            prints: self.prints.iter().cloned().collect(),
            timing: self.timing,
        }
    }
}

impl Session {
    pub(in crate::runtime) fn eval_test_report(
        &mut self,
        head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let (properties, explicit) = match args.as_ref() {
            [_] => (None, None),
            [_, second] => {
                if let Some(options) = self.parse_test_report_options(second) {
                    (None, Some(options))
                } else {
                    (Some(second.clone()), None)
                }
            }
            [_, properties, options_expr] => {
                let Some(options) = self.parse_test_report_options(options_expr) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                (Some(properties.clone()), Some(options))
            }
            _ => return Ok(Expr::call(head.clone(), args.to_vec())),
        };
        if let Some(explicit) = explicit {
            return super::options::read_test_report_options(
                self, head, args, properties, &explicit,
            );
        }
        self.start_test_report(head, args, properties, TestReportOptions::default())
    }

    #[inline(always)]
    pub(super) fn start_test_report(
        &mut self,
        head: &Expr,
        args: &SharedArgs,
        properties: Option<Expr>,
        options: TestReportOptions,
    ) -> Result<Expr, EvalError> {
        let tests_expr = &args[0];
        if let Some(report) = self.merge_test_report_objects(tests_expr) {
            return self.project_association_properties(
                "TestReport",
                &[Expr::string("<omitted>")],
                report,
                properties,
            );
        }

        let tests = match parse_test_case_exprs(tests_expr) {
            Some(tests) => tests,
            None => return Ok(Expr::call(head.clone(), args.to_vec())),
        };

        self.resume_test_report_from_index(
            TestReportEvalState {
                head: head.clone(),
                raw_args: args.to_vec(),
                tests,
                properties,
                options,
            },
            0,
            0,
            0,
            Vec::new(),
        )
    }

    fn resume_test_report_from_index(
        &mut self,
        state: TestReportEvalState,
        index: usize,
        successes: usize,
        failures: usize,
        rendered_results: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(test_expr) = state.tests.get(index) else {
            return self.finish_test_report(state, successes, failures, rendered_results);
        };

        let Some(test_case) = parse_test_case(test_expr) else {
            return Ok(Expr::call(state.head.clone(), state.raw_args.clone()));
        };
        let expected_messages =
            self.normalize_expected_message_exprs(test_case.expected_messages.clone())?;
        let evaluation_result =
            self.evaluate_test_case_input(test_case.input.clone(), &state.options);
        let case_state = TestReportCaseEvalState {
            report: state,
            index,
            successes,
            failures,
            rendered_results,
            test_case,
            expected_messages,
        };

        match evaluation_result {
            Ok(evaluation) => self.resume_test_report_after_evaluation(case_state, evaluation),
            Err(EvalError::InputRequired { request }) => {
                let case_state = SuspendedTestReportCase::freeze(case_state);
                crate::resume_or_suspend!(insert self, ResumeTestReportFromIndexSuspensionFrame { case_state });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn resume_test_report_after_evaluation(
        &mut self,
        case_state: TestReportCaseEvalState,
        evaluation: EvalCapturedOutcome,
    ) -> Result<Expr, EvalError> {
        let messages_exprs: Vec<Expr> = evaluation
            .messages
            .iter()
            .cloned()
            .map(runtime_message_name_expr)
            .collect();
        let messages_match = case_state
            .expected_messages
            .as_ref()
            .map_or(messages_exprs.is_empty(), |expected| {
                *expected == messages_exprs
            });
        let output_match_result = self.test_case_output_matches(
            &case_state.test_case,
            case_state.report.options.same_test.clone(),
            evaluation.value.clone(),
            case_state.test_case.expected.clone(),
        );
        match output_match_result {
            Ok(output_match) => self.resume_test_report_after_output_match(
                case_state,
                evaluation,
                messages_match,
                output_match,
            ),
            Err(EvalError::InputRequired { request }) => {
                let output_state = SuspendedTestReportOutput {
                    case: SuspendedTestReportCase::freeze(case_state),
                    evaluation: SuspendedEvalCapturedOutcome::freeze(evaluation),
                    messages_match,
                };
                crate::resume_or_suspend!(insert self, ResumeTestReportAfterEvaluationSuspensionFrame { output_state });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn resume_test_report_after_output_match(
        &mut self,
        case: TestReportCaseEvalState,
        evaluation: EvalCapturedOutcome,
        messages_match: bool,
        output_match: bool,
    ) -> Result<Expr, EvalError> {
        let TestReportCaseEvalState {
            report,
            index,
            mut successes,
            mut failures,
            mut rendered_results,
            test_case,
            expected_messages,
        } = case;
        let (success, rendered) = render_test_case_result(
            test_case,
            expected_messages,
            evaluation,
            messages_match,
            output_match,
        );
        if success {
            successes += 1;
        } else {
            failures += 1;
        }
        rendered_results.push(rendered);
        self.resume_test_report_from_index(report, index + 1, successes, failures, rendered_results)
    }

    fn resume_suspended_test_report_after_evaluation(
        &mut self,
        case: SuspendedTestReportCase,
        evaluation: EvalCapturedOutcome,
    ) -> Result<Expr, EvalError> {
        let messages_exprs = evaluation
            .messages
            .iter()
            .cloned()
            .map(runtime_message_name_expr)
            .collect::<Vec<_>>();
        let messages_match = case
            .expected_messages
            .as_ref()
            .map_or(messages_exprs.is_empty(), |expected| {
                expected.as_ref() == &messages_exprs
            });
        let output_match_result = self.test_case_output_matches_parts(
            case.test_case.same_test.clone(),
            case.report.options.same_test.clone(),
            evaluation.value.clone(),
            case.test_case.expected.clone(),
        );

        match output_match_result {
            Ok(output_match) => self.finish_suspended_test_report_case(
                case,
                evaluation,
                messages_match,
                output_match,
            ),
            Err(EvalError::InputRequired { request }) => {
                let output = SuspendedTestReportOutput {
                    case,
                    evaluation: SuspendedEvalCapturedOutcome::freeze(evaluation),
                    messages_match,
                };
                crate::resume_or_suspend!(insert self, ResumeSuspendedTestReportAfterEvaluationSuspensionFrame { output });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn resume_suspended_test_report_after_output_match(
        &mut self,
        output: SuspendedTestReportOutput,
        output_match: bool,
    ) -> Result<Expr, EvalError> {
        self.finish_suspended_test_report_case(
            output.case,
            output.evaluation.into_owned(),
            output.messages_match,
            output_match,
        )
    }

    fn finish_suspended_test_report_case(
        &mut self,
        mut case: SuspendedTestReportCase,
        evaluation: EvalCapturedOutcome,
        messages_match: bool,
        output_match: bool,
    ) -> Result<Expr, EvalError> {
        let current_test_case = std::mem::replace(
            &mut case.test_case,
            SuspendedTestCaseSpec {
                input: system_expr("Null"),
                expected: system_expr("Null"),
                test_id: None,
                same_test: None,
            },
        );
        let test_case = TestCaseSpec {
            input: current_test_case.input,
            expected: current_test_case.expected,
            test_id: current_test_case.test_id,
            expected_messages: None,
            same_test: current_test_case.same_test,
        };
        let expected_messages = case.expected_messages.take().map(SharedArgs::into_vec);
        let (success, rendered) = render_test_case_result(
            test_case,
            expected_messages,
            evaluation,
            messages_match,
            output_match,
        );
        if success {
            case.successes += 1;
        } else {
            case.failures += 1;
        }
        case.rendered_results = case.rendered_results.appended(rendered);
        case.index += 1;
        self.resume_suspended_test_report_from_index(case)
    }

    fn resume_suspended_test_report_from_index(
        &mut self,
        mut state: SuspendedTestReportCase,
    ) -> Result<Expr, EvalError> {
        let Some(test_expr) = state.report.tests.get(state.index) else {
            return self.finish_suspended_test_report(state);
        };
        let Some(test_case) = parse_test_case(test_expr) else {
            return Ok(Expr::call(
                state.report.head,
                state.report.raw_args.into_vec(),
            ));
        };
        let expected_messages =
            self.normalize_expected_message_exprs(test_case.expected_messages.clone())?;
        let evaluation_result =
            self.evaluate_test_case_input(test_case.input.clone(), &state.report.options);
        state.test_case = SuspendedTestCaseSpec {
            input: test_case.input,
            expected: test_case.expected,
            test_id: test_case.test_id,
            same_test: test_case.same_test,
        };
        state.expected_messages = expected_messages.map(SharedArgs::new);

        match evaluation_result {
            Ok(evaluation) => self.resume_suspended_test_report_after_evaluation(state, evaluation),
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, ResumeSuspendedTestReportFromIndexSuspensionFrame { state });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_suspended_test_report(
        &mut self,
        state: SuspendedTestReportCase,
    ) -> Result<Expr, EvalError> {
        let rendered_results = state.rendered_results.into_vec();
        let report = render_test_report_summary(state.successes, state.failures, rendered_results);
        self.project_association_properties(
            "TestReport",
            &[Expr::string("<omitted>")],
            report,
            state.report.properties,
        )
    }

    fn finish_test_report(
        &mut self,
        state: TestReportEvalState,
        successes: usize,
        failures: usize,
        rendered_results: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let report = render_test_report_summary(successes, failures, rendered_results);

        self.project_association_properties(
            "TestReport",
            &[Expr::string("<omitted>")],
            report,
            state.properties,
        )
    }

    pub(in crate::runtime::debug_tools) fn project_evaluation_data_properties(
        &mut self,
        data: Expr,
        properties: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        self.project_association_properties(
            "EvaluationData",
            &[Expr::string("<omitted>")],
            data,
            properties,
        )
    }

    fn merge_test_report_objects(&self, expr: &Expr) -> Option<Expr> {
        let reports = parse_test_report_objects(expr)?;
        let mut tests_run = 0usize;
        let mut tests_succeeded = 0usize;
        let mut tests_failed = 0usize;
        let mut merged_results = Vec::new();

        for report in reports {
            tests_run = tests_run.saturating_add(report.tests_run);
            tests_succeeded = tests_succeeded.saturating_add(report.tests_succeeded);
            tests_failed = tests_failed.saturating_add(report.tests_failed);
            merged_results.extend(report.test_results);
        }

        Some(render_association(vec![
            (Expr::string("Success"), debug_bool_expr(tests_failed == 0)),
            (
                Expr::string("TestsRunCount"),
                Expr::integer(BigInt::from(tests_run as u64)),
            ),
            (
                Expr::string("TestsSucceededCount"),
                Expr::integer(BigInt::from(tests_succeeded as u64)),
            ),
            (
                Expr::string("TestsFailedCount"),
                Expr::integer(BigInt::from(tests_failed as u64)),
            ),
            (
                Expr::string("TestResults"),
                Expr::call(system_expr("List"), merged_results),
            ),
        ]))
    }
}

fn render_test_case_result(
    test_case: TestCaseSpec,
    expected_messages: Option<Vec<Expr>>,
    evaluation: EvalCapturedOutcome,
    messages_match: bool,
    output_match: bool,
) -> (bool, Expr) {
    let EvalCapturedOutcome {
        value: actual,
        failure_type,
        messages,
        prints,
        timing,
    } = evaluation;
    let failure_type = failure_type.map(str::to_string);
    let messages_exprs = messages
        .iter()
        .cloned()
        .map(runtime_message_name_expr)
        .collect::<Vec<_>>();
    let messages_text = messages
        .iter()
        .map(|message| Expr::string(message.text.clone()))
        .collect::<Vec<_>>();
    let messages_emitted = messages
        .into_iter()
        .map(runtime_message_emit_expr)
        .collect::<Vec<_>>();
    let output_log = prints.into_iter().map(Expr::string).collect::<Vec<_>>();

    let success = output_match && messages_match && failure_type.is_none();
    let mut entries = vec![
        (
            Expr::string("Input"),
            Expr::call(system_expr("Hold"), vec![test_case.input]),
        ),
        (Expr::string("Success"), debug_bool_expr(success)),
        (Expr::string("ActualOutput"), actual),
        (
            Expr::string("ExpectedOutput"),
            Expr::call(system_expr("Hold"), vec![test_case.expected]),
        ),
        (
            Expr::string("ActualMessages"),
            Expr::call(system_expr("List"), messages_exprs),
        ),
        (
            Expr::string("ActualMessagesExpressions"),
            Expr::call(system_expr("List"), messages_emitted),
        ),
        (
            Expr::string("ActualMessagesText"),
            Expr::call(system_expr("List"), messages_text),
        ),
        (
            Expr::string("ExpectedMessages"),
            Expr::call(system_expr("List"), expected_messages.unwrap_or_default()),
        ),
        (
            Expr::string("OutputLog"),
            Expr::call(system_expr("List"), output_log),
        ),
        (Expr::string("Timing"), timing),
        (
            Expr::string("MessagesMatch"),
            debug_bool_expr(messages_match),
        ),
        (
            Expr::string("Outcome"),
            Expr::string(if success { "Success" } else { "Failure" }),
        ),
    ];

    if let Some(failure_type) = failure_type {
        entries.push((Expr::string("FailureType"), Expr::string(failure_type)));
    }
    if let Some(test_id) = test_case.test_id {
        entries.push((Expr::string("TestID"), test_id));
    }

    (success, render_association(entries))
}

fn render_test_report_summary(
    successes: usize,
    failures: usize,
    rendered_results: Vec<Expr>,
) -> Expr {
    render_association(vec![
        (Expr::string("Success"), debug_bool_expr(failures == 0)),
        (
            Expr::string("TestsRunCount"),
            Expr::integer(BigInt::from(rendered_results.len() as u64)),
        ),
        (
            Expr::string("TestsSucceededCount"),
            Expr::integer(BigInt::from(successes as u64)),
        ),
        (
            Expr::string("TestsFailedCount"),
            Expr::integer(BigInt::from(failures as u64)),
        ),
        (
            Expr::string("TestResults"),
            Expr::call(system_expr("List"), rendered_results),
        ),
    ])
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<TestReportCaseEvalState>();
    assert_traces::<TestReportEvalState>();
    assert_traces::<TestReportResultAccumulator>();
    assert_traces::<SuspendedTestReportCase>();
    assert_traces::<SuspendedTestReportOutput>();
};
