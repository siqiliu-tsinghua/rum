use super::*;

fn tagged(name: &'static str, args: Vec<Expr>) -> Expr {
    Expr::call(system_expr(name), args)
}

fn tagged_args(expr: Expr, name: &'static str) -> std::vec::IntoIter<Expr> {
    let Expr::Call { head, args } = expr else {
        panic!("unexpected {name} error-continuation payload")
    };
    assert!(
        is_system_symbol_name(&head, name),
        "unexpected {name} error-continuation tag: {head:?}"
    );
    args.into_iter()
}

fn string_expr(value: impl Into<String>) -> Expr {
    Expr::string(value)
}

fn string_from_expr(expr: Expr, field: &str) -> String {
    let Expr::Atom(Atom::String(value)) = expr else {
        panic!("error-continuation {field} should be a string: {expr:?}")
    };
    value
}

fn usize_expr(value: usize) -> Expr {
    string_expr(value.to_string())
}

fn usize_from_expr(expr: Expr, field: &str) -> usize {
    string_from_expr(expr, field)
        .parse()
        .unwrap_or_else(|_| panic!("error-continuation {field} should be a usize"))
}

fn i64_expr(value: i64) -> Expr {
    string_expr(value.to_string())
}

fn i64_from_expr(expr: Expr, field: &str) -> i64 {
    string_from_expr(expr, field)
        .parse()
        .unwrap_or_else(|_| panic!("error-continuation {field} should be an i64"))
}

fn span_expr(span: rum_syntax::Span) -> Expr {
    tagged(
        "InternalContinuationSyntaxSpan",
        vec![usize_expr(span.start), usize_expr(span.end)],
    )
}

fn span_from_expr(expr: Expr) -> rum_syntax::Span {
    let mut args = tagged_args(expr, "InternalContinuationSyntaxSpan");
    let span = rum_syntax::Span::new(
        usize_from_expr(
            args.next().expect("syntax span stores its start"),
            "span start",
        ),
        usize_from_expr(args.next().expect("syntax span stores its end"), "span end"),
    );
    debug_assert!(args.next().is_none());
    span
}

fn static_syntax_text(value: String) -> &'static str {
    match value.as_str() {
        "(" => "(",
        ")" => ")",
        "," => ",",
        "<|" => "<|",
        "[" => "[",
        "[[" => "[[",
        "]" => "]",
        "]]" => "]]",
        "expression" => "expression",
        "|>" => "|>",
        "~" => "~",
        other => panic!("unknown static syntax-continuation text: {other}"),
    }
}

fn syntax_error_into_expr(error: SyntaxError) -> Expr {
    match error {
        SyntaxError::EmptyInput => tagged("InternalContinuationSyntaxEmptyInput", Vec::new()),
        SyntaxError::UnexpectedToken {
            expected,
            found,
            span,
        } => tagged(
            "InternalContinuationSyntaxUnexpectedToken",
            vec![string_expr(expected), string_expr(found), span_expr(span)],
        ),
        SyntaxError::Incomplete { expected, span } => tagged(
            "InternalContinuationSyntaxIncomplete",
            vec![string_expr(expected), span_expr(span)],
        ),
        SyntaxError::UnclosedDelimiter {
            expected,
            span,
            opening_span,
            opening,
        } => tagged(
            "InternalContinuationSyntaxUnclosedDelimiter",
            vec![
                string_expr(expected),
                span_expr(span),
                span_expr(opening_span),
                string_expr(opening),
            ],
        ),
        SyntaxError::TrailingInput { span } => tagged(
            "InternalContinuationSyntaxTrailingInput",
            vec![span_expr(span)],
        ),
        SyntaxError::ParseDepthExceeded { max, span } => tagged(
            "InternalContinuationSyntaxParseDepthExceeded",
            vec![usize_expr(max), span_expr(span)],
        ),
        SyntaxError::StatementChainTooLong { max, span } => tagged(
            "InternalContinuationSyntaxStatementChainTooLong",
            vec![usize_expr(max), span_expr(span)],
        ),
        SyntaxError::LeftAssociativeChainTooLong { max, span } => tagged(
            "InternalContinuationSyntaxLeftAssociativeChainTooLong",
            vec![usize_expr(max), span_expr(span)],
        ),
        SyntaxError::UnexpectedCharacter { found, span } => tagged(
            "InternalContinuationSyntaxUnexpectedCharacter",
            vec![string_expr(found.to_string()), span_expr(span)],
        ),
        SyntaxError::UnterminatedString { span } => tagged(
            "InternalContinuationSyntaxUnterminatedString",
            vec![span_expr(span)],
        ),
        SyntaxError::UnterminatedComment { span } => tagged(
            "InternalContinuationSyntaxUnterminatedComment",
            vec![span_expr(span)],
        ),
    }
}

