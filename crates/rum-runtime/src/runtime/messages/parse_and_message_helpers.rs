use super::super::*;

pub(in crate::runtime) fn parse_thread_arg_spec(expr: &Expr) -> Option<ThreadArgSpec> {
    if is_system_symbol_name(expr, "All") {
        return Some(ThreadArgSpec::All);
    }

    if is_system_symbol_name(expr, "None") {
        return Some(ThreadArgSpec::None);
    }

    if let Some(count) = expr_as_signed_isize(expr) {
        return if count >= 0 {
            usize::try_from(count).ok().map(ThreadArgSpec::Prefix)
        } else {
            count
                .checked_abs()
                .and_then(|value| usize::try_from(value).ok())
                .map(ThreadArgSpec::Suffix)
        };
    }

    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") {
        return None;
    }

    match args.as_slice() {
        [index] => Some(ThreadArgSpec::Single(expr_as_signed_isize(index)?)),
        [start, end] => Some(ThreadArgSpec::Range {
            start: expr_as_signed_isize(start)?,
            end: expr_as_signed_isize(end)?,
            step: 1,
        }),
        [start, end, step] => Some(ThreadArgSpec::Range {
            start: expr_as_signed_isize(start)?,
            end: expr_as_signed_isize(end)?,
            step: expr_as_signed_isize(step)?,
        }),
        _ => None,
    }
}

pub(in crate::runtime) fn resolve_thread_arg_indices(
    spec: &ThreadArgSpec,
    arg_count: usize,
) -> Vec<usize> {
    match spec {
        ThreadArgSpec::All => (0..arg_count).collect(),
        ThreadArgSpec::None => Vec::new(),
        ThreadArgSpec::Prefix(count) => (0..(*count).min(arg_count)).collect(),
        ThreadArgSpec::Suffix(count) => {
            let start = arg_count.saturating_sub(*count);
            (start..arg_count).collect()
        }
        ThreadArgSpec::Single(index) => normalize_thread_position(*index, arg_count)
            .map(|position| vec![position])
            .unwrap_or_default(),
        ThreadArgSpec::Range { start, end, step } => {
            resolve_thread_range_indices(*start, *end, *step, arg_count)
        }
    }
}

pub(in crate::runtime) fn resolve_thread_range_indices(
    start: isize,
    end: isize,
    step: isize,
    arg_count: usize,
) -> Vec<usize> {
    if step == 0 || arg_count == 0 {
        return Vec::new();
    }

    let Some(start_index) = clamp_thread_position(start, arg_count) else {
        return Vec::new();
    };
    let Some(end_index) = clamp_thread_position(end, arg_count) else {
        return Vec::new();
    };

    let ascending = step > 0;
    let step = step.unsigned_abs();
    let mut indices = Vec::new();

    if ascending {
        if start_index > end_index {
            return indices;
        }

        let mut current = start_index;
        while current <= end_index {
            indices.push(current);
            current = current.saturating_add(step);
            if current == usize::MAX {
                break;
            }
        }
    } else {
        if start_index < end_index {
            return indices;
        }

        let mut current = start_index;
        while current >= end_index {
            indices.push(current);
            let next = current.saturating_sub(step);
            if next == current {
                break;
            }
            current = next;
        }
    }

    indices
}

pub(in crate::runtime) fn normalize_thread_position(
    position: isize,
    arg_count: usize,
) -> Option<usize> {
    if arg_count == 0 || position == 0 {
        return None;
    }

    if position > 0 {
        let zero_based = usize::try_from(position).ok()?.checked_sub(1)?;
        (zero_based < arg_count).then_some(zero_based)
    } else {
        let from_end = usize::try_from(position.checked_abs()?).ok()?;
        (from_end <= arg_count).then_some(arg_count - from_end)
    }
}

pub(in crate::runtime) fn clamp_thread_position(
    position: isize,
    arg_count: usize,
) -> Option<usize> {
    if arg_count == 0 || position == 0 {
        return None;
    }

    if position > 0 {
        let one_based = usize::try_from(position).ok()?;
        Some(one_based.clamp(1, arg_count) - 1)
    } else {
        let from_end = usize::try_from(position.checked_abs()?).ok()?;
        Some(arg_count.saturating_sub(from_end.min(arg_count)))
    }
}

