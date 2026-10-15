use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use crate::runtime::suspension::{ContinueToken, TypedAction, TypedFrame};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum CasesTraversalNormalizationError {
    InvalidThirdArgument,
    InvalidFourthArgument,
    InvalidFifthArgument,
    InvalidHeadsOption {
        positional_count: usize,
        option_expr: Expr,
        classification: HeadsOptionClassification,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum CasesTraversalResolution {
    Valid(CasesTraversalConfig),
    Invalid(CasesTraversalNormalizationError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum TraversalWithoutLimitNormalizationError {
    InvalidThirdArgument,
    InvalidFourthArgument,
    InvalidHeadsOption {
        positional_count: usize,
        option_expr: Expr,
        classification: HeadsOptionClassification,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum TraversalWithoutLimitResolution {
    Valid(CasesTraversalConfig),
    Invalid(TraversalWithoutLimitNormalizationError),
}

impl CasesTraversalNormalizationError {
    pub(in crate::runtime) fn message(&self) -> &'static str {
        match self {
            Self::InvalidThirdArgument => {
                "expected a standard levelspec or Heads -> True/False as the third argument"
            }
            Self::InvalidFourthArgument => {
                "expected a non-negative match limit or Heads -> True/False as the fourth argument"
            }
            Self::InvalidFifthArgument => {
                "expected a Heads -> True/False option as the fifth argument"
            }
            Self::InvalidHeadsOption {
                positional_count, ..
            } => match positional_count {
                2 => "expected a standard levelspec or Heads -> True/False as the third argument",
                3 => {
                    "expected a non-negative match limit or Heads -> True/False as the fourth argument"
                }
                4 => "expected a Heads -> True/False option as the fifth argument",
                _ => "expected a valid Heads option",
            },
        }
    }
}

impl TraversalWithoutLimitNormalizationError {
    pub(in crate::runtime) fn message(&self) -> &'static str {
        match self {
            Self::InvalidThirdArgument => {
                "expected a standard levelspec or Heads -> True/False as the third argument"
            }
            Self::InvalidFourthArgument => {
                "expected a Heads -> True/False option as the fourth argument"
            }
            Self::InvalidHeadsOption {
                positional_count, ..
            } => match positional_count {
                2 => "expected a standard levelspec or Heads -> True/False as the third argument",
                3 => "expected a Heads -> True/False option as the fourth argument",
                _ => "expected a valid Heads option",
            },
        }
    }
}

impl Session {
    fn compile_traversal_from_resolved_exprs(
        &self,
        resolved: &[Expr],
        defaults: CasesTraversalConfig,
    ) -> Result<CasesTraversalConfig, CasesTraversalNormalizationError> {
        match resolved {
            [] => Ok(defaults),
            [third] => {
                let classification = classify_heads_option_expr(third);
                if let HeadsOptionClassification::Value(Some(include_heads)) = classification {
                    return Ok(CasesTraversalConfig {
                        include_heads,
                        ..defaults
                    });
                }

                if let Some(levels) = parse_standard_level_spec(third) {
                    Ok(CasesTraversalConfig {
                        levels,
                        max_matches: defaults.max_matches,
                        include_heads: defaults.include_heads,
                    })
                } else if heads_option_classification_looks_like_option_syntax(&classification) {
                    Err(CasesTraversalNormalizationError::InvalidHeadsOption {
                        positional_count: 2,
                        option_expr: third.clone(),
                        classification,
                    })
                } else {
                    Err(CasesTraversalNormalizationError::InvalidThirdArgument)
                }
            }
            [third, fourth] => {
                let Some(levels) = parse_standard_level_spec(third) else {
                    return Err(CasesTraversalNormalizationError::InvalidThirdArgument);
                };

                let classification = classify_heads_option_expr(fourth);
                if let HeadsOptionClassification::Value(include_heads) = classification {
                    Ok(CasesTraversalConfig {
                        levels,
                        max_matches: defaults.max_matches,
                        include_heads: include_heads.unwrap_or(defaults.include_heads),
                    })
                } else if let Some(max_matches) = parse_non_negative_match_limit(fourth) {
                    Ok(CasesTraversalConfig {
                        levels,
                        max_matches: Some(max_matches),
                        include_heads: defaults.include_heads,
                    })
                } else if heads_option_classification_looks_like_option_syntax(&classification) {
                    Err(CasesTraversalNormalizationError::InvalidHeadsOption {
                        positional_count: 3,
                        option_expr: fourth.clone(),
                        classification,
                    })
                } else {
                    Err(CasesTraversalNormalizationError::InvalidFourthArgument)
                }
            }
            [third, fourth, fifth] => {
                let Some(levels) = parse_standard_level_spec(third) else {
                    return Err(CasesTraversalNormalizationError::InvalidThirdArgument);
                };
                let Some(max_matches) = parse_non_negative_match_limit(fourth) else {
                    return Err(CasesTraversalNormalizationError::InvalidFourthArgument);
                };
                let classification = classify_heads_option_expr(fifth);
                let HeadsOptionClassification::Value(include_heads) = classification else {
                    return Err(CasesTraversalNormalizationError::InvalidHeadsOption {
                        positional_count: 4,
                        option_expr: fifth.clone(),
                        classification,
                    });
                };

                Ok(CasesTraversalConfig {
                    levels,
                    max_matches: Some(max_matches),
                    include_heads: include_heads.unwrap_or(defaults.include_heads),
                })
            }
            _ => unreachable!("traversal arity is validated before traversal normalization"),
        }
    }

    fn compile_traversal_without_limit_from_resolved_exprs(
        &self,
        resolved: &[Expr],
        defaults: CasesTraversalConfig,
    ) -> Result<CasesTraversalConfig, TraversalWithoutLimitNormalizationError> {
        match resolved {
            [] => Ok(defaults),
            [third] => {
                let classification = classify_heads_option_expr(third);
                if let HeadsOptionClassification::Value(Some(include_heads)) = classification {
                    return Ok(CasesTraversalConfig {
                        include_heads,
                        ..defaults
                    });
                }

                if let Some(levels) = parse_standard_level_spec(third) {
                    Ok(CasesTraversalConfig { levels, ..defaults })
                } else if heads_option_classification_looks_like_option_syntax(&classification) {
                    Err(
                        TraversalWithoutLimitNormalizationError::InvalidHeadsOption {
                            positional_count: 2,
                            option_expr: third.clone(),
                            classification,
                        },
                    )
                } else {
                    Err(TraversalWithoutLimitNormalizationError::InvalidThirdArgument)
                }
            }
            [third, fourth] => {
                let Some(levels) = parse_standard_level_spec(third) else {
                    return Err(TraversalWithoutLimitNormalizationError::InvalidThirdArgument);
                };
                let classification = classify_heads_option_expr(fourth);
                let HeadsOptionClassification::Value(include_heads) = classification else {
                    return Err(
                        TraversalWithoutLimitNormalizationError::InvalidHeadsOption {
                            positional_count: 3,
                            option_expr: fourth.clone(),
                            classification,
                        },
                    );
                };

                Ok(CasesTraversalConfig {
                    levels,
                    include_heads: include_heads.unwrap_or(defaults.include_heads),
                    ..defaults
                })
            }
            _ => {
                unreachable!("count/member/free traversal arity is validated before normalization")
            }
        }
    }

    pub(in crate::runtime) fn invalid_cases_traversal_call(
        &mut self,
        symbol: &str,
        args: &[Expr],
        error: CasesTraversalNormalizationError,
    ) -> Expr {
        let fallback = error.message().to_string();
        match error {
            CasesTraversalNormalizationError::InvalidThirdArgument => {
                self.invalid_level_spec_call(symbol, args, &args[2])
            }
            CasesTraversalNormalizationError::InvalidFourthArgument => {
                self.invalid_non_negative_or_infinity_call(symbol, args, 4)
            }
            CasesTraversalNormalizationError::InvalidHeadsOption {
                positional_count,
                option_expr,
                classification,
            } => self.invalid_heads_option_call(
                symbol,
                args,
                &option_expr,
                positional_count,
                classification,
                fallback,
            ),
            CasesTraversalNormalizationError::InvalidFifthArgument => {
                self.invalid_call_with_arg_form(symbol, args, fallback)
            }
        }
    }

    pub(in crate::runtime) fn invalid_traversal_without_limit_call(
        &mut self,
        symbol: &str,
        args: &[Expr],
        error: TraversalWithoutLimitNormalizationError,
    ) -> Expr {
        let fallback = error.message().to_string();
        match error {
            TraversalWithoutLimitNormalizationError::InvalidThirdArgument => {
                self.invalid_level_spec_call(symbol, args, &args[2])
            }
            TraversalWithoutLimitNormalizationError::InvalidHeadsOption {
                positional_count,
                option_expr,
                classification,
            } => self.invalid_heads_option_call(
                symbol,
                args,
                &option_expr,
                positional_count,
                classification,
                fallback,
            ),
            TraversalWithoutLimitNormalizationError::InvalidFourthArgument => {
                self.invalid_call_with_arg_form(symbol, args, fallback)
            }
        }
    }

    pub(in crate::runtime) fn compile_cases_traversal_from_resolved_exprs(
        &self,
        resolved: &[Expr],
        defaults: CasesTraversalConfig,
    ) -> Result<CasesTraversalConfig, CasesTraversalNormalizationError> {
        self.compile_traversal_from_resolved_exprs(resolved, defaults)
    }

    pub(in crate::runtime) fn compile_position_traversal_from_resolved_exprs(
        &self,
        resolved: &[Expr],
        defaults: CasesTraversalConfig,
    ) -> Result<CasesTraversalConfig, CasesTraversalNormalizationError> {
        self.compile_traversal_from_resolved_exprs(resolved, defaults)
    }

    pub(in crate::runtime) fn resolve_cases_traversal_owned(
        &mut self,
        exprs: Vec<Expr>,
        defaults: CasesTraversalConfig,
    ) -> Result<CasesTraversalResolution, EvalError> {
        let resolved_exprs_result = self.resolve_argument_exprs_owned(exprs);
        crate::resume_typed_or_suspend!(
            self,
            resolved_exprs_result,
            ok(resolved_exprs) => { let session = self; (|| {
                    Ok(
                        match session.compile_cases_traversal_from_resolved_exprs(&resolved_exprs, defaults)
                        {
                            Ok(config) => CasesTraversalResolution::Valid(config),
                            Err(error) => CasesTraversalResolution::Invalid(error),
                        },
                    )
                })() },
            suspend => ResolveCasesTraversalOwnedSuspensionFrame { defaults },
        )
    }

    pub(in crate::runtime) fn resolve_cases_traversal_shared_from(
        &mut self,
        exprs: SharedArgs,
        cursor: usize,
        defaults: CasesTraversalConfig,
    ) -> Result<CasesTraversalResolution, EvalError> {
        let resolved_exprs_result = self.resolve_argument_exprs_shared_from(exprs, cursor);
        crate::resume_typed_or_suspend!(
            self,
            resolved_exprs_result,
            ok(resolved_exprs) => { let session = self; (|| {
                    Ok(
                        match session.compile_cases_traversal_from_resolved_exprs(&resolved_exprs, defaults)
                        {
                            Ok(config) => CasesTraversalResolution::Valid(config),
                            Err(error) => CasesTraversalResolution::Invalid(error),
                        },
                    )
                })() },
            suspend => ResolveCasesTraversalOwnedSuspensionFrame { defaults },
        )
    }

    pub(in crate::runtime) fn resolve_position_traversal_owned(
        &mut self,
        exprs: Vec<Expr>,
        defaults: CasesTraversalConfig,
    ) -> Result<CasesTraversalResolution, EvalError> {
        let resolved_exprs_result = self.resolve_argument_exprs_owned(exprs);
        crate::resume_typed_or_suspend!(
            self,
            resolved_exprs_result,
            ok(resolved_exprs) => { let session = self; (|| {
                    Ok(
                        match session
                            .compile_position_traversal_from_resolved_exprs(&resolved_exprs, defaults)
                        {
                            Ok(config) => CasesTraversalResolution::Valid(config),
                            Err(error) => CasesTraversalResolution::Invalid(error),
                        },
                    )
                })() },
            suspend => ResolvePositionTraversalOwnedSuspensionFrame { defaults },
        )
    }

    pub(in crate::runtime) fn resolve_traversal_without_limit_owned(
        &mut self,
        exprs: Vec<Expr>,
        defaults: CasesTraversalConfig,
    ) -> Result<TraversalWithoutLimitResolution, EvalError> {
        let resolved_exprs_result = self.resolve_argument_exprs_owned(exprs);
        crate::resume_typed_or_suspend!(
            self,
            resolved_exprs_result,
            ok(resolved_exprs) => { let session = self; (|| {
                    Ok(
                        match session
                            .compile_traversal_without_limit_from_resolved_exprs(&resolved_exprs, defaults)
                        {
                            Ok(config) => TraversalWithoutLimitResolution::Valid(config),
                            Err(error) => TraversalWithoutLimitResolution::Invalid(error),
                        },
                    )
                })() },
            suspend => ResolveTraversalWithoutLimitOwnedSuspensionFrame { defaults },
        )
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResolveCasesTraversalOwnedSuspensionFrame {
    defaults: crate::runtime::CasesTraversalConfig,
}

impl TypedFrame for ResolveCasesTraversalOwnedSuspensionFrame {
    type Input = Vec<Expr>;
    type Output = CasesTraversalResolution;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Vec<Expr>, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<CasesTraversalResolution> {
        #[allow(unused_mut)]
        let mut defaults = self.defaults.clone();
        let resolved_exprs = match outcome {
            Ok(resolved_exprs) => resolved_exprs,
            Err(error) => return TypedAction::complete(Err(error)),
        };
        TypedAction::complete((|| {
            Ok(
                match session.compile_cases_traversal_from_resolved_exprs(&resolved_exprs, defaults)
                {
                    Ok(config) => CasesTraversalResolution::Valid(config),
                    Err(error) => CasesTraversalResolution::Invalid(error),
                },
            )
        })())
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResolvePositionTraversalOwnedSuspensionFrame {
    defaults: crate::runtime::CasesTraversalConfig,
}

impl TypedFrame for ResolvePositionTraversalOwnedSuspensionFrame {
    type Input = Vec<Expr>;
    type Output = CasesTraversalResolution;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Vec<Expr>, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<CasesTraversalResolution> {
        #[allow(unused_mut)]
        let mut defaults = self.defaults.clone();
        let resolved_exprs = match outcome {
            Ok(resolved_exprs) => resolved_exprs,
            Err(error) => return TypedAction::complete(Err(error)),
        };
        TypedAction::complete((|| {
            Ok(
                match session
                    .compile_position_traversal_from_resolved_exprs(&resolved_exprs, defaults)
                {
                    Ok(config) => CasesTraversalResolution::Valid(config),
                    Err(error) => CasesTraversalResolution::Invalid(error),
                },
            )
        })())
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResolveTraversalWithoutLimitOwnedSuspensionFrame {
    defaults: crate::runtime::CasesTraversalConfig,
}

impl TypedFrame for ResolveTraversalWithoutLimitOwnedSuspensionFrame {
    type Input = Vec<Expr>;
    type Output = TraversalWithoutLimitResolution;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Vec<Expr>, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<TraversalWithoutLimitResolution> {
        #[allow(unused_mut)]
        let mut defaults = self.defaults.clone();
        let resolved_exprs = match outcome {
            Ok(resolved_exprs) => resolved_exprs,
            Err(error) => return TypedAction::complete(Err(error)),
        };
        TypedAction::complete((|| {
            Ok(
                match session
                    .compile_traversal_without_limit_from_resolved_exprs(&resolved_exprs, defaults)
                {
                    Ok(config) => TraversalWithoutLimitResolution::Valid(config),
                    Err(error) => TraversalWithoutLimitResolution::Invalid(error),
                },
            )
        })())
    }
}
