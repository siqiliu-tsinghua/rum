use rum_core::{Atom, Expr, Rational};
use rum_runtime::{
    EvalTerminationReason, RuntimeInputKind, RuntimeInputRequest, RuntimeMessage, Session,
    SessionEvalResponse, SessionRequestError, SessionRequestFailure, TraceEvent,
    kernel_api::{
        ToStringFormat, association_entries_from_expr, association_entry_forms_from_expr,
        render_to_string_expr,
    },
};

const MAX_TYPED_DEPTH: usize = 64;

/// Outcome returned by a host evaluation request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostEvalOutcome {
    /// Evaluation completed with a value.
    Success(HostEvalSuccess),
    /// Evaluation suspended for host-provided input.
    InputRequired(HostInputSuspension),
    /// Evaluation failed before producing a final value.
    Failure(HostFailure),
}

/// Successful evaluation result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostEvalSuccess {
    /// Rendered and typed final value.
    pub value: HostValue,
    /// Runtime termination reason when the final value was synthesized after early termination.
    pub termination: Option<HostTerminationReason>,
    /// Messages emitted before success.
    pub messages: Vec<HostMessage>,
    /// Text printed before success.
    pub prints: Vec<String>,
    /// Trace events collected when tracing is enabled.
    pub trace: Vec<HostTraceEvent>,
    /// Flattened adapter-friendly output stream.
    pub outputs: Vec<HostStructuredOutput>,
}

/// Structured reason for a runtime-synthesized terminated evaluation value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostTerminationReason {
    /// Evaluation was cancelled through a host cancel handle.
    Cancelled,
    /// Evaluation exceeded its wall-clock time constraint.
    TimeConstraint,
    /// Evaluation exceeded its modeled memory proxy-byte ceiling.
    MemoryLimit,
    /// Evaluation exceeded its recursion limit.
    RecursionLimit,
    /// Evaluation exceeded its native stack limit.
    StackLimit,
    /// Pattern matching exceeded its recursion limit.
    PatternMatchRecursionLimit,
    /// Evaluation exceeded an iteration or fixed-point limit.
    IterationLimit,
    /// Evaluation generated more elements than allowed.
    GeneratedElementLimit,
    /// Evaluation exceeded its exact-integer work budget.
    IntegerWorkLimit,
    /// Evaluation exceeded its iteration work budget.
    IterationWorkLimit,
}

/// Suspended evaluation waiting for host input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostInputSuspension {
    /// Input request that the host should answer.
    pub request: HostInputRequest,
    /// Messages emitted before suspension.
    pub messages: Vec<HostMessage>,
    /// Text printed before suspension.
    pub prints: Vec<String>,
    /// Trace events collected before suspension.
    pub trace: Vec<HostTraceEvent>,
    /// Flattened adapter-friendly output stream collected before suspension.
    pub outputs: Vec<HostStructuredOutput>,
}

/// Failed evaluation or host request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostFailure {
    /// Coarse failure classification for host dispatch.
    pub code: HostFailureCode,
    /// Requested process exit status for an [`HostFailureCode::Exit`] failure.
    ///
    /// This is `Some(code)` only for a failure mapped from a language-level
    /// `Exit[code]` request and is `None` for every other failure.
    pub exit_status: Option<i64>,
    /// User-facing failure message.
    pub message: String,
    /// Full rendered source diagnostic for a syntax failure.
    ///
    /// This is `Some(diagnostic.rendered)` only for a failure mapped from
    /// [`SessionRequestError::Syntax`] and is `None` for every other failure.
    pub syntax_rendered: Option<String>,
    /// Messages emitted before failure.
    pub messages: Vec<HostMessage>,
    /// Text printed before failure.
    pub prints: Vec<String>,
    /// Trace events collected before failure.
    pub trace: Vec<HostTraceEvent>,
    /// Flattened adapter-friendly output stream collected before failure.
    pub outputs: Vec<HostStructuredOutput>,
}

