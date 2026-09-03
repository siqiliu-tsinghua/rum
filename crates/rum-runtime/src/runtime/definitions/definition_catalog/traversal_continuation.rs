use super::traversal::{
    CasesTraversalNormalizationError, TraversalWithoutLimitNormalizationError,
    TraversalWithoutLimitResolution,
};
use super::*;

impl ContinuationValue for TraversalWithoutLimitResolution {
    fn into_continuation_expr(self) -> Expr {
        match self {
            Self::Valid(config) => Expr::call(
                system_expr("InternalContinuationTraversalWithoutLimitValid"),
                vec![config.into_continuation_expr()],
            ),
            Self::Invalid(error) => Expr::call(
                system_expr("InternalContinuationTraversalWithoutLimitInvalid"),
                vec![error.into_continuation_expr()],
            ),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args, .. }
                if is_system_symbol_name(
                    &head,
                    "InternalContinuationTraversalWithoutLimitValid",
                ) && args.len() == 1 =>
            {
                let mut args = Expr::unwrap_shared_args(args);
                let config_expr = args
                    .pop()
                    .expect("valid traversal-without-limit stores config");
                Self::Valid(CasesTraversalConfig::from_continuation_expr(config_expr))
            }
            Expr::Call { head, args, .. }
                if is_system_symbol_name(
                    &head,
                    "InternalContinuationTraversalWithoutLimitInvalid",
                ) && args.len() == 1 =>
            {
                let mut args = Expr::unwrap_shared_args(args);
                let error_expr = args
                    .pop()
                    .expect("invalid traversal-without-limit stores error");
                Self::Invalid(
                    TraversalWithoutLimitNormalizationError::from_continuation_expr(error_expr),
                )
            }
            other => panic!(
                "unexpected continuation payload for TraversalWithoutLimitResolution: {other:?}"
            ),
        }
    }
}

impl ContinuationValue for TraversalWithoutLimitNormalizationError {
    fn into_continuation_expr(self) -> Expr {
        match self {
            Self::InvalidThirdArgument => Expr::integer(BigInt::from(3_i32)),
            Self::InvalidFourthArgument => Expr::integer(BigInt::from(4_i32)),
            Self::InvalidHeadsOption {
                positional_count,
                option_expr,
                classification,
            } => Expr::call(
                system_expr("InternalContinuationTraversalWithoutLimitHeadsOptionInvalid"),
                vec![
                    Expr::integer(BigInt::from(positional_count)),
                    option_expr,
                    classification.into_continuation_expr(),
                ],
            ),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        if let Some(code) = expr_as_non_negative_usize(&expr) {
            return match code {
                3 => Self::InvalidThirdArgument,
                4 => Self::InvalidFourthArgument,
                other => panic!("unexpected traversal-without-limit error code: {other}"),
            };
        }

        match expr {
            Expr::Call { head, args, .. }
                if is_system_symbol_name(
                    &head,
                    "InternalContinuationTraversalWithoutLimitHeadsOptionInvalid",
                ) && args.len() == 3 =>
            {
                let mut args = args.into_iter();
                let positional_count = expr_as_non_negative_usize(
                    &args.next().expect("heads option error stores a position"),
                )
                .expect("heads option error position should be an integer");
                let option_expr = args.next().expect("heads option error stores option expr");
                let classification = HeadsOptionClassification::from_continuation_expr(
                    args.next()
                        .expect("heads option error stores classification"),
                );
                Self::InvalidHeadsOption {
                    positional_count,
                    option_expr,
                    classification,
                }
            }
            other => panic!("unexpected traversal-without-limit error payload: {other:?}"),
        }
    }
}

impl ContinuationValue for CasesTraversalNormalizationError {
    fn into_continuation_expr(self) -> Expr {
        match self {
            Self::InvalidThirdArgument => Expr::integer(BigInt::from(3_i32)),
            Self::InvalidFourthArgument => Expr::integer(BigInt::from(4_i32)),
            Self::InvalidFifthArgument => Expr::integer(BigInt::from(5_i32)),
            Self::InvalidHeadsOption {
                positional_count,
                option_expr,
                classification,
            } => Expr::call(
                system_expr("InternalContinuationCasesTraversalHeadsOptionInvalid"),
                vec![
                    Expr::integer(BigInt::from(positional_count)),
                    option_expr,
                    classification.into_continuation_expr(),
                ],
            ),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        if let Some(code) = expr_as_non_negative_usize(&expr) {
            return match code {
                3 => Self::InvalidThirdArgument,
                4 => Self::InvalidFourthArgument,
                5 => Self::InvalidFifthArgument,
                other => panic!("unexpected cases-traversal error code: {other}"),
            };
        }

        match expr {
            Expr::Call { head, args, .. }
                if is_system_symbol_name(
                    &head,
                    "InternalContinuationCasesTraversalHeadsOptionInvalid",
                ) && args.len() == 3 =>
            {
                let mut args = args.into_iter();
                let positional_count = expr_as_non_negative_usize(
                    &args.next().expect("heads option error stores a position"),
                )
                .expect("heads option error position should be an integer");
                let option_expr = args.next().expect("heads option error stores option expr");
                let classification = HeadsOptionClassification::from_continuation_expr(
                    args.next()
                        .expect("heads option error stores classification"),
                );
                Self::InvalidHeadsOption {
                    positional_count,
                    option_expr,
                    classification,
                }
            }
            other => panic!("unexpected cases-traversal error payload: {other:?}"),
        }
    }
}
