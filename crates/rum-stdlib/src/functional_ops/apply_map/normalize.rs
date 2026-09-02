use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TraversalConfig {
    pub(super) levels: CasesLevelSpec,
    pub(super) include_heads: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ApplyFlavor {
    Apply,
    MapApply,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MapLikeFlavor {
    Map,
    Scan,
    MapIndexed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MapTraversalFlavor {
    Map,
    MapAll,
}

pub(super) enum ApplyNormalization {
    OperatorForm(Expr),
    Call {
        operator: Expr,
        expr: Expr,
        config: TraversalConfig,
    },
    Immediate(Expr),
}

pub(super) enum MapLikeNormalization {
    OperatorForm(Expr),
    Call {
        operator: Expr,
        expr: Expr,
        config: TraversalConfig,
    },
    Immediate(Expr),
}

pub(super) enum CommonArgMapNormalization {
    OperatorForm(Expr),
    Call {
        functions: Expr,
        common_expr: Expr,
        config: TraversalConfig,
    },
    Immediate(Expr),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ScanCollection {
    pub(super) parts: Vec<Expr>,
    pub(super) depth: usize,
}

impl ApplyFlavor {
    pub(super) fn symbol(self) -> &'static str {
        match self {
            Self::Apply => "Apply",
            Self::MapApply => "MapApply",
        }
    }

    pub(super) fn default_levels(self) -> CasesLevelSpec {
        match self {
            Self::Apply => CasesLevelSpec::exact(0),
            Self::MapApply => CasesLevelSpec::exact(1),
        }
    }

    pub(super) fn max_args(self) -> usize {
        match self {
            Self::Apply => 4,
            Self::MapApply => 3,
        }
    }
}

impl MapLikeFlavor {
    pub(super) fn symbol(self) -> &'static str {
        match self {
            Self::Map => "Map",
            Self::Scan => "Scan",
            Self::MapIndexed => "MapIndexed",
        }
    }
}

pub(super) fn normalize_apply_call(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    flavor: ApplyFlavor,
) -> ApplyNormalization {
    let symbol = flavor.symbol();

    if !(1..=flavor.max_args()).contains(&args.len()) {
        return ApplyNormalization::Immediate(session.invalid_call_with_arg_range(
            symbol,
            args,
            1,
            flavor.max_args(),
        ));
    }

    if args.len() == 1 {
        return ApplyNormalization::OperatorForm(unary_operator_form(symbol, &args[0]));
    }

    if args.len() == 2 && parse_heads_option(&args[1]).is_some() {
        return ApplyNormalization::OperatorForm(unary_operator_form_with_tail(
            symbol,
            args[0].clone(),
            args[1].clone(),
        ));
    }

    let default_levels = flavor.default_levels();

    let (expr, levels, include_heads) = match args {
        [_, expr] => (expr.clone(), default_levels, false),
        [_, expr, third] => {
            if let Some(include_heads) = parse_heads_option(third) {
                (expr.clone(), default_levels, include_heads)
            } else if flavor == ApplyFlavor::Apply {
                let Some(levels) = parse_standard_level_spec(third) else {
                    return ApplyNormalization::Immediate(session.invalid_call_with_arg_form(
                        symbol,
                        args,
                        "expected a standard levelspec or Heads -> True/False as the third argument",
                    ));
                };
                (expr.clone(), levels, false)
            } else {
                return ApplyNormalization::Immediate(session.invalid_call_with_arg_form(
                    symbol,
                    args,
                    "expected a Heads -> True/False option as the third argument",
                ));
            }
        }
        [_, expr, third, fourth] if flavor == ApplyFlavor::Apply => {
            let Some(levels) = parse_standard_level_spec(third) else {
                return ApplyNormalization::Immediate(session.invalid_call_with_arg_form(
                    symbol,
                    args,
                    "expected a standard levelspec as the third argument",
                ));
            };
            let Some(include_heads) = parse_heads_option_with_default(fourth, false) else {
                return ApplyNormalization::Immediate(session.invalid_call_with_arg_form(
                    symbol,
                    args,
                    "expected a Heads -> True/False option as the fourth argument",
                ));
            };
            (expr.clone(), levels, include_heads)
        }
        _ => return ApplyNormalization::Immediate(Expr::call(head.clone(), args.to_vec())),
    };

    ApplyNormalization::Call {
        operator: args[0].clone(),
        expr,
        config: TraversalConfig {
            levels,
            include_heads,
        },
    }
}

pub(super) fn normalize_map_like_call(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    flavor: MapLikeFlavor,
) -> MapLikeNormalization {
    let symbol = flavor.symbol();

    if !(1..=4).contains(&args.len()) {
        return MapLikeNormalization::Immediate(
            session.invalid_call_with_arg_range(symbol, args, 1, 4),
        );
    }

    if args.len() == 1 {
        return MapLikeNormalization::OperatorForm(unary_operator_form(symbol, &args[0]));
    }

    if args.len() == 2 && parse_heads_option(&args[1]).is_some() {
        return MapLikeNormalization::OperatorForm(unary_operator_form_with_tail(
            symbol,
            args[0].clone(),
            args[1].clone(),
        ));
    }

    let (expr, levels, include_heads) = match args {
        [_, expr] => (expr.clone(), CasesLevelSpec::exact(1), false),
        [_, expr, third] => {
            if let Some(include_heads) = parse_heads_option(third) {
                (expr.clone(), CasesLevelSpec::exact(1), include_heads)
            } else {
                let Some(levels) = parse_standard_level_spec(third) else {
                    return MapLikeNormalization::Immediate(session.invalid_call_with_arg_form(
                        symbol,
                        args,
                        "expected a standard levelspec or Heads -> True/False as the third argument",
                    ));
                };
                (expr.clone(), levels, false)
            }
        }
        [_, expr, third, fourth] => {
            let Some(levels) = parse_standard_level_spec(third) else {
                return MapLikeNormalization::Immediate(session.invalid_call_with_arg_form(
                    symbol,
                    args,
                    "expected a standard levelspec as the third argument",
                ));
            };
            let Some(include_heads) = parse_heads_option_with_default(fourth, false) else {
                return MapLikeNormalization::Immediate(session.invalid_call_with_arg_form(
                    symbol,
                    args,
                    "expected a Heads -> True/False option as the fourth argument",
                ));
            };
            (expr.clone(), levels, include_heads)
        }
        _ => return MapLikeNormalization::Immediate(Expr::call(head.clone(), args.to_vec())),
    };

    MapLikeNormalization::Call {
        operator: args[0].clone(),
        expr,
        config: TraversalConfig {
            levels,
            include_heads,
        },
    }
}

pub(super) fn normalize_map_all_call(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> MapLikeNormalization {
    if !(1..=3).contains(&args.len()) {
        return MapLikeNormalization::Immediate(
            session.invalid_call_with_arg_range("MapAll", args, 1, 3),
        );
    }

    if args.len() == 1 {
        return MapLikeNormalization::OperatorForm(unary_operator_form("MapAll", &args[0]));
    }

    if args.len() == 2 && parse_heads_option(&args[1]).is_some() {
        return MapLikeNormalization::OperatorForm(unary_operator_form_with_tail(
            "MapAll",
            args[0].clone(),
            args[1].clone(),
        ));
    }

    let include_heads = match args {
        [_, _] => false,
        [_, _, option] => {
            let Some(include_heads) = parse_heads_option_with_default(option, false) else {
                return MapLikeNormalization::Immediate(session.invalid_call_with_arg_form(
                    "MapAll",
                    args,
                    "expected a Heads -> True/False option as the third argument",
                ));
            };
            include_heads
        }
        _ => return MapLikeNormalization::Immediate(Expr::call(head.clone(), args.to_vec())),
    };

    MapLikeNormalization::Call {
        operator: args[0].clone(),
        expr: args[1].clone(),
        config: TraversalConfig {
            levels: CasesLevelSpec::range(0, None),
            include_heads,
        },
    }
}

pub(super) fn normalize_comap_call(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> CommonArgMapNormalization {
    if !(1..=4).contains(&args.len()) {
        return CommonArgMapNormalization::Immediate(
            session.invalid_call_with_arg_range("Comap", args, 1, 4),
        );
    }

    if args.len() == 1 {
        return CommonArgMapNormalization::OperatorForm(unary_operator_form("Comap", &args[0]));
    }

    if args.len() == 2 && parse_heads_option(&args[1]).is_some() {
        return CommonArgMapNormalization::OperatorForm(unary_operator_form_with_tail(
            "Comap",
            args[0].clone(),
            args[1].clone(),
        ));
    }

    let (functions, common_expr, levels, include_heads) = match args {
        [functions, common_expr] => (
            functions.clone(),
            common_expr.clone(),
            CasesLevelSpec::exact(1),
            false,
        ),
        [functions, common_expr, third] => {
            if let Some(include_heads) = parse_heads_option(third) {
                (
                    functions.clone(),
                    common_expr.clone(),
                    CasesLevelSpec::exact(1),
                    include_heads,
                )
            } else {
                let Some(levels) = parse_standard_level_spec(third) else {
                    return CommonArgMapNormalization::Immediate(
                        session.invalid_call_with_arg_form(
                            "Comap",
                            args,
                            "expected a standard levelspec or Heads -> True/False as the third argument",
                        ),
                    );
                };
                (functions.clone(), common_expr.clone(), levels, false)
            }
        }
        [functions, common_expr, levels_expr, option] => {
            let Some(levels) = parse_standard_level_spec(levels_expr) else {
                return CommonArgMapNormalization::Immediate(session.invalid_call_with_arg_form(
                    "Comap",
                    args,
                    "expected a standard levelspec as the third argument",
                ));
            };
            let Some(include_heads) = parse_heads_option_with_default(option, false) else {
                return CommonArgMapNormalization::Immediate(session.invalid_call_with_arg_form(
                    "Comap",
                    args,
                    "expected a Heads -> True/False option as the fourth argument",
                ));
            };
            (
                functions.clone(),
                common_expr.clone(),
                levels,
                include_heads,
            )
        }
        _ => {
            return CommonArgMapNormalization::Immediate(Expr::call(head.clone(), args.to_vec()));
        }
    };

    CommonArgMapNormalization::Call {
        functions,
        common_expr,
        config: TraversalConfig {
            levels,
            include_heads,
        },
    }
}

pub(super) fn normalize_comap_apply_call(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> CommonArgMapNormalization {
    if !(1..=3).contains(&args.len()) {
        return CommonArgMapNormalization::Immediate(session.invalid_call_with_arg_range(
            "ComapApply",
            args,
            1,
            3,
        ));
    }

    if args.len() == 1 {
        return CommonArgMapNormalization::OperatorForm(unary_operator_form(
            "ComapApply",
            &args[0],
        ));
    }

    if args.len() == 2 && parse_heads_option(&args[1]).is_some() {
        return CommonArgMapNormalization::OperatorForm(unary_operator_form_with_tail(
            "ComapApply",
            args[0].clone(),
            args[1].clone(),
        ));
    }

    let include_heads = match args {
        [_, _] => false,
        [_, _, option] => {
            let Some(include_heads) = parse_heads_option_with_default(option, false) else {
                return CommonArgMapNormalization::Immediate(session.invalid_call_with_arg_form(
                    "ComapApply",
                    args,
                    "expected a Heads -> True/False option as the third argument",
                ));
            };
            include_heads
        }
        _ => {
            return CommonArgMapNormalization::Immediate(Expr::call(head.clone(), args.to_vec()));
        }
    };

    CommonArgMapNormalization::Call {
        functions: args[0].clone(),
        common_expr: args[1].clone(),
        config: TraversalConfig {
            levels: CasesLevelSpec::exact(1),
            include_heads,
        },
    }
}

pub(super) fn evaluate_scan_parts(
    session: &mut Session,
    operator: &Expr,
    parts: Vec<Expr>,
) -> Result<Expr, EvalError> {
    for part in parts {
        let result = session.evaluate(Expr::call(operator.clone(), vec![part]));
        rum_runtime::resume_or_suspend!(
            session,
            result,
            ok(value) => Ok(value),
            suspend => EvaluateScanPartsSuspensionFrame,
        )?;
    }

    Ok(system_expr("Null"))
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct EvaluateScanPartsSuspensionFrame;

impl rum_runtime::kernel_api::SuspensionFrame for EvaluateScanPartsSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        let resumed = match outcome {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