fn syntax_error_from_expr(expr: Expr) -> SyntaxError {
    let Expr::Call { head, args } = expr else {
        panic!("unexpected syntax-error continuation payload")
    };
    let mut args = args.into_iter();
    let error = if is_system_symbol_name(&head, "InternalContinuationSyntaxEmptyInput") {
        SyntaxError::EmptyInput
    } else if is_system_symbol_name(&head, "InternalContinuationSyntaxUnexpectedToken") {
        SyntaxError::UnexpectedToken {
            expected: static_syntax_text(string_from_expr(
                args.next()
                    .expect("unexpected-token error stores expected text"),
                "expected token",
            )),
            found: string_from_expr(
                args.next()
                    .expect("unexpected-token error stores found text"),
                "found token",
            ),
            span: span_from_expr(args.next().expect("unexpected-token error stores its span")),
        }
    } else if is_system_symbol_name(&head, "InternalContinuationSyntaxIncomplete") {
        SyntaxError::Incomplete {
            expected: static_syntax_text(string_from_expr(
                args.next().expect("incomplete error stores expected text"),
                "expected token",
            )),
            span: span_from_expr(args.next().expect("incomplete error stores its span")),
        }
    } else if is_system_symbol_name(&head, "InternalContinuationSyntaxUnclosedDelimiter") {
        SyntaxError::UnclosedDelimiter {
            expected: static_syntax_text(string_from_expr(
                args.next()
                    .expect("unclosed-delimiter error stores expected text"),
                "expected delimiter",
            )),
            span: span_from_expr(
                args.next()
                    .expect("unclosed-delimiter error stores its closing span"),
            ),
            opening_span: span_from_expr(
                args.next()
                    .expect("unclosed-delimiter error stores its opening span"),
            ),
            opening: static_syntax_text(string_from_expr(
                args.next()
                    .expect("unclosed-delimiter error stores opening text"),
                "opening delimiter",
            )),
        }
    } else if is_system_symbol_name(&head, "InternalContinuationSyntaxTrailingInput") {
        SyntaxError::TrailingInput {
            span: span_from_expr(args.next().expect("trailing-input error stores its span")),
        }
    } else if is_system_symbol_name(&head, "InternalContinuationSyntaxParseDepthExceeded") {
        SyntaxError::ParseDepthExceeded {
            max: usize_from_expr(
                args.next().expect("parse-depth error stores its limit"),
                "parse-depth limit",
            ),
            span: span_from_expr(args.next().expect("parse-depth error stores its span")),
        }
    } else if is_system_symbol_name(&head, "InternalContinuationSyntaxStatementChainTooLong") {
        SyntaxError::StatementChainTooLong {
            max: usize_from_expr(
                args.next().expect("statement-chain error stores its limit"),
                "statement-chain limit",
            ),
            span: span_from_expr(args.next().expect("statement-chain error stores its span")),
        }
    } else if is_system_symbol_name(
        &head,
        "InternalContinuationSyntaxLeftAssociativeChainTooLong",
    ) {
        SyntaxError::LeftAssociativeChainTooLong {
            max: usize_from_expr(
                args.next()
                    .expect("left-associative-chain error stores its limit"),
                "left-associative-chain limit",
            ),
            span: span_from_expr(
                args.next()
                    .expect("left-associative-chain error stores its span"),
            ),
        }
    } else if is_system_symbol_name(&head, "InternalContinuationSyntaxUnexpectedCharacter") {
        let found = string_from_expr(
            args.next()
                .expect("unexpected-character error stores its character"),
            "unexpected character",
        )
        .chars()
        .next()
        .expect("unexpected-character continuation should not be empty");
        SyntaxError::UnexpectedCharacter {
            found,
            span: span_from_expr(
                args.next()
                    .expect("unexpected-character error stores its span"),
            ),
        }
    } else if is_system_symbol_name(&head, "InternalContinuationSyntaxUnterminatedString") {
        SyntaxError::UnterminatedString {
            span: span_from_expr(
                args.next()
                    .expect("unterminated-string error stores its span"),
            ),
        }
    } else if is_system_symbol_name(&head, "InternalContinuationSyntaxUnterminatedComment") {
        SyntaxError::UnterminatedComment {
            span: span_from_expr(
                args.next()
                    .expect("unterminated-comment error stores its span"),
            ),
        }
    } else {
        panic!("unexpected syntax-error continuation tag: {head:?}")
    };
    debug_assert!(args.next().is_none());
    error
}