/// Coarse host failure classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostFailureCode {
    /// A panic was caught at the host session boundary.
    Panic,
    /// The session was used after being poisoned by a caught panic.
    Poisoned,
    /// Source text could not be parsed.
    Syntax,
    /// Integer syntax or conversion failed.
    InvalidInteger,
    /// An input/output request failed.
    Io,
    /// A requested package or load path was not found.
    LoadPathNotFound,
    /// Evaluation detected a recursive definition cycle.
    EvaluationCycleDetected,
    /// The evaluator recursion limit was exceeded.
    RecursionLimitExceeded,
    /// The pattern matcher recursion limit was exceeded.
    MatchRecursionLimitExceeded,
    /// Evaluation was cancelled through a host cancel handle.
    Cancelled,
    /// The request exceeded its wall-clock time constraint.
    TimeConstraintExceeded,
    /// The request exceeded a memory constraint.
    MemoryConstraintExceeded,
    /// The request generated more elements than allowed.
    GeneratedElementLimitExceeded,
    /// The request exceeded the exact-integer work budget.
    IntegerWorkLimitExceeded,
    /// The request exceeded the iteration work budget.
    IterationWorkLimitExceeded,
    /// A language-level iteration limit was exceeded.
    IterationLimitExceeded,
    /// The evaluator pass limit was exceeded.
    EvaluationPassLimitExceeded,
    /// Evaluation requested process-style exit.
    Exit,
    /// A failure that does not map to a more specific host code.
    Other,
}

impl HostFailureCode {
    /// Returns whether this failure requires rebuilding the host session.
    ///
    /// This is `true` for [`Self::Panic`] and [`Self::Poisoned`] and `false`
    /// for every other failure code.
    ///
    /// Panics poison the session boundary, and a poisoned-session failure
    /// reports that prior state. All other failures leave the session usable.
    pub const fn requires_reset(self) -> bool {
        matches!(self, Self::Panic | Self::Poisoned)
    }
}

/// Host-facing value with both rendered and typed forms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostValue {
    /// Canonically rendered expression text with one top-level `HoldForm` peeled.
    ///
    /// This matches the value rendering used by the CLI front end. Use
    /// [`HostTyped`] for the transport-friendly structural view.
    pub text: String,
    /// Transport-friendly typed representation of the expression.
    pub typed: HostTyped,
}

/// Transport-friendly typed expression value for exact scalars, booleans,
/// lists, associations, and immediate rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostTyped {
    /// Small exact integer that fits in `i64`.
    Integer(i64),
    /// Exact integer represented as a decimal string.
    BigInteger(String),
    /// Exact rational represented by decimal numerator and denominator strings.
    ///
    /// Typed input is normalized as an exact rational, so `2/4` returns as
    /// numerator `1` and denominator `2` after evaluation.
    Rational {
        /// Decimal numerator.
        numerator: String,
        /// Decimal denominator.
        denominator: String,
    },
    /// Boolean mapped from the system symbol `True` or `False`.
    ///
    /// These two system symbols surface as `Bool` rather than [`Self::Symbol`].
    Bool(bool),
    /// Symbol name other than the system symbols `True` and `False`.
    Symbol(String),
    /// String value.
    Str(String),
    /// List elements, up to the typed-depth cap.
    List(Vec<HostTyped>),
    /// Ordered key/value pairs from an association, up to the typed-depth cap.
    ///
    /// An association containing a delayed entry has no typed data view and
    /// surfaces as [`Self::Other`].
    Association(Vec<(HostTyped, HostTyped)>),
    /// Immediate rule sides, up to the typed-depth cap.
    ///
    /// `RuleDelayed` has no typed data view and surfaces as [`Self::Other`].
    Rule {
        /// Left-hand side of the immediate rule.
        lhs: Box<HostTyped>,
        /// Right-hand side of the immediate rule.
        rhs: Box<HostTyped>,
    },
    /// Expression that does not have a specialized typed representation.
    ///
    /// This is an output-only lossy view. It deliberately has no typed input
    /// path because its rendered text would need to be parsed as source.
    Other {
        /// Rendered head name.
        head: String,
        /// Rendered expression text.
        text: String,
    },
}

