use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

impl fmt::Display for SessionRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContinuationSlotMismatch { expected, got } => write!(
                f,
                "continuation slot mismatch: expected {expected}, got {got}"
            ),
            Self::Syntax(diagnostic) => f.write_str(&diagnostic.message),
            Self::InvalidInteger { literal } => write!(f, "invalid integer literal `{literal}`"),
            Self::Io { path, message } => write!(f, "could not read `{path}`: {message}"),
            Self::LoadPathNotFound { target, .. } => {
                write!(f, "could not resolve package or file `{target}`")
            }
            Self::EvaluationCycleDetected { expr } => {
                write!(f, "evaluation cycle detected while re-entering `{expr}`")
            }
            Self::RecursionLimitExceeded { limit } => {
                write!(f, "evaluation exceeded recursion limit ({limit})")
            }
            Self::StackLimitExceeded { limit } => {
                write!(f, "evaluation exceeded native stack limit ({limit} bytes)")
            }
            Self::MatchRecursionLimitExceeded { limit } => {
                write!(
                    f,
                    "evaluation exceeded pattern-match recursion limit ({limit})"
                )
            }
            Self::Cancelled => f.write_str("evaluation cancelled by host"),
            Self::TimeConstraintExceeded => {
                f.write_str("evaluation exceeded TimeConstrained limit")
            }
            Self::MemoryConstraintExceeded { limit } => {
                write!(f, "evaluation exceeded MemoryConstrained limit ({limit})")
            }
            Self::GeneratedElementLimitExceeded { limit } => {
                write!(f, "evaluation exceeded generated element limit ({limit})")
            }
            Self::IntegerWorkLimitExceeded { limit } => {
                write!(f, "evaluation exceeded integer work limit ({limit})")
            }
            Self::IterationWorkLimitExceeded { limit } => {
                write!(f, "evaluation exceeded iteration work limit ({limit})")
            }
            Self::IterationLimitExceeded { limit } => {
                write!(f, "ReplaceRepeated exceeded iteration limit ({limit})")
            }
            Self::EvaluationPassLimitExceeded { limit } => {
                write!(f, "evaluation exceeded fixed-point pass limit ({limit})")
            }
            Self::Exit { code } => write!(f, "Exit[{code}]"),
            Self::InputRequired { request } => write!(f, "{request}"),
            Self::Other { message } => f.write_str(message),
        }
    }
}

/// Structured reason for a runtime-synthesized terminated evaluation value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, rum_derive::FrameRootTrace)]
pub enum EvalTerminationReason {
    Cancelled,
    TimeConstraint,
    MemoryLimit,
    RecursionLimit,
    StackLimit,
    PatternMatchRecursionLimit,
    IterationLimit,
    GeneratedElementLimit,
    IntegerWorkLimit,
    IterationWorkLimit,
}

