use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

mod down_value_dispatch;
mod tagged_value_store;

use down_value_dispatch::DownValueDispatchMetadata;
pub(in crate::runtime) use down_value_dispatch::{DownValueScan, LinearDownValueScan};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolInfo {
    pub qualified_name: String,
    pub context: String,
    pub display_name: String,
    pub summary: Option<String>,
    pub is_builtin: bool,
    pub attribute_count: usize,
    pub attributes: Vec<String>,
    pub has_own_value: bool,
    pub has_delayed_value: bool,
    pub has_up_values: bool,
    pub has_down_values: bool,
    pub has_sub_values: bool,
    pub message_count: usize,
    pub message_tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinitionCatalogEntry {
    pub kind: DefinitionKind,
    pub lhs: Expr,
    pub rhs: Expr,
    pub delayed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteRuleInfo {
    pub lhs: Expr,
    pub rhs: Expr,
    pub delayed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageCatalogEntry {
    pub message_name: String,
    pub qualified_symbol_name: String,
    pub tag: String,
    pub template: Expr,
    pub delayed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, FrameRootTrace)]
pub struct SymbolDefinition {
    pub own_value: Option<Expr>,
    pub delayed_own_value: Option<Expr>,
    pub(in crate::runtime) own_value_guards: Option<Box<[Expr]>>,
    pub up_values: Vec<UpValue>,
    pub down_values: Vec<DownValue>,
    down_value_dispatch: Option<Box<DownValueDispatchMetadata>>,
    pub sub_values: Vec<SubValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub struct UpValue {
    pub lhs: Expr,
    pub rhs: Expr,
    pub delayed: bool,
    pub(in crate::runtime) rhs_may_splice_without_bindings: bool,
    pub(in crate::runtime) matcher: MatcherNode,
    pub(in crate::runtime) matcher_attribute_revision: u64,
    pub guards: Box<[Expr]>,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub struct DownValue {
    pub lhs: Expr,
    pub rhs: Expr,
    pub delayed: bool,
    pub(in crate::runtime) lhs_is_ground: bool,
    pub(in crate::runtime) rhs_may_splice_without_bindings: bool,
    pub(in crate::runtime) matcher: MatcherNode,
    pub(in crate::runtime) matcher_attribute_revision: u64,
    pub guards: Box<[Expr]>,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub struct SubValue {
    pub lhs: Expr,
    pub rhs: Expr,
    pub delayed: bool,
    pub(in crate::runtime) rhs_may_splice_without_bindings: bool,
    pub(in crate::runtime) matcher: MatcherNode,
    pub(in crate::runtime) matcher_attribute_revision: u64,
    pub guards: Box<[Expr]>,
}

impl SymbolDefinition {
    pub(in crate::runtime) fn with_own_value(value: Expr) -> Self {
        Self {
            own_value: Some(value),
            ..Self::default()
        }
    }

    pub(in crate::runtime) fn store_own_value(
        &mut self,
        value: Expr,
        guards: Box<[Expr]>,
        delayed: bool,
    ) {
        if delayed {
            self.own_value = None;
            self.delayed_own_value = Some(value);
        } else {
            self.own_value = Some(value);
            self.delayed_own_value = None;
        }
        self.own_value_guards = (!guards.is_empty()).then_some(guards);
    }

    pub(in crate::runtime) fn clear_own_values(&mut self) -> bool {
        let changed = self.own_value.take().is_some() | self.delayed_own_value.take().is_some();
        self.own_value_guards = None;
        changed
    }

    pub(in crate::runtime) fn own_value_guards(&self) -> &[Expr] {
        self.own_value_guards.as_deref().unwrap_or_default()
    }

    pub(in crate::runtime) fn is_empty(&self) -> bool {
        self.own_value.is_none()
            && self.delayed_own_value.is_none()
            && self.up_values.is_empty()
            && self.down_values.is_empty()
            && self.sub_values.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct DownValueShallowSignature {
    args: Vec<DownValueShallowArg>,
}

impl DownValueShallowSignature {
    fn from_lhs(symbol: &Symbol, lhs: &Expr) -> Option<Self> {
        let lhs = strip_hold_pattern(lhs);
        let Expr::Call { head, args, .. } = lhs else {
            return None;
        };
        if expr_as_symbol(head).is_none_or(|head_symbol| head_symbol != symbol) {
            return None;
        }
        if shallow_pattern_args_may_have_variable_arity(args) {
            return None;
        }

        args.iter()
            .map(DownValueShallowArg::from_pattern_arg)
            .collect::<Option<Vec<_>>>()
            .map(|args| Self { args })
    }

    pub(in crate::runtime) fn could_match_args(&self, args: &[Expr]) -> bool {
        self.args.len() == args.len()
            && self
                .args
                .iter()
                .zip(args)
                .all(|(pattern, value)| pattern.could_match(value))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
enum DownValueShallowArg {
    Any,
    Exact(Expr),
    Head(Symbol),
}

impl DownValueShallowArg {
    fn from_pattern_arg(pattern: &Expr) -> Option<Self> {
        let pattern = strip_hold_pattern(pattern);

        if let Some(inner) = pattern_verbatim_inner(pattern) {
            return Some(Self::Exact(inner.clone()));
        }

        if let Some((_, inner)) = pattern_named_pattern(pattern) {
            return Self::from_pattern_arg(inner);
        }

        if pattern_condition(pattern).is_some()
            || pattern_test(pattern).is_some()
            || pattern_except(pattern).is_some()
            || pattern_optional(pattern).is_some()
            || pattern_alternatives(pattern).is_some()
            || pattern_key_value_pattern_entries(pattern).is_some()
            || decompose_sequence_pattern(pattern).is_some()
            || pattern_sequence_items(pattern).is_some()
            || pattern_orderless_sequence_items(pattern).is_some()
        {
            return None;
        }

        if let Some(head_constraint) = pattern_blank_head(pattern) {
            return match head_constraint {
                None => Some(Self::Any),
                Some(head) => expr_as_symbol(head).map(|symbol| Self::Head(*symbol)),
            };
        }

        if expr_contains_pattern_syntax(pattern) {
            return None;
        }

        Some(Self::Exact(pattern.clone()))
    }

    fn could_match(&self, value: &Expr) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(pattern) => pattern == value,
            Self::Head(symbol) => expr_has_symbol_head(value, symbol),
        }
    }
}

pub(in crate::runtime) fn expr_contains_pattern_syntax(expr: &Expr) -> bool {
    if pattern_verbatim_inner(expr).is_some()
        || pattern_named_pattern(expr).is_some()
        || pattern_condition(expr).is_some()
        || pattern_test(expr).is_some()
        || pattern_except(expr).is_some()
        || pattern_optional(expr).is_some()
        || pattern_alternatives(expr).is_some()
        || pattern_key_value_pattern_entries(expr).is_some()
        || pattern_blank_head(expr).is_some()
        || decompose_sequence_pattern(expr).is_some()
        || pattern_sequence_items(expr).is_some()
        || pattern_orderless_sequence_items(expr).is_some()
    {
        return true;
    }

    match expr {
        Expr::Atom(_) => false,
        Expr::Call { head, args, .. } => {
            expr_contains_pattern_syntax(head) || args.iter().any(expr_contains_pattern_syntax)
        }
    }
}

fn shallow_pattern_args_may_have_variable_arity(args: &[Expr]) -> bool {
    args.iter().any(|arg| {
        let arg = strip_hold_pattern(arg);
        decompose_sequence_pattern(arg).is_some()
            || pattern_sequence_items(arg).is_some()
            || pattern_orderless_sequence_items(arg).is_some()
            || pattern_optional(arg).is_some()
    })
}

fn expr_has_symbol_head(value: &Expr, expected_symbol: &Symbol) -> bool {
    match value {
        Expr::Atom(Atom::Integer(_)) => expected_symbol.name() == "Integer",
        Expr::Atom(Atom::Rational(_)) => expected_symbol.name() == "Rational",
        Expr::Atom(Atom::String(_)) => expected_symbol.name() == "String",
        Expr::Atom(Atom::Symbol(_)) => expected_symbol.name() == "Symbol",
        Expr::Call { head, .. } => expr_as_symbol(head)
            .map(|actual| actual == expected_symbol)
            .unwrap_or(false),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub struct RewriteRule {
    pub(in crate::runtime) lhs: Expr,
    pub(in crate::runtime) rhs: Expr,
    pub(in crate::runtime) delayed: bool,
    pub(in crate::runtime) rhs_may_splice_without_bindings: bool,
    pub(in crate::runtime) matcher: MatcherNode,
    pub(in crate::runtime) guards: Box<[Expr]>,
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<DownValue>();
    assert_traces::<DownValueShallowArg>();
    assert_traces::<DownValueShallowSignature>();
    assert_traces::<RewriteRule>();
    assert_traces::<SubValue>();
    assert_traces::<SymbolDefinition>();
    assert_traces::<UpValue>();
};