/// Message emitted during evaluation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostMessage {
    /// Qualified symbol name that owns the message.
    pub symbol: String,
    /// Message tag.
    pub tag: String,
    /// Rendered message text.
    pub text: String,
}

/// Kind of input requested by a suspended evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostInputKind {
    /// Input should be treated as a raw string.
    InputString,
    /// Input should be parsed as source before resuming.
    Input,
}

/// Host input request produced by `Input` or `InputString`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostInputRequest {
    /// Requested input mode.
    pub kind: HostInputKind,
    /// Prompt text to present to the user.
    pub prompt: String,
    /// Whether the host should hide input while collecting it.
    pub password: bool,
}

/// Trace event exposed to host adapters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostTraceEvent {
    /// One evaluation pass transformed an expression.
    EvaluationPass {
        /// Expression before the pass.
        before: String,
        /// Expression after the pass.
        after: String,
    },
    /// A definition was stored in the session.
    DefinitionStored {
        /// Symbol that received the definition.
        symbol: String,
        /// Definition kind.
        kind: String,
        /// Rendered stored value.
        value: String,
    },
    /// A rule was applied during evaluation.
    RuleApplied {
        /// Rule source classification.
        source: String,
        /// Input expression before the rule.
        input: String,
        /// Rendered rule.
        rule: String,
        /// Result after applying the rule.
        result: String,
    },
}

/// Flattened output event for adapters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostStructuredOutput {
    /// Output event kind.
    pub kind: HostStructuredOutputKind,
    /// Rendered output text.
    pub text: String,
}

/// Kind of flattened structured output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostStructuredOutputKind {
    /// Final result value.
    Result,
    /// Printed text.
    Print,
    /// Message text.
    Message,
    /// Trace event text.
    Trace,
}

impl From<RuntimeMessage> for HostMessage {
    fn from(message: RuntimeMessage) -> Self {
        Self {
            symbol: message.symbol,
            tag: message.tag,
            text: message.text,
        }
    }
}

impl From<RuntimeInputRequest> for HostInputRequest {
    fn from(request: RuntimeInputRequest) -> Self {
        Self {
            kind: match request.kind {
                RuntimeInputKind::InputString => HostInputKind::InputString,
                RuntimeInputKind::Input => HostInputKind::Input,
            },
            prompt: request.prompt,
            password: request.password,
        }
    }
}

impl From<EvalTerminationReason> for HostTerminationReason {
    fn from(reason: EvalTerminationReason) -> Self {
        match reason {
            EvalTerminationReason::Cancelled => Self::Cancelled,
            EvalTerminationReason::TimeConstraint => Self::TimeConstraint,
            EvalTerminationReason::MemoryLimit => Self::MemoryLimit,
            EvalTerminationReason::RecursionLimit => Self::RecursionLimit,
            EvalTerminationReason::StackLimit => Self::StackLimit,
            EvalTerminationReason::PatternMatchRecursionLimit => Self::PatternMatchRecursionLimit,
            EvalTerminationReason::IterationLimit => Self::IterationLimit,
            EvalTerminationReason::GeneratedElementLimit => Self::GeneratedElementLimit,
            EvalTerminationReason::IntegerWorkLimit => Self::IntegerWorkLimit,
            EvalTerminationReason::IterationWorkLimit => Self::IterationWorkLimit,
        }
    }
}

impl From<TraceEvent> for HostTraceEvent {
    fn from(event: TraceEvent) -> Self {
        match event {
            TraceEvent::EvaluationPass { before, after } => Self::EvaluationPass {
                before: before.to_string(),
                after: after.to_string(),
            },
            TraceEvent::DefinitionStored {
                symbol,
                kind,
                value,
            } => Self::DefinitionStored {
                symbol,
                kind: format!("{kind:?}"),
                value: value.to_string(),
            },
            TraceEvent::RuleApplied {
                source,
                input,
                rule,
                result,
            } => Self::RuleApplied {
                source: format!("{source:?}"),
                input: input.to_string(),
                rule: rule.to_string(),
                result: result.to_string(),
            },
        }
    }
}

