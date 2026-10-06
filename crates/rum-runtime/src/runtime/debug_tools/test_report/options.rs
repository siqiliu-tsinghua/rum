use super::model::TestReportOptions;
use super::*;
use crate::runtime::kernel_api::{
    FrameRootTrace, FrameRootVisitor, FrozenOptionReadContinuation, OptionReadContinuation,
};
use std::sync::Arc;

#[derive(Clone, Copy, FrameRootTrace)]
enum TestReportOptionStage {
    SameTest,
    Time,
    Memory,
}

struct TestReportOptionWorking<'a> {
    head: &'a Expr,
    args: &'a SharedArgs,
    properties: Option<Expr>,
    explicit: &'a SymbolOptions,
    stage: TestReportOptionStage,
    options: &'a mut TestReportOptions,
}

#[derive(FrameRootTrace)]
struct TestReportOptionRead {
    head: Expr,
    args: SharedArgs,
    properties: Option<Expr>,
    explicit: Arc<SymbolOptions>,
    stage: TestReportOptionStage,
    options: TestReportOptions,
}

pub(super) fn read_test_report_options(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
    properties: Option<Expr>,
    explicit: &SymbolOptions,
) -> Result<Expr, EvalError> {
    let mut options = TestReportOptions::default();
    TestReportOptionWorking {
        head,
        args,
        properties,
        explicit,
        stage: TestReportOptionStage::SameTest,
        options: &mut options,
    }
    .read(session)
}

impl TestReportOptionWorking<'_> {
    fn read(self, session: &mut Session) -> Result<Expr, EvalError> {
        let name = match self.stage {
            TestReportOptionStage::SameTest => "SameTest",
            TestReportOptionStage::Time => "TimeConstraint",
            TestReportOptionStage::Memory => "MemoryConstraint",
        };
        session.runtime_option_expr_for_symbol(
            Some(self.explicit),
            &Symbol::system("TestReport"),
            name,
            self,
        )
    }
}

impl OptionReadContinuation<Option<Expr>> for TestReportOptionWorking<'_> {
    type Frozen = TestReportOptionRead;
    #[inline(always)]
    fn finish(mut self, session: &mut Session, value: Option<Expr>) -> Result<Expr, EvalError> {
        match self.stage {
            TestReportOptionStage::SameTest => {
                self.options.same_test = Some(value.unwrap_or_else(|| system_expr("SameQ")));
                self.stage = TestReportOptionStage::Time;
                self.read(session)
            }
            TestReportOptionStage::Time => {
                self.options.time_constraint = test_report_constraint_expr(value);
                self.stage = TestReportOptionStage::Memory;
                self.read(session)
            }
            TestReportOptionStage::Memory => {
                self.options.memory_constraint = test_report_constraint_expr(value);
                session.start_test_report(
                    self.head,
                    self.args,
                    self.properties,
                    std::mem::take(self.options),
                )
            }
        }
    }
    fn freeze(self) -> Self::Frozen {
        TestReportOptionRead {
            head: self.head.clone(),
            args: self.args.clone(),
            properties: self.properties,
            explicit: Arc::new(self.explicit.clone()),
            stage: self.stage,
            options: self.options.clone(),
        }
    }
}

impl FrozenOptionReadContinuation<Option<Expr>> for TestReportOptionRead {
    fn finish(&self, session: &mut Session, value: Option<Expr>) -> Result<Expr, EvalError> {
        let mut options = self.options.clone();
        TestReportOptionWorking {
            head: &self.head,
            args: &self.args,
            properties: self.properties.clone(),
            explicit: &self.explicit,
            stage: self.stage,
            options: &mut options,
        }
        .finish(session, value)
    }
}

impl Session {
    pub(super) fn parse_test_report_options(&self, expr: &Expr) -> Option<SymbolOptions> {
        let options = self.compile_assigned_options_from_resolved(expr).ok()?;
        for (name, _, _) in options.iter() {
            if !matches!(
                name.name(),
                "SameTest" | "TimeConstraint" | "MemoryConstraint"
            ) {
                return None;
            }
        }
        Some(options)
    }
}

fn test_report_constraint_expr(resolved: Option<Expr>) -> Option<Expr> {
    resolved.filter(|value| !is_system_symbol_name(value, "Infinity"))
}
