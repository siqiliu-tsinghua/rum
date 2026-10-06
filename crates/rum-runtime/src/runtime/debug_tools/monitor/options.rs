use super::*;
use crate::runtime::kernel_api::{
    FrameRootTrace, FrozenOptionReadContinuation, OptionReadContinuation,
};
use std::sync::Arc;

#[derive(Clone, Copy, FrameRootTrace)]
enum MonitorOptionStage {
    Syntax,
    Evaluation,
    Step,
}

pub(super) struct MonitorOptionWorking<'a> {
    args: &'a [Expr],
    explicit: Option<&'a SymbolOptions>,
    stage: MonitorOptionStage,
    evaluation_monitor: Option<Expr>,
}

#[derive(FrameRootTrace)]
pub(super) struct MonitorOptionRead {
    args: SharedArgs,
    explicit: Option<Arc<SymbolOptions>>,
    stage: MonitorOptionStage,
    evaluation_monitor: Option<Expr>,
}

impl<'a> MonitorOptionWorking<'a> {
    pub(super) fn syntax(args: &'a [Expr]) -> Self {
        Self {
            args,
            explicit: None,
            stage: MonitorOptionStage::Syntax,
            evaluation_monitor: None,
        }
    }

    fn freeze_read(self) -> MonitorOptionRead {
        MonitorOptionRead {
            args: SharedArgs::new(self.args.to_vec()),
            explicit: self.explicit.map(|options| Arc::new(options.clone())),
            stage: self.stage,
            evaluation_monitor: self.evaluation_monitor,
        }
    }
}

impl OptionReadContinuation<Expr> for MonitorOptionWorking<'_> {
    type Frozen = MonitorOptionRead;
    fn finish(self, session: &mut Session, value: Expr) -> Result<Expr, EvalError> {
        finish_syntax(session, self.args, value)
    }
    fn freeze(self) -> Self::Frozen {
        self.freeze_read()
    }
}

impl FrozenOptionReadContinuation<Expr> for MonitorOptionRead {
    fn finish(&self, session: &mut Session, value: Expr) -> Result<Expr, EvalError> {
        finish_syntax(session, &self.args, value)
    }
}

fn finish_syntax(session: &mut Session, args: &[Expr], value: Expr) -> Result<Expr, EvalError> {
    let options = match session.compile_assigned_options_from_resolved(&value) {
        Ok(options) => options,
        Err(_) => {
            return Ok(session.invalid_call_with_arg_form(
                "Monitor", args,
                "expected the optional third argument to be a rule or list of rules in the current first-pass subset",
            ));
        }
    };
    for (name, _, _) in options.iter() {
        if !matches!(name.name(), "EvaluationMonitor" | "StepMonitor") {
            return Ok(session.invalid_call_with_arg_form(
                "Monitor", args,
                "expected only EvaluationMonitor and StepMonitor option rules in the current first-pass subset",
            ));
        }
    }
    read_evaluation_monitor(session, args, Some(&options))
}

pub(super) fn read_evaluation_monitor(
    session: &mut Session,
    args: &[Expr],
    explicit: Option<&SymbolOptions>,
) -> Result<Expr, EvalError> {
    session.runtime_option_expr_for_symbol(
        explicit,
        &Symbol::system("Monitor"),
        "EvaluationMonitor",
        MonitorOptionWorking {
            args,
            explicit,
            stage: MonitorOptionStage::Evaluation,
            evaluation_monitor: None,
        },
    )
}

impl OptionReadContinuation<Option<Expr>> for MonitorOptionWorking<'_> {
    type Frozen = MonitorOptionRead;
    fn finish(self, session: &mut Session, value: Option<Expr>) -> Result<Expr, EvalError> {
        let value = monitor_option_expr(value);
        match self.stage {
            MonitorOptionStage::Evaluation => session.runtime_option_expr_for_symbol(
                self.explicit,
                &Symbol::system("Monitor"),
                "StepMonitor",
                Self {
                    stage: MonitorOptionStage::Step,
                    evaluation_monitor: value,
                    ..self
                },
            ),
            MonitorOptionStage::Step => {
                session.start_monitor(self.args, self.evaluation_monitor, value)
            }
            MonitorOptionStage::Syntax => unreachable!("syntax has an Expr decoder"),
        }
    }
    fn freeze(self) -> Self::Frozen {
        self.freeze_read()
    }
}

impl FrozenOptionReadContinuation<Option<Expr>> for MonitorOptionRead {
    fn finish(&self, session: &mut Session, value: Option<Expr>) -> Result<Expr, EvalError> {
        MonitorOptionWorking {
            args: &self.args,
            explicit: self.explicit.as_deref(),
            stage: self.stage,
            evaluation_monitor: self.evaluation_monitor.clone(),
        }
        .finish(session, value)
    }
}

fn monitor_option_expr(resolved: Option<Expr>) -> Option<Expr> {
    match resolved {
        Some(expr) if is_system_symbol_name(&expr, "None") => None,
        other => other,
    }
}
