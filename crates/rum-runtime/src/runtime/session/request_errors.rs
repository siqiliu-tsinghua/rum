//! The public failure surface of host requests: how an `EvalError` maps to a
//! `SessionRequestError`.
use super::super::*;

pub(super) fn no_suspended_evaluation_failure() -> SessionRequestFailure {
    SessionRequestFailure {
        error: SessionRequestError::Other {
            message: "no suspended evaluation is waiting for host input".to_string(),
        },
        messages: Vec::new(),
        prints: Vec::new(),
        trace: Vec::new(),
    }
}

pub(super) fn request_syntax_failure(
    diagnostic: rum_syntax::SourceDiagnostic,
) -> SessionRequestFailure {
    SessionRequestFailure {
        error: SessionRequestError::Syntax(diagnostic.structured()),
        messages: Vec::new(),
        prints: Vec::new(),
        trace: Vec::new(),
    }
}

pub(super) fn session_request_error_from_eval_error(err: &EvalError) -> SessionRequestError {
    session_request_error_from_eval_error_with_source(err, None)
}

pub(super) fn session_request_error_from_eval_error_with_source(
    err: &EvalError,
    source: Option<&str>,
) -> SessionRequestError {
    match err {
        EvalError::ContinuationSlotMismatch(fault) => {
            SessionRequestError::ContinuationSlotMismatch {
                expected: fault.expected(),
                got: fault.got(),
            }
        }
        EvalError::Syntax(error) => source
            .map(|source| {
                SessionRequestError::Syntax(
                    rum_syntax::SourceDiagnostic::new(
                        rum_syntax::SourceText::new(source),
                        error.clone(),
                    )
                    .structured(),
                )
            })
            .unwrap_or_else(|| SessionRequestError::Other {
                message: error.to_string(),
            }),
        EvalError::InvalidInteger { literal } => SessionRequestError::InvalidInteger {
            literal: literal.clone(),
        },
        EvalError::Io { path, source } => SessionRequestError::Io {
            path: path.clone(),
            message: source.to_string(),
        },
        EvalError::LoadPathNotFound { target, searched } => SessionRequestError::LoadPathNotFound {
            target: target.clone(),
            searched: searched.clone(),
        },
        EvalError::NoFileIoProvider => SessionRequestError::Other {
            message: "no file provider installed".to_string(),
        },
        EvalError::EvaluationCycleDetected { expr } => {
            SessionRequestError::EvaluationCycleDetected { expr: expr.clone() }
        }
        EvalError::RecursionLimitExceeded { limit } => {
            SessionRequestError::RecursionLimitExceeded { limit: *limit }
        }
        EvalError::StackLimitExceeded { limit } => {
            SessionRequestError::StackLimitExceeded { limit: *limit }
        }
        EvalError::MatchRecursionLimitExceeded { limit } => {
            SessionRequestError::MatchRecursionLimitExceeded { limit: *limit }
        }
        EvalError::Cancelled => SessionRequestError::Cancelled,
        EvalError::TimeConstraintExceeded => SessionRequestError::TimeConstraintExceeded,
        EvalError::MemoryConstraintExceeded { limit } => {
            SessionRequestError::MemoryConstraintExceeded { limit: *limit }
        }
        EvalError::GeneratedElementLimitExceeded { limit } => {
            SessionRequestError::GeneratedElementLimitExceeded { limit: *limit }
        }
        EvalError::IntegerWorkLimitExceeded { limit } => {
            SessionRequestError::IntegerWorkLimitExceeded { limit: *limit }
        }
        EvalError::IterationWorkLimitExceeded { limit } => {
            SessionRequestError::IterationWorkLimitExceeded { limit: *limit }
        }
        EvalError::IterationLimitExceeded { limit } => {
            SessionRequestError::IterationLimitExceeded { limit: *limit }
        }
        EvalError::EvaluationPassLimitExceeded { limit } => {
            SessionRequestError::EvaluationPassLimitExceeded { limit: *limit }
        }
        EvalError::Exit { code } => SessionRequestError::Exit { code: *code },
        EvalError::Return { value } => SessionRequestError::Other {
            message: format!("uncaught top-level Return resolved to {value}"),
        },
        EvalError::Thrown { value, tag, .. } => SessionRequestError::Other {
            message: format!("uncaught throw: {value} with tag {tag}"),
        },
        EvalError::ThrownBypassingCatch {
            value,
            tag,
            cleanup_value,
            cleanup_tag,
            ..
        } => SessionRequestError::Other {
            message: format!(
                "uncaught throw: {cleanup_value} with tag {cleanup_tag}; uncaught throw: {value} with tag {tag}"
            ),
        },
        EvalError::Abort | EvalError::ProtectedAbort => SessionRequestError::Other {
            message: "uncaught Abort[]".to_string(),
        },
        EvalError::Break => SessionRequestError::Other {
            message: "uncaught Break[]".to_string(),
        },
        EvalError::Continue => SessionRequestError::Other {
            message: "uncaught Continue[]".to_string(),
        },
        EvalError::InputRequired { request } => SessionRequestError::InputRequired {
            request: request.clone(),
        },
        EvalError::ExceptionThrown { exception } => SessionRequestError::Other {
            message: format!("uncaught exception: {exception}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_error_mapper_preserves_non_request_surface_limit_codes() {
        let memory = session_request_error_from_eval_error_with_source(
            &EvalError::MemoryConstraintExceeded { limit: 20 },
            Some("MemoryConstrained[Table[x, 100], 20]"),
        );
        assert!(matches!(
            memory,
            SessionRequestError::MemoryConstraintExceeded { limit: 20 }
        ));

        let pass = session_request_error_from_eval_error_with_source(
            &EvalError::EvaluationPassLimitExceeded { limit: 2 },
            Some("loopA"),
        );
        assert!(matches!(
            pass,
            SessionRequestError::EvaluationPassLimitExceeded { limit: 2 }
        ));
    }
}
