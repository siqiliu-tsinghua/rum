use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime::debug_tools::test_report) struct TestCaseSpec {
    pub(in crate::runtime::debug_tools::test_report) input: Expr,
    pub(in crate::runtime::debug_tools::test_report) expected: Expr,
    pub(in crate::runtime::debug_tools::test_report) test_id: Option<Expr>,
    pub(in crate::runtime::debug_tools::test_report) expected_messages: Option<Vec<Expr>>,
    pub(in crate::runtime::debug_tools::test_report) same_test: Option<Expr>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime::debug_tools::test_report) struct TestReportOptions {
    pub(in crate::runtime::debug_tools::test_report) same_test: Option<Expr>,
    pub(in crate::runtime::debug_tools::test_report) time_constraint: Option<Expr>,
    pub(in crate::runtime::debug_tools::test_report) memory_constraint: Option<Expr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime::debug_tools::test_report) struct ParsedTestReportObject {
    pub(in crate::runtime::debug_tools::test_report) tests_run: usize,
    pub(in crate::runtime::debug_tools::test_report) tests_succeeded: usize,
    pub(in crate::runtime::debug_tools::test_report) tests_failed: usize,
    pub(in crate::runtime::debug_tools::test_report) test_results: Vec<Expr>,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct EvalCapturedOutcome {
    pub(in crate::runtime::debug_tools::test_report) value: Expr,
    pub(in crate::runtime::debug_tools::test_report) failure_type: Option<&'static str>,
    pub(in crate::runtime::debug_tools::test_report) messages: Vec<RuntimeMessage>,
    pub(in crate::runtime::debug_tools::test_report) prints: Vec<String>,
    pub(in crate::runtime::debug_tools::test_report) timing: Expr,
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<EvalCapturedOutcome>();
    assert_traces::<TestCaseSpec>();
    assert_traces::<TestReportOptions>();
};