pub(crate) fn eval_success_from_runtime(response: SessionEvalResponse) -> HostEvalSuccess {
    let text = render_to_string_expr(
        &response.value,
        ToStringFormat::CanonicalHoldFormTransparent,
    );
    let typed = host_typed_from_expr(&response.value);
    let value = HostValue { text, typed };
    let termination = response.termination.map(Into::into);
    let messages = host_messages(response.messages);
    let prints = response.prints;
    let trace = host_trace(response.trace);
    let outputs = structured_outputs(Some(&value), &messages, &prints, &trace);

    HostEvalSuccess {
        value,
        termination,
        messages,
        prints,
        trace,
        outputs,
    }
}

pub(crate) fn eval_failure_from_runtime(failure: SessionRequestFailure) -> HostEvalOutcome {
    if let SessionRequestError::InputRequired { request } = &failure.error {
        let request = request.clone().into();
        let messages = host_messages(failure.messages);
        let prints = failure.prints;
        let trace = host_trace(failure.trace);
        let outputs = structured_outputs(None, &messages, &prints, &trace);

        return HostEvalOutcome::InputRequired(HostInputSuspension {
            request,
            messages,
            prints,
            trace,
            outputs,
        });
    }

    HostEvalOutcome::Failure(host_failure_from_runtime(failure))
}

pub(crate) fn host_failure_from_runtime(failure: SessionRequestFailure) -> HostFailure {
    let code = failure_code(&failure.error);
    let exit_status = match &failure.error {
        SessionRequestError::Exit { code } => Some(*code),
        _ => None,
    };
    let syntax_rendered = match &failure.error {
        SessionRequestError::Syntax(diagnostic) => Some(diagnostic.rendered.clone()),
        _ => None,
    };
    let message = failure.error.to_string();
    let messages = host_messages(failure.messages);
    let prints = failure.prints;
    let trace = host_trace(failure.trace);
    let outputs = structured_outputs(None, &messages, &prints, &trace);

    HostFailure {
        code,
        exit_status,
        message,
        syntax_rendered,
        messages,
        prints,
        trace,
        outputs,
    }
}

pub(crate) fn synthesized_host_failure(
    code: HostFailureCode,
    message: impl Into<String>,
) -> HostFailure {
    HostFailure {
        code,
        exit_status: None,
        message: message.into(),
        syntax_rendered: None,
        messages: Vec::new(),
        prints: Vec::new(),
        trace: Vec::new(),
        outputs: Vec::new(),
    }
}

pub(crate) fn expr_from_typed(session: &Session, typed: &HostTyped) -> Result<Expr, String> {
    expr_from_typed_at_depth(session, typed, 0)
}

impl HostTyped {
    pub(crate) fn to_expr(&self, session: &Session) -> Result<Expr, String> {
        expr_from_typed(session, self)
    }
}

