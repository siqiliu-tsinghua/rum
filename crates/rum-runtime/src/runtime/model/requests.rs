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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct PendingEvalRequest {
    pub(in crate::runtime) message_checkpoint: usize,
    pub(in crate::runtime) print_checkpoint: usize,
    pub(in crate::runtime) handled_error_checkpoint: usize,
    pub(in crate::runtime) trace_checkpoint: usize,
}