impl ContinuationValue for EvalError {
    fn into_continuation_expr(self) -> Expr {
        match self {
            EvalError::ContinuationSlotMismatch(_) => {
                unreachable!(
                    "a structural slot fault cannot enter the legacy error wire; this wire and guard are removed at R"
                )
            }
            EvalError::Syntax(error) => tagged(
                "InternalContinuationEvalErrorSyntax",
                vec![syntax_error_into_expr(error)],
            ),
            EvalError::InvalidInteger { literal } => tagged(
                "InternalContinuationEvalErrorInvalidInteger",
                vec![string_expr(literal)],
            ),
            EvalError::Io { path, source } => tagged(
                "InternalContinuationEvalErrorIo",
                vec![string_expr(path), string_expr(source.message())],
            ),
            EvalError::NoFileIoProvider => {
                tagged("InternalContinuationEvalErrorNoFileIoProvider", Vec::new())
            }
            EvalError::LoadPathNotFound { target, searched } => tagged(
                "InternalContinuationEvalErrorLoadPathNotFound",
                vec![
                    string_expr(target),
                    tagged(
                        "InternalContinuationEvalErrorSearchedPaths",
                        searched.into_iter().map(string_expr).collect(),
                    ),
                ],
            ),
            EvalError::EvaluationCycleDetected { expr } => tagged(
                "InternalContinuationEvalErrorEvaluationCycleDetected",
                vec![string_expr(expr)],
            ),
            EvalError::RecursionLimitExceeded { limit } => tagged(
                "InternalContinuationEvalErrorRecursionLimitExceeded",
                vec![usize_expr(limit)],
            ),
            EvalError::StackLimitExceeded { limit } => tagged(
                "InternalContinuationEvalErrorStackLimitExceeded",
                vec![usize_expr(limit)],
            ),
            EvalError::MatchRecursionLimitExceeded { limit } => tagged(
                "InternalContinuationEvalErrorMatchRecursionLimitExceeded",
                vec![usize_expr(limit)],
            ),
            EvalError::Cancelled => tagged("InternalContinuationEvalErrorCancelled", Vec::new()),
            EvalError::TimeConstraintExceeded => tagged(
                "InternalContinuationEvalErrorTimeConstraintExceeded",
                Vec::new(),
            ),
            EvalError::MemoryConstraintExceeded { limit } => tagged(
                "InternalContinuationEvalErrorMemoryConstraintExceeded",
                vec![usize_expr(limit)],
            ),
            EvalError::GeneratedElementLimitExceeded { limit } => tagged(
                "InternalContinuationEvalErrorGeneratedElementLimitExceeded",
                vec![usize_expr(limit)],
            ),
            EvalError::IntegerWorkLimitExceeded { limit } => tagged(
                "InternalContinuationEvalErrorIntegerWorkLimitExceeded",
                vec![usize_expr(limit)],
            ),
            EvalError::IterationWorkLimitExceeded { limit } => tagged(
                "InternalContinuationEvalErrorIterationWorkLimitExceeded",
                vec![usize_expr(limit)],
            ),
            EvalError::IterationLimitExceeded { limit } => tagged(
                "InternalContinuationEvalErrorIterationLimitExceeded",
                vec![usize_expr(limit)],
            ),
            EvalError::EvaluationPassLimitExceeded { limit } => tagged(
                "InternalContinuationEvalErrorEvaluationPassLimitExceeded",
                vec![usize_expr(limit)],
            ),
            EvalError::Return { value } => {
                tagged("InternalContinuationEvalErrorReturn", vec![value])
            }
            EvalError::Thrown {
                value,
                tag,
                explicit_tag,
            } => tagged(
                "InternalContinuationEvalErrorThrown",
                vec![value, tag, explicit_tag.into_continuation_expr()],
            ),
            EvalError::ThrownBypassingCatch {
                value,
                tag,
                explicit_tag,
                cleanup_value,
                cleanup_tag,
                cleanup_explicit_tag,
            } => tagged(
                "InternalContinuationEvalErrorThrownBypassingCatch",
                vec![
                    value,
                    tag,
                    explicit_tag.into_continuation_expr(),
                    cleanup_value,
                    cleanup_tag,
                    cleanup_explicit_tag.into_continuation_expr(),
                ],
            ),
            EvalError::Abort => tagged("InternalContinuationEvalErrorAbort", Vec::new()),
            EvalError::ProtectedAbort => {
                tagged("InternalContinuationEvalErrorProtectedAbort", Vec::new())
            }
            EvalError::Break => tagged("InternalContinuationEvalErrorBreak", Vec::new()),
            EvalError::Continue => tagged("InternalContinuationEvalErrorContinue", Vec::new()),
            EvalError::Exit { code } => {
                tagged("InternalContinuationEvalErrorExit", vec![i64_expr(code)])
            }
            EvalError::InputRequired { request } => tagged(
                "InternalContinuationEvalErrorInputRequired",
                vec![
                    bool::from(matches!(request.kind, RuntimeInputKind::Input))
                        .into_continuation_expr(),
                    string_expr(request.prompt),
                    request.password.into_continuation_expr(),
                ],
            ),
            EvalError::ExceptionThrown { exception } => tagged(
                "InternalContinuationEvalErrorExceptionThrown",
                vec![exception],
            ),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        let Expr::Call { head, args } = expr else {
            panic!("unexpected evaluation-error continuation payload")
        };
        let mut args = args.into_iter();
        let error = if is_system_symbol_name(&head, "InternalContinuationEvalErrorSyntax") {
            Self::Syntax(syntax_error_from_expr(
                args.next()
                    .expect("syntax evaluation error stores its error"),
            ))
        } else if is_system_symbol_name(&head, "InternalContinuationEvalErrorInvalidInteger") {
            Self::InvalidInteger {
                literal: string_from_expr(
                    args.next()
                        .expect("invalid-integer error stores its literal"),
                    "integer literal",
                ),
            }
        } else if is_system_symbol_name(&head, "InternalContinuationEvalErrorIo") {
            Self::Io {
                path: string_from_expr(args.next().expect("I/O error stores its path"), "I/O path"),
                source: FileIoError::new(string_from_expr(
                    args.next().expect("I/O error stores its source message"),
                    "I/O source",
                )),
            }
        } else if is_system_symbol_name(&head, "InternalContinuationEvalErrorNoFileIoProvider") {
            Self::NoFileIoProvider
        } else if is_system_symbol_name(&head, "InternalContinuationEvalErrorLoadPathNotFound") {
            let target = string_from_expr(
                args.next().expect("load-path error stores its target"),
                "load target",
            );
            let searched_expr = args
                .next()
                .expect("load-path error stores its searched paths");
            let searched = tagged_args(searched_expr, "InternalContinuationEvalErrorSearchedPaths")
                .map(|path| string_from_expr(path, "searched path"))
                .collect();
            Self::LoadPathNotFound { target, searched }
        } else if is_system_symbol_name(
            &head,
            "InternalContinuationEvalErrorEvaluationCycleDetected",
        ) {
            Self::EvaluationCycleDetected {
                expr: string_from_expr(
                    args.next().expect("cycle error stores its expression"),
                    "cycle expression",
                ),
            }
        } else if is_system_symbol_name(
            &head,
            "InternalContinuationEvalErrorRecursionLimitExceeded",
        ) {
            Self::RecursionLimitExceeded {
                limit: usize_from_expr(
                    args.next().expect("recursion error stores its limit"),
                    "recursion limit",
                ),
            }
        } else if is_system_symbol_name(&head, "InternalContinuationEvalErrorStackLimitExceeded") {
            Self::StackLimitExceeded {
                limit: usize_from_expr(
                    args.next().expect("stack error stores its limit"),
                    "stack limit",
                ),
            }
        } else if is_system_symbol_name(
            &head,
            "InternalContinuationEvalErrorMatchRecursionLimitExceeded",
        ) {
            Self::MatchRecursionLimitExceeded {
                limit: usize_from_expr(
                    args.next().expect("match-recursion error stores its limit"),
                    "match-recursion limit",
                ),
            }
        } else if is_system_symbol_name(&head, "InternalContinuationEvalErrorCancelled") {
            Self::Cancelled
        } else if is_system_symbol_name(
            &head,
            "InternalContinuationEvalErrorTimeConstraintExceeded",
        ) {
            Self::TimeConstraintExceeded
        } else if is_system_symbol_name(
            &head,
            "InternalContinuationEvalErrorMemoryConstraintExceeded",
        ) {
            Self::MemoryConstraintExceeded {
                limit: usize_from_expr(
                    args.next().expect("memory error stores its limit"),
                    "memory limit",
                ),
            }
        } else if is_system_symbol_name(
            &head,
            "InternalContinuationEvalErrorGeneratedElementLimitExceeded",
        ) {
            Self::GeneratedElementLimitExceeded {
                limit: usize_from_expr(
                    args.next()
                        .expect("generated-element error stores its limit"),
                    "generated-element limit",
                ),
            }
        } else if is_system_symbol_name(
            &head,
            "InternalContinuationEvalErrorIntegerWorkLimitExceeded",
        ) {
            Self::IntegerWorkLimitExceeded {
                limit: usize_from_expr(
                    args.next().expect("integer-work error stores its limit"),
                    "integer-work limit",
                ),
            }
        } else if is_system_symbol_name(
            &head,
            "InternalContinuationEvalErrorIterationWorkLimitExceeded",
        ) {
            Self::IterationWorkLimitExceeded {
                limit: usize_from_expr(
                    args.next().expect("iteration-work error stores its limit"),
                    "iteration-work limit",
                ),
            }
        } else if is_system_symbol_name(
            &head,
            "InternalContinuationEvalErrorIterationLimitExceeded",
        ) {
            Self::IterationLimitExceeded {
                limit: usize_from_expr(
                    args.next().expect("iteration error stores its limit"),
                    "iteration limit",
                ),
            }
        } else if is_system_symbol_name(
            &head,
            "InternalContinuationEvalErrorEvaluationPassLimitExceeded",
        ) {
            Self::EvaluationPassLimitExceeded {
                limit: usize_from_expr(
                    args.next().expect("evaluation-pass error stores its limit"),
                    "evaluation-pass limit",
                ),
            }
        } else if is_system_symbol_name(&head, "InternalContinuationEvalErrorReturn") {
            Self::Return {
                value: args.next().expect("return error stores its value"),
            }
        } else if is_system_symbol_name(&head, "InternalContinuationEvalErrorThrown") {
            Self::Thrown {
                value: args.next().expect("throw error stores its value"),
                tag: args.next().expect("throw error stores its tag"),
                explicit_tag: bool::from_continuation_expr(
                    args.next()
                        .expect("throw error stores its explicit-tag flag"),
                ),
            }
        } else if is_system_symbol_name(&head, "InternalContinuationEvalErrorThrownBypassingCatch")
        {
            Self::ThrownBypassingCatch {
                value: args.next().expect("bypassing throw stores its value"),
                tag: args.next().expect("bypassing throw stores its tag"),
                explicit_tag: bool::from_continuation_expr(
                    args.next()
                        .expect("bypassing throw stores its explicit-tag flag"),
                ),
                cleanup_value: args
                    .next()
                    .expect("bypassing throw stores its cleanup value"),
                cleanup_tag: args.next().expect("bypassing throw stores its cleanup tag"),
                cleanup_explicit_tag: bool::from_continuation_expr(
                    args.next()
                        .expect("bypassing throw stores its cleanup explicit-tag flag"),
                ),
            }
        } else if is_system_symbol_name(&head, "InternalContinuationEvalErrorAbort") {
            Self::Abort
        } else if is_system_symbol_name(&head, "InternalContinuationEvalErrorProtectedAbort") {
            Self::ProtectedAbort
        } else if is_system_symbol_name(&head, "InternalContinuationEvalErrorBreak") {
            Self::Break
        } else if is_system_symbol_name(&head, "InternalContinuationEvalErrorContinue") {
            Self::Continue
        } else if is_system_symbol_name(&head, "InternalContinuationEvalErrorExit") {
            Self::Exit {
                code: i64_from_expr(
                    args.next().expect("exit error stores its code"),
                    "exit code",
                ),
            }
        } else if is_system_symbol_name(&head, "InternalContinuationEvalErrorInputRequired") {
            let input =
                bool::from_continuation_expr(args.next().expect("input error stores its kind"));
            let prompt = string_from_expr(
                args.next().expect("input error stores its prompt"),
                "input prompt",
            );
            let password = bool::from_continuation_expr(
                args.next().expect("input error stores its password flag"),
            );
            let mut request = RuntimeInputRequest::new(prompt);
            request.kind = if input {
                RuntimeInputKind::Input
            } else {
                RuntimeInputKind::InputString
            };
            request.password = password;
            Self::InputRequired { request }
        } else if is_system_symbol_name(&head, "InternalContinuationEvalErrorExceptionThrown") {
            Self::ExceptionThrown {
                exception: args.next().expect("exception error stores its exception"),
            }
        } else {
            panic!("unexpected evaluation-error continuation tag: {head:?}")
        };
        debug_assert!(args.next().is_none());
        error
    }
}

pub(in crate::runtime) fn result_into_continuation_expr(outcome: Result<Expr, EvalError>) -> Expr {
    match outcome {
        Ok(value) => tagged("InternalContinuationEvalResultOk", vec![value]),
        Err(error) => tagged(
            "InternalContinuationEvalResultErr",
            vec![error.into_continuation_expr()],
        ),
    }
}

pub(in crate::runtime) fn result_from_continuation_expr(expr: Expr) -> Result<Expr, EvalError> {
    let Expr::Call { head, args } = expr else {
        panic!("unexpected evaluation-result continuation payload")
    };
    let mut args = args.into_iter();
    let outcome = if is_system_symbol_name(&head, "InternalContinuationEvalResultOk") {
        Ok(args
            .next()
            .expect("successful evaluation result stores its value"))
    } else if is_system_symbol_name(&head, "InternalContinuationEvalResultErr") {
        Err(EvalError::from_continuation_expr(
            args.next()
                .expect("failed evaluation result stores its error"),
        ))
    } else {
        panic!("unexpected evaluation-result continuation tag: {head:?}")
    };
    debug_assert!(args.next().is_none());
    outcome
}