fn expr_from_typed_at_depth(
    session: &Session,
    typed: &HostTyped,
    depth: usize,
) -> Result<Expr, String> {
    match typed {
        HostTyped::Integer(value) => Ok(Expr::small_integer(*value)),
        HostTyped::BigInteger(decimal) => decimal
            .parse()
            .map(Expr::integer)
            .map_err(|_| format!("invalid decimal BigInteger value: {decimal:?}")),
        HostTyped::Rational {
            numerator,
            denominator,
        } => {
            let numerator = numerator
                .parse()
                .map_err(|_| format!("invalid decimal Rational numerator: {numerator:?}"))?;
            let denominator = denominator
                .parse()
                .map_err(|_| format!("invalid decimal Rational denominator: {denominator:?}"))?;
            let rational = Rational::new(numerator, denominator)
                .ok_or_else(|| "Rational denominator must not be zero".to_string())?;
            Ok(Expr::rational(rational))
        }
        HostTyped::Bool(value) => Ok(Expr::system_symbol(if *value { "True" } else { "False" })),
        HostTyped::Symbol(raw) => session
            .resolve_symbol_name(raw, true)
            .map(|symbol| Expr::Atom(Atom::Symbol(symbol)))
            .ok_or_else(|| format!("unable to resolve Symbol value: {raw:?}")),
        HostTyped::Str(value) => Ok(Expr::string(value)),
        HostTyped::List(items) => {
            ensure_typed_container_depth(depth)?;
            let items = items
                .iter()
                .map(|item| expr_from_typed_at_depth(session, item, depth + 1))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(system_call("List", items))
        }
        HostTyped::Association(entries) => {
            ensure_typed_container_depth(depth)?;
            let entries = entries
                .iter()
                .map(|(key, value)| {
                    Ok(system_call(
                        "Rule",
                        vec![
                            expr_from_typed_at_depth(session, key, depth + 1)?,
                            expr_from_typed_at_depth(session, value, depth + 1)?,
                        ],
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(system_call("Association", entries))
        }
        HostTyped::Rule { lhs, rhs } => {
            ensure_typed_container_depth(depth)?;
            Ok(system_call(
                "Rule",
                vec![
                    expr_from_typed_at_depth(session, lhs, depth + 1)?,
                    expr_from_typed_at_depth(session, rhs, depth + 1)?,
                ],
            ))
        }
        HostTyped::Other { .. } => {
            Err("HostTyped::Other is an output-only lossy view and cannot be used as input".into())
        }
    }
}

fn ensure_typed_container_depth(depth: usize) -> Result<(), String> {
    if depth >= MAX_TYPED_DEPTH {
        Err(format!(
            "typed value exceeds the maximum input depth of {MAX_TYPED_DEPTH}"
        ))
    } else {
        Ok(())
    }
}

fn system_call(head: &str, args: Vec<Expr>) -> Expr {
    Expr::call(Expr::system_symbol(head), args)
}

fn host_typed_from_expr(expr: &Expr) -> HostTyped {
    host_typed_from_expr_at_depth(expr, 0)
}

fn host_typed_from_expr_at_depth(expr: &Expr, depth: usize) -> HostTyped {
    match expr {
        Expr::Atom(atom) => host_typed_from_atom(atom),
        Expr::Call { head, args, .. } => {
            if depth >= MAX_TYPED_DEPTH {
                return host_typed_other(head.as_ref(), expr);
            }

            if association_entry_forms_from_expr(expr)
                .is_some_and(|entries| entries.iter().any(|entry| entry.delayed))
            {
                return host_typed_other(head.as_ref(), expr);
            }

            if let Some(entries) = association_entries_from_expr(expr) {
                return HostTyped::Association(
                    entries
                        .into_iter()
                        .map(|(key, value)| {
                            (
                                host_typed_from_expr_at_depth(&key, depth + 1),
                                host_typed_from_expr_at_depth(&value, depth + 1),
                            )
                        })
                        .collect(),
                );
            }

            if is_list_head(head.as_ref()) {
                return HostTyped::List(
                    args.iter()
                        .map(|arg| host_typed_from_expr_at_depth(arg, depth + 1))
                        .collect(),
                );
            }

            if is_rule_head(head.as_ref()) {
                if let [lhs, rhs] = args.as_slice() {
                    return HostTyped::Rule {
                        lhs: Box::new(host_typed_from_expr_at_depth(lhs, depth + 1)),
                        rhs: Box::new(host_typed_from_expr_at_depth(rhs, depth + 1)),
                    };
                }
            }

            host_typed_other(head.as_ref(), expr)
        }
    }
}

fn host_typed_from_atom(atom: &Atom) -> HostTyped {
    match atom {
        Atom::Integer(value) => value
            .as_i64()
            .map(HostTyped::Integer)
            .unwrap_or_else(|| HostTyped::BigInteger(value.to_bigint().to_string())),
        Atom::Rational(value) => HostTyped::Rational {
            numerator: value.numerator().to_string(),
            denominator: value.denominator().to_string(),
        },
        Atom::String(value) => HostTyped::Str(value.clone()),
        Atom::Symbol(symbol) if symbol.is_system_name("True") => HostTyped::Bool(true),
        Atom::Symbol(symbol) if symbol.is_system_name("False") => HostTyped::Bool(false),
        Atom::Symbol(symbol) => HostTyped::Symbol(symbol.to_string()),
    }
}

fn is_list_head(expr: &Expr) -> bool {
    matches!(expr, Expr::Atom(Atom::Symbol(symbol)) if symbol.is_system_name("List"))
}

fn is_rule_head(expr: &Expr) -> bool {
    matches!(expr, Expr::Atom(Atom::Symbol(symbol)) if symbol.is_system_name("Rule"))
}

fn host_typed_other(head: &Expr, expr: &Expr) -> HostTyped {
    HostTyped::Other {
        head: host_typed_head_name(head),
        text: expr.to_string(),
    }
}

fn host_typed_head_name(head: &Expr) -> String {
    match head {
        Expr::Atom(Atom::Symbol(symbol)) => symbol.to_string(),
        _ => head.to_string(),
    }
}

fn failure_code(error: &SessionRequestError) -> HostFailureCode {
    match error {
        SessionRequestError::Syntax(_) => HostFailureCode::Syntax,
        SessionRequestError::InvalidInteger { .. } => HostFailureCode::InvalidInteger,
        SessionRequestError::Io { .. } => HostFailureCode::Io,
        SessionRequestError::LoadPathNotFound { .. } => HostFailureCode::LoadPathNotFound,
        SessionRequestError::EvaluationCycleDetected { .. } => {
            HostFailureCode::EvaluationCycleDetected
        }
        SessionRequestError::RecursionLimitExceeded { .. } => {
            HostFailureCode::RecursionLimitExceeded
        }
        SessionRequestError::MatchRecursionLimitExceeded { .. } => {
            HostFailureCode::MatchRecursionLimitExceeded
        }
        SessionRequestError::Cancelled => HostFailureCode::Cancelled,
        SessionRequestError::TimeConstraintExceeded => HostFailureCode::TimeConstraintExceeded,
        SessionRequestError::MemoryConstraintExceeded { .. } => {
            HostFailureCode::MemoryConstraintExceeded
        }
        SessionRequestError::GeneratedElementLimitExceeded { .. } => {
            HostFailureCode::GeneratedElementLimitExceeded
        }
        SessionRequestError::IntegerWorkLimitExceeded { .. } => {
            HostFailureCode::IntegerWorkLimitExceeded
        }
        SessionRequestError::IterationWorkLimitExceeded { .. } => {
            HostFailureCode::IterationWorkLimitExceeded
        }
        SessionRequestError::IterationLimitExceeded { .. } => {
            HostFailureCode::IterationLimitExceeded
        }
        SessionRequestError::EvaluationPassLimitExceeded { .. } => {
            HostFailureCode::EvaluationPassLimitExceeded
        }
        SessionRequestError::Exit { .. } => HostFailureCode::Exit,
        SessionRequestError::StackLimitExceeded { .. }
        | SessionRequestError::InputRequired { .. }
        | SessionRequestError::Other { .. } => HostFailureCode::Other,
    }
}

fn host_messages(messages: Vec<RuntimeMessage>) -> Vec<HostMessage> {
    messages.into_iter().map(Into::into).collect()
}

fn host_trace(trace: Vec<TraceEvent>) -> Vec<HostTraceEvent> {
    trace.into_iter().map(Into::into).collect()
}

fn structured_outputs(
    value: Option<&HostValue>,
    messages: &[HostMessage],
    prints: &[String],
    trace: &[HostTraceEvent],
) -> Vec<HostStructuredOutput> {
    let mut outputs = Vec::new();

    outputs.extend(prints.iter().cloned().map(|text| HostStructuredOutput {
        kind: HostStructuredOutputKind::Print,
        text,
    }));
    outputs.extend(messages.iter().map(|message| HostStructuredOutput {
        kind: HostStructuredOutputKind::Message,
        text: format!("{}::{}: {}", message.symbol, message.tag, message.text),
    }));
    outputs.extend(trace.iter().map(|event| HostStructuredOutput {
        kind: HostStructuredOutputKind::Trace,
        text: format!("{event:?}"),
    }));

    if let Some(value) = value {
        outputs.push(HostStructuredOutput {
            kind: HostStructuredOutputKind::Result,
            text: value.text.clone(),
        });
    }

    outputs
}

#[cfg(test)]
mod tests {
    use super::*;
    use rum_runtime::{RuntimeInputRequest, Session};

    #[test]
    fn failure_code_maps_runtime_request_errors_to_host_codes() {
        let syntax = Session::new()
            .parse_request("f[1,]")
            .expect_err("malformed source should produce a syntax request error")
            .error;

        let cases = vec![
            (syntax, HostFailureCode::Syntax),
            (
                SessionRequestError::InvalidInteger {
                    literal: "bad".to_string(),
                },
                HostFailureCode::InvalidInteger,
            ),
            (
                SessionRequestError::Io {
                    path: "missing.rum".to_string(),
                    message: "missing".to_string(),
                },
                HostFailureCode::Io,
            ),
            (
                SessionRequestError::LoadPathNotFound {
                    target: "Nope`".to_string(),
                    searched: vec![".".to_string()],
                },
                HostFailureCode::LoadPathNotFound,
            ),
            (
                SessionRequestError::EvaluationCycleDetected {
                    expr: "f[1]".to_string(),
                },
                HostFailureCode::EvaluationCycleDetected,
            ),
            (
                SessionRequestError::RecursionLimitExceeded { limit: 7 },
                HostFailureCode::RecursionLimitExceeded,
            ),
            (
                SessionRequestError::StackLimitExceeded { limit: 1024 },
                HostFailureCode::Other,
            ),
            (
                SessionRequestError::MatchRecursionLimitExceeded { limit: 8 },
                HostFailureCode::MatchRecursionLimitExceeded,
            ),
            (SessionRequestError::Cancelled, HostFailureCode::Cancelled),
            (
                SessionRequestError::TimeConstraintExceeded,
                HostFailureCode::TimeConstraintExceeded,
            ),
            (
                SessionRequestError::MemoryConstraintExceeded { limit: 9 },
                HostFailureCode::MemoryConstraintExceeded,
            ),
            (
                SessionRequestError::GeneratedElementLimitExceeded { limit: 10 },
                HostFailureCode::GeneratedElementLimitExceeded,
            ),
            (
                SessionRequestError::IntegerWorkLimitExceeded { limit: 11 },
                HostFailureCode::IntegerWorkLimitExceeded,
            ),
            (
                SessionRequestError::IterationWorkLimitExceeded { limit: 12 },
                HostFailureCode::IterationWorkLimitExceeded,
            ),
            (
                SessionRequestError::IterationLimitExceeded { limit: 13 },
                HostFailureCode::IterationLimitExceeded,
            ),
            (
                SessionRequestError::EvaluationPassLimitExceeded { limit: 14 },
                HostFailureCode::EvaluationPassLimitExceeded,
            ),
            (SessionRequestError::Exit { code: 5 }, HostFailureCode::Exit),
            (
                SessionRequestError::InputRequired {
                    request: RuntimeInputRequest::new("name: "),
                },
                HostFailureCode::Other,
            ),
            (
                SessionRequestError::Other {
                    message: "misc".to_string(),
                },
                HostFailureCode::Other,
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(failure_code(&error), expected, "{error:?}");
        }
    }
}