pub fn parse_standard_level_spec(expr: &Expr) -> Option<CasesLevelSpec> {
    if let Some(level) = expr_as_signed_isize(expr) {
        return Some(match level.cmp(&0) {
            std::cmp::Ordering::Greater => CasesLevelSpec::range(1, Some(level)),
            std::cmp::Ordering::Equal => CasesLevelSpec::exact(0),
            std::cmp::Ordering::Less => CasesLevelSpec::range(level, Some(-1)),
        });
    }

    if is_system_symbol_name(expr, "Infinity") {
        return Some(CasesLevelSpec::range(1, None));
    }

    if is_negative_infinity_expr(expr) {
        return Some(CasesLevelSpec::exact(isize::MIN));
    }

    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(CasesLevelSpec::range(0, None));
    }

    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") {
        return None;
    }

    match args.as_slice() {
        [level] => Some(CasesLevelSpec::exact(parse_level_bound(level)?)),
        [min, max] => parse_standard_level_range(min, max),
        _ => None,
    }
}

fn parse_standard_level_range(min: &Expr, max: &Expr) -> Option<CasesLevelSpec> {
    let min = parse_level_bound(min)?;

    if is_system_symbol_name(max, "Infinity") {
        Some(CasesLevelSpec::range(min, None))
    } else if let Some(max) = parse_level_bound(max) {
        Some(CasesLevelSpec::range(min, Some(max)))
    } else {
        None
    }
}

fn parse_level_bound(expr: &Expr) -> Option<isize> {
    if let Some(level) = expr_as_signed_isize(expr) {
        return Some(level);
    }

    if is_system_symbol_name(expr, "Infinity") {
        return Some(isize::MAX);
    }

    is_negative_infinity_expr(expr).then_some(isize::MIN)
}

fn is_negative_infinity_expr(expr: &Expr) -> bool {
    let Expr::Call { head, args, .. } = expr else {
        return false;
    };

    if !is_system_symbol_name(head, heads::arithmetic::TIMES) || args.len() != 2 {
        return false;
    }

    let has_negative_one = args.iter().any(|arg| expr_as_signed_isize(arg) == Some(-1));
    let has_infinity = args
        .iter()
        .any(|arg| is_system_symbol_name(arg, "Infinity"));

    has_negative_one && has_infinity
}