impl EvalTerminationReason {
    /// Return the language-visible termination tag.
    pub fn tag(self) -> &'static str {
        match self {
            Self::Cancelled => "Cancelled",
            Self::TimeConstraint => "TimeConstraint",
            Self::MemoryLimit => "MemoryLimit",
            Self::RecursionLimit => "RecursionLimit",
            Self::StackLimit => "StackLimit",
            Self::PatternMatchRecursionLimit => "PatternMatchRecursionLimit",
            Self::IterationLimit => "IterationLimit",
            Self::GeneratedElementLimit => "GeneratedElementLimit",
            Self::IntegerWorkLimit => "IntegerWorkLimit",
            Self::IterationWorkLimit => "IterationWorkLimit",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalOutcome {
    pub value: Expr,
    pub termination: Option<EvalTerminationReason>,
    pub messages: Vec<RuntimeMessage>,
    pub prints: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SessionParseResponse {
    pub parsed: ParsedSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionCompletionResponse {
    pub prefix: String,
    pub matches: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionCompletenessStatus {
    Complete,
    Incomplete,
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionIsCompleteResponse {
    pub status: SessionCompletenessStatus,
    pub indent: Option<String>,
    pub diagnostic: Option<StructuredSourceDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionLowerResponse {
    pub expr: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionEvalResponse {
    pub value: Expr,
    pub termination: Option<EvalTerminationReason>,
    pub messages: Vec<RuntimeMessage>,
    pub prints: Vec<String>,
    pub trace: Vec<TraceEvent>,
}

/// Summary of a WL-level symbol definition stored in a session.
///
/// Native builtins are excluded because they live in the builtin registry, not
/// in the session definitions map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionDefinitionSummary {
    pub name: String,
    pub has_own_value: bool,
    pub down_value_count: usize,
    pub up_value_count: usize,
    pub sub_value_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionInspectResponse {
    pub info: SymbolInfo,
    pub definitions: Vec<DefinitionCatalogEntry>,
    pub message_catalog: Vec<MessageCatalogEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionRequestError {
    ContinuationSlotMismatch {
        expected: &'static str,
        got: &'static str,
    },
    Syntax(StructuredSourceDiagnostic),
    InvalidInteger {
        literal: String,
    },
    Io {
        path: String,
        message: String,
    },
    LoadPathNotFound {
        target: String,
        searched: Vec<String>,
    },
    EvaluationCycleDetected {
        expr: String,
    },
    RecursionLimitExceeded {
        limit: usize,
    },
    StackLimitExceeded {
        limit: usize,
    },
    MatchRecursionLimitExceeded {
        limit: usize,
    },
    Cancelled,
    TimeConstraintExceeded,
    MemoryConstraintExceeded {
        limit: usize,
    },
    GeneratedElementLimitExceeded {
        limit: usize,
    },
    IntegerWorkLimitExceeded {
        limit: usize,
    },
    IterationWorkLimitExceeded {
        limit: usize,
    },
    IterationLimitExceeded {
        limit: usize,
    },
    EvaluationPassLimitExceeded {
        limit: usize,
    },
    Exit {
        code: i64,
    },
    InputRequired {
        request: RuntimeInputRequest,
    },
    Other {
        message: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionRequestFailure {
    pub error: SessionRequestError,
    pub messages: Vec<RuntimeMessage>,
    pub prints: Vec<String>,
    pub trace: Vec<TraceEvent>,
}

/// One published mark per channel. An input pause can settle the channels at
/// different clocks, so the marks advance independently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct ChannelMarks {
    pub(in crate::runtime) messages: OutputMark,
    pub(in crate::runtime) prints: OutputMark,
    pub(in crate::runtime) trace: OutputMark,
}

/// What one request has shown its host. Append-only, and never touched by a
/// rollback: publication copies session items, then the session lists may lose
/// them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct ReportedOutput {
    pub(in crate::runtime) messages: Vec<RuntimeMessage>,
    pub(in crate::runtime) prints: Vec<String>,
    pub(in crate::runtime) trace: Vec<TraceEvent>,
    pub(in crate::runtime) published: ChannelMarks,
}

impl ReportedOutput {
    pub(in crate::runtime) fn at(mark: OutputMark) -> Self {
        Self {
            messages: Vec::new(),
            prints: Vec::new(),
            trace: Vec::new(),
            published: ChannelMarks {
                messages: mark,
                prints: mark,
                trace: mark,
            },
        }
    }

    /// Append session items stamped after each published mark and at or before
    /// `through`, then advance that mark. A `through` behind the published mark
    /// publishes nothing and leaves the mark where it is.
    pub(in crate::runtime) fn publish(
        &mut self,
        messages: &OutputLog<RuntimeMessage>,
        prints: &OutputLog<String>,
        trace: &OutputLog<(TraceEvent, usize)>,
        through_messages: OutputMark,
        through_prints: OutputMark,
        through_trace: OutputMark,
    ) {
        // `has_after` reads one stamp, so a pause with nothing new on a channel
        // does no search there.
        if through_messages > self.published.messages {
            if messages.has_after(self.published.messages) {
                self.messages.extend(
                    messages
                        .between(self.published.messages, through_messages)
                        .cloned(),
                );
            }
            self.published.messages = through_messages;
        }
        if through_prints > self.published.prints {
            if prints.has_after(self.published.prints) {
                self.prints.extend(
                    prints
                        .between(self.published.prints, through_prints)
                        .cloned(),
                );
            }
            self.published.prints = through_prints;
        }
        if through_trace > self.published.trace {
            if trace.has_after(self.published.trace) {
                self.trace.extend(
                    trace
                        .between(self.published.trace, through_trace)
                        .map(|(event, _depth)| event.clone()),
                );
            }
            self.published.trace = through_trace;
        }
    }
}

/// The request a pending suspension belongs to: its admission mark and what
/// the host has already been shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct PendingReport {
    pub(in crate::runtime) mark: OutputMark,
    pub(in crate::runtime) reported: ReportedOutput,
}

/// What a request left waiting when it returned `InputRequired`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum PendingRequest {
    /// `eval`, `eval_expr` and `call`. Only this variant can be resumed.
    Resumable(PendingReport),
    /// `eval_script` and `eval_file`, which are not resumable, as documented.
    /// Retirement still publishes the reported prefix.
    Batch(PendingReport),
}

impl PendingRequest {
    pub(in crate::runtime) fn report(&self) -> &PendingReport {
        match self {
            Self::Resumable(report) | Self::Batch(report) => report,
        }
    }

    pub(in crate::runtime) fn report_mut(&mut self) -> &mut PendingReport {
        match self {
            Self::Resumable(report) | Self::Batch(report) => report,
        }
    }

    pub(in crate::runtime) fn into_report(self) -> PendingReport {
        match self {
            Self::Resumable(report) | Self::Batch(report) => report,
        }
    }
}