pub(in crate::runtime) fn parse_non_negative_match_limit(expr: &Expr) -> Option<usize> {
    expr_as_non_negative_usize(expr)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HeadsOptionClassification {
    Value(Option<bool>),
    UnknownOption(Symbol),
    InvalidTruthValue { option: Symbol, value: Expr },
    NonOption { option_expr: Expr, offender: Expr },
    NotOptionSyntax,
}

impl ContinuationValue for HeadsOptionClassification {
    fn into_continuation_expr(self) -> Expr {
        match self {
            Self::Value(value) => Expr::call(
                system_expr("InternalContinuationHeadsOptionValue"),
                match value {
                    Some(value) => vec![value.into_continuation_expr()],
                    None => Vec::new(),
                },
            ),
            Self::UnknownOption(option) => Expr::call(
                system_expr("InternalContinuationHeadsOptionUnknown"),
                vec![Expr::Atom(Atom::Symbol(option))],
            ),
            Self::InvalidTruthValue { option, value } => Expr::call(
                system_expr("InternalContinuationHeadsOptionInvalidTruthValue"),
                vec![Expr::Atom(Atom::Symbol(option)), value],
            ),
            Self::NonOption {
                option_expr,
                offender,
            } => Expr::call(
                system_expr("InternalContinuationHeadsOptionNonOption"),
                vec![option_expr, offender],
            ),
            Self::NotOptionSyntax => Expr::call(
                system_expr("InternalContinuationHeadsOptionNotOptionSyntax"),
                Vec::new(),
            ),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args, .. }
                if is_system_symbol_name(&head, "InternalContinuationHeadsOptionValue") =>
            {
                match args.as_slice() {
                    [] => Self::Value(None),
                    [value] => Self::Value(Some(bool::from_continuation_expr(value.clone()))),
                    _ => panic!("heads option value continuation stores zero or one item"),
                }
            }
            Expr::Call { head, args, .. }
                if is_system_symbol_name(&head, "InternalContinuationHeadsOptionUnknown")
                    && args.len() == 1 =>
            {
                let mut args = Expr::unwrap_shared_args(args);
                Self::UnknownOption(symbol_from_continuation_expr(
                    args.pop().expect("unknown option stores a symbol"),
                ))
            }
            Expr::Call { head, args, .. }
                if is_system_symbol_name(
                    &head,
                    "InternalContinuationHeadsOptionInvalidTruthValue",
                ) && args.len() == 2 =>
            {
                let mut args = args.into_iter();
                let option = symbol_from_continuation_expr(
                    args.next().expect("invalid truth option stores a symbol"),
                );
                let value = args.next().expect("invalid truth option stores a value");
                Self::InvalidTruthValue { option, value }
            }
            Expr::Call { head, args, .. }
                if is_system_symbol_name(&head, "InternalContinuationHeadsOptionNonOption")
                    && args.len() == 2 =>
            {
                let mut args = args.into_iter();
                let option_expr = args
                    .next()
                    .expect("non-option stores the option expression");
                let offender = args.next().expect("non-option stores the offender");
                Self::NonOption {
                    option_expr,
                    offender,
                }
            }
            Expr::Call { head, args, .. }
                if is_system_symbol_name(
                    &head,
                    "InternalContinuationHeadsOptionNotOptionSyntax",
                ) && args.is_empty() =>
            {
                Self::NotOptionSyntax
            }
            other => panic!("unexpected heads option continuation payload: {other:?}"),
        }
    }
}

pub(in crate::runtime) fn heads_option_classification_looks_like_option_syntax(
    classification: &HeadsOptionClassification,
) -> bool {
    match classification {
        HeadsOptionClassification::Value(_)
        | HeadsOptionClassification::UnknownOption(_)
        | HeadsOptionClassification::InvalidTruthValue { .. } => true,
        HeadsOptionClassification::NonOption { option_expr, .. } => {
            heads_option_expr_contains_rule(option_expr)
        }
        HeadsOptionClassification::NotOptionSyntax => false,
    }
}

pub fn parse_heads_option(expr: &Expr) -> Option<bool> {
    match classify_heads_option_expr(expr) {
        HeadsOptionClassification::Value(value) => value,
        _ => None,
    }
}

/// Parses a Heads option expression, using `default` for bare `Heads`.
pub fn parse_heads_option_with_default(expr: &Expr, default: bool) -> Option<bool> {
    match classify_heads_option_expr(expr) {
        HeadsOptionClassification::Value(value) => Some(value.unwrap_or(default)),
        _ => None,
    }
}

/// Classifies a Heads option expression for builtin diagnostics.
pub fn classify_heads_option_expr(expr: &Expr) -> HeadsOptionClassification {
    if let Some(classification) = classify_heads_option_rule(expr) {
        return classification;
    }

    let Expr::Call { head, args, .. } = expr else {
        return HeadsOptionClassification::NotOptionSyntax;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return HeadsOptionClassification::NotOptionSyntax;
    }

    let mut value = None;
    for arg in args {
        match classify_heads_option_expr(arg) {
            HeadsOptionClassification::Value(parsed) => {
                if value.is_none() {
                    value = parsed;
                }
            }
            HeadsOptionClassification::InvalidTruthValue { option, value: _ }
                if value.is_some()
                    && option.context() == SYSTEM_CONTEXT
                    && option.name() == "Heads" => {}
            HeadsOptionClassification::InvalidTruthValue { option, value } => {
                return HeadsOptionClassification::InvalidTruthValue { option, value };
            }
            HeadsOptionClassification::UnknownOption(option) => {
                return HeadsOptionClassification::UnknownOption(option);
            }
            HeadsOptionClassification::NonOption {
                option_expr,
                offender,
            } => {
                return HeadsOptionClassification::NonOption {
                    option_expr,
                    offender,
                };
            }
            HeadsOptionClassification::NotOptionSyntax => {
                return HeadsOptionClassification::NonOption {
                    option_expr: expr.clone(),
                    offender: arg.clone(),
                };
            }
        }
    }
    HeadsOptionClassification::Value(value)
}

fn classify_heads_option_rule(expr: &Expr) -> Option<HeadsOptionClassification> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    let symbol = heads_option_symbol(&args[0])?;

    if symbol.context() != SYSTEM_CONTEXT || symbol.name() != "Heads" {
        return Some(HeadsOptionClassification::UnknownOption(symbol));
    }

    match truthiness_symbol(&args[1]) {
        Some(value) => Some(HeadsOptionClassification::Value(Some(value))),
        None => Some(HeadsOptionClassification::InvalidTruthValue {
            option: symbol,
            value: args[1].clone(),
        }),
    }
}

fn heads_option_symbol(expr: &Expr) -> Option<Symbol> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.clone()),
        Expr::Atom(Atom::String(name)) => resolve_unqualified_symbol(name),
        _ => None,
    }
}

fn heads_option_expr_contains_rule(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, args, .. }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            heads_option_symbol(&args[0]).is_some()
        }
        Expr::Call { head, args, .. } if is_system_symbol_name(head, heads::structure::LIST) => {
            args.iter().any(heads_option_expr_contains_rule)
        }
        _ => false,
    }
}

fn symbol_from_continuation_expr(expr: Expr) -> Symbol {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => symbol,
        other => panic!("expected symbol continuation payload, got {other:?}"),
    }
}

impl Session {
    pub(in crate::runtime) fn invalid_level_spec_call(
        &mut self,
        symbol: &str,
        args: &[Expr],
        spec: &Expr,
    ) -> Expr {
        self.push_message(
            symbol,
            "level",
            format!("Level specification {spec} is not of the form n, {{n}} or {{m, n}}."),
        );
        self.invalid_call(symbol, args)
    }

    pub(in crate::runtime) fn invalid_non_negative_or_infinity_call(
        &mut self,
        symbol: &str,
        args: &[Expr],
        position: usize,
    ) -> Expr {
        let call = Expr::call(system_expr(symbol), args.to_vec());
        self.push_message(
            symbol,
            "innf",
            format!("Non-negative integer or Infinity expected at position {position} in {call}."),
        );
        self.invalid_call(symbol, args)
    }

    /// Emits the standard diagnostic for an invalid Heads option.
    pub fn invalid_heads_option_call(
        &mut self,
        symbol: &str,
        args: &[Expr],
        option_expr: &Expr,
        positional_count: usize,
        classification: HeadsOptionClassification,
        fallback: impl Into<String>,
    ) -> Expr {
        let call = Expr::call(system_expr(symbol), args.to_vec());
        match classification {
            HeadsOptionClassification::UnknownOption(option) => {
                self.push_message(
                    symbol,
                    "optx",
                    format!("Unknown option {option} in {call}."),
                );
            }
            HeadsOptionClassification::InvalidTruthValue { option, value } => {
                self.push_message(
                    symbol,
                    "opttf",
                    format!("Value of option {option} -> {value} should be True or False."),
                );
            }
            HeadsOptionClassification::NonOption {
                option_expr,
                offender: _,
            } => {
                self.push_message(
                    symbol,
                    "nonopt",
                    format!(
                        "Options expected (instead of {option_expr}) beyond position {positional_count} in {call}. An option must be a rule or a list of rules."
                    ),
                );
            }
            HeadsOptionClassification::NotOptionSyntax => {
                self.push_message(
                    symbol,
                    "nonopt",
                    format!(
                        "Options expected (instead of {option_expr}) beyond position {positional_count} in {call}. An option must be a rule or a list of rules."
                    ),
                );
            }
            HeadsOptionClassification::Value(_) => {
                self.push_message(symbol, "argform", fallback);
            }
        }
        self.invalid_call(symbol, args)
    }
}
