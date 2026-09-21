use super::*;
use crate::runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) struct CallPreparationMetadata {
    attributes: AttributeSet,
    head_symbol: Option<Symbol>,
    definition_symbol: Option<Symbol>,
}

impl CallPreparationMetadata {
    pub(super) fn attributes(self) -> AttributeSet {
        self.attributes
    }

    pub(super) fn head_symbol(self) -> Option<Symbol> {
        self.head_symbol
    }

    pub(super) fn definition_symbol(self) -> Option<Symbol> {
        self.definition_symbol
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) struct CallArgumentPlan {
    pub(super) expr: Expr,
    pub(super) mode: CallArgumentMode,
    pub(super) allow_sequence_splice: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) enum CallArgumentMode {
    Evaluate,
    Hold,
    StrippedUnevaluated,
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct PreparedArgumentAccumulator {
    prefix: SharedArgs,
    tail: AccumulatorChain<PreparedArgumentNode>,
    pub(super) len: usize,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct PreparedArgumentNode {
    #[accumulator_link]
    previous: AccumulatorChain<PreparedArgumentNode>,
    value: Expr,
}

impl PreparedArgumentAccumulator {
    pub(super) fn freeze(values: Vec<Expr>) -> Self {
        let len = values.len();
        Self {
            prefix: SharedArgs::new(values),
            tail: AccumulatorChain::default(),
            len,
        }
    }

    fn appended(&self, value: Expr) -> Self {
        Self {
            prefix: self.prefix.clone(),
            tail: AccumulatorChain::new(Some(Arc::new(PreparedArgumentNode {
                previous: self.tail.clone(),
                value,
            }))),
            len: self.len + 1,
        }
    }

    pub(super) fn into_vec(self) -> Vec<Expr> {
        let mut suffix = Vec::with_capacity(self.len.saturating_sub(self.prefix.len()));
        let mut node = self.tail.into_inner();
        while let Some(current) = node {
            let current = Arc::try_unwrap(current)
                .unwrap_or_else(|_| panic!("completed call argument state must be unique"));
            suffix.push(current.value);
            node = current.previous.into_inner();
        }
        suffix.reverse();

        let mut values = self.prefix.into_vec();
        values.reserve(suffix.len());
        values.extend(suffix);
        values
    }

    pub(super) fn into_vec_allow_shared(self) -> Vec<Expr> {
        let mut suffix = Vec::with_capacity(self.len.saturating_sub(self.prefix.len()));
        let mut node = self.tail.into_inner();
        while let Some(current) = node {
            match Arc::try_unwrap(current) {
                Ok(current) => {
                    suffix.push(current.value);
                    node = current.previous.into_inner();
                }
                Err(current) => {
                    suffix.push(current.value.clone());
                    node = current.previous.clone().into_inner();
                }
            }
        }
        suffix.reverse();

        let mut values = self.prefix.into_vec();
        values.reserve(suffix.len());
        values.extend(suffix);
        values
    }
}

pub(super) fn plan_call_argument_preparation(
    attributes: AttributeSet,
    index: usize,
    arg: Expr,
) -> CallArgumentPlan {
    let hold = attributes.hold_attribute();
    let blocks_evaluate_override = attributes.blocks_evaluate_override();

    if hold.holds_argument(index) {
        if blocks_evaluate_override {
            return CallArgumentPlan {
                expr: arg,
                mode: CallArgumentMode::Hold,
                allow_sequence_splice: false,
            };
        }

        if let Some(inner) = unevaluated_wrapper_arg(&arg) {
            return CallArgumentPlan {
                expr: inner.clone(),
                mode: CallArgumentMode::StrippedUnevaluated,
                allow_sequence_splice: true,
            };
        }

        if let Some(inner) = evaluate_wrapper_arg(&arg) {
            return CallArgumentPlan {
                expr: inner.clone(),
                mode: CallArgumentMode::Evaluate,
                allow_sequence_splice: true,
            };
        }

        return CallArgumentPlan {
            expr: arg,
            mode: CallArgumentMode::Hold,
            allow_sequence_splice: true,
        };
    }

    if let Some(inner) = unevaluated_wrapper_arg(&arg) {
        return CallArgumentPlan {
            expr: inner.clone(),
            mode: CallArgumentMode::StrippedUnevaluated,
            allow_sequence_splice: true,
        };
    }

    CallArgumentPlan {
        expr: arg,
        mode: CallArgumentMode::Evaluate,
        allow_sequence_splice: true,
    }
}

pub(super) fn append_prepared_argument(
    prepared_args: &mut Vec<Expr>,
    value: Expr,
    allow_sequence_splice: bool,
    sequence_hold: bool,
) {
    if allow_sequence_splice && !sequence_hold {
        splice_sequence_argument(prepared_args, value);
    } else {
        prepared_args.push(value);
    }
}

pub(super) fn append_prepared_list_argument(
    prepared_args: &mut Vec<Expr>,
    value: Expr,
    allow_sequence_splice: bool,
    sequence_hold: bool,
) {
    if allow_sequence_splice && !sequence_hold {
        match value {
            Expr::Call { head, args }
                if is_system_symbol_name(head.as_ref(), heads::functions::SEQUENCE) =>
            {
                prepared_args.extend(
                    args.into_iter()
                        .filter(|item| !is_system_symbol_name(item, "Nothing")),
                );
            }
            other => append_prepared_list_item(prepared_args, other),
        }
    } else {
        append_prepared_list_item(prepared_args, value);
    }
}

fn append_prepared_list_item(prepared_args: &mut Vec<Expr>, value: Expr) {
    if !is_system_symbol_name(&value, "Nothing") {
        prepared_args.push(value);
    }
}

pub(super) fn append_suspended_prepared_argument(
    mut prepared: PreparedArgumentAccumulator,
    value: Expr,
    allow_sequence_splice: bool,
    sequence_hold: bool,
    list_drops_nothing: bool,
) -> PreparedArgumentAccumulator {
    if allow_sequence_splice && !sequence_hold {
        if let Some(items) = sequence_items(&value) {
            for item in items {
                if !list_drops_nothing || !is_nothing_symbol(item) {
                    prepared = prepared.appended(item.clone());
                }
            }
            return prepared;
        }
    }

    if !list_drops_nothing || !is_nothing_symbol(&value) {
        prepared = prepared.appended(value);
    }
    prepared
}

pub(super) fn prepared_argument_splices(
    value: &Expr,
    allow_sequence_splice: bool,
    sequence_hold: bool,
) -> bool {
    allow_sequence_splice && !sequence_hold && sequence_items(value).is_some()
}

pub(super) fn prepared_list_argument_drops_nothing(
    value: &Expr,
    allow_sequence_splice: bool,
    sequence_hold: bool,
) -> bool {
    if is_nothing_symbol(value) {
        return true;
    }

    allow_sequence_splice
        && !sequence_hold
        && sequence_items(value).is_some_and(|items| items.iter().any(is_nothing_symbol))
}

fn list_args_drop_nothing(head: &Expr, args: &[Expr]) -> bool {
    is_system_symbol_name(head, heads::structure::LIST) && args.iter().any(is_nothing_symbol)
}

fn is_nothing_symbol(expr: &Expr) -> bool {
    is_system_symbol_name(expr, "Nothing")
}

fn attribute_normalization_is_noop(head: &Expr, args: &[Expr], attributes: AttributeSet) -> bool {
    if list_args_drop_nothing(head, args) {
        return false;
    }

    if attributes.contains(Attribute::Flat)
        && args
            .iter()
            .any(|arg| call_items_with_head(arg, head).is_some())
    {
        return false;
    }

    if attributes.contains(Attribute::Orderless)
        && args
            .windows(2)
            .any(|pair| compare_expr_canonical(&pair[0], &pair[1]).is_gt())
    {
        return false;
    }

    true
}

impl Session {
    pub(super) fn call_preparation_metadata(&mut self, head: &Expr) -> CallPreparationMetadata {
        let mut attributes = self.cached_attribute_set_for_head(head);
        let head_symbol = expr_as_symbol(head).copied();
        let definition_symbol = if let Some(symbol) = head_symbol {
            Some(symbol)
        } else {
            let base_symbol = base_head_symbol_of_nested_call(head).copied();
            if let Some(symbol) = base_symbol {
                if self
                    .cached_attribute_set_for_symbol(&symbol)
                    .contains(Attribute::SubValuesHoldAll)
                {
                    attributes = attributes.with(Attribute::HoldAll);
                }
            }
            base_symbol
        };

        CallPreparationMetadata {
            attributes,
            head_symbol,
            definition_symbol,
        }
    }

    pub(super) fn attribute_set_for_head(&self, head: &Expr) -> AttributeSet {
        if let Some(function) = parse_function(head) {
            return function.attributes();
        }

        let Some(symbol) = expr_as_symbol(head) else {
            return AttributeSet::empty();
        };

        self.attribute_set_for_symbol(symbol)
    }

    pub(super) fn cached_attribute_set_for_head(&mut self, head: &Expr) -> AttributeSet {
        if let Some(function) = parse_function(head) {
            return function.attributes();
        }

        let Some(symbol) = expr_as_symbol(head) else {
            return AttributeSet::empty();
        };

        self.cached_attribute_set_for_symbol(symbol)
    }

    pub(super) fn normalize_call_args_by_attributes(
        &self,
        head: &Expr,
        args: Vec<Expr>,
        attributes: AttributeSet,
    ) -> Vec<Expr> {
        let runtime_profile_total = runtime_path_timing_start("prepare_arguments.normalize.total");
        let mut normalized_args = if attributes.contains(Attribute::Flat) {
            let runtime_profile_flat =
                runtime_path_timing_start("prepare_arguments.normalize.flat");
            let result = flatten_flat_call_args(head, args);
            record_profile_runtime_path_timing(runtime_profile_flat, RuntimeProfileOutcome::Ok);
            result
        } else {
            args
        };

        if attributes.contains(Attribute::Orderless) {
            let runtime_profile_orderless =
                runtime_path_timing_start("prepare_arguments.normalize.orderless");
            normalized_args.sort_by(compare_expr_canonical);
            record_profile_runtime_path_timing(
                runtime_profile_orderless,
                RuntimeProfileOutcome::Ok,
            );
        }

        if is_system_symbol_name(head, heads::structure::LIST) {
            normalized_args.retain(|arg| !is_nothing_symbol(arg));
        }

        record_profile_runtime_path_timing(runtime_profile_total, RuntimeProfileOutcome::Ok);
        normalized_args
    }

    pub(super) fn normalize_call_args_shared(
        &self,
        head: &Expr,
        args: SharedArgs,
        attributes: AttributeSet,
    ) -> SharedArgs {
        if attribute_normalization_is_noop(head, args.as_ref(), attributes) {
            return args;
        }

        SharedArgs::new(self.normalize_call_args_by_attributes(
            head,
            Expr::unwrap_shared_args(args),
            attributes,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planner_keeps_held_arguments_verbatim_under_holdallcomplete() {
        let arg = Expr::call(system_expr("Evaluate"), vec![Expr::integer(1_i64.into())]);
        let attributes = AttributeSet::from_attribute(Attribute::HoldAllComplete);
        let plan = plan_call_argument_preparation(attributes, 0, arg.clone());

        assert_eq!(
            plan,
            CallArgumentPlan {
                expr: arg,
                mode: CallArgumentMode::Hold,
                allow_sequence_splice: false,
            }
        );
    }

    #[test]
    fn planner_respects_evaluate_wrapper_for_held_arguments() {
        let inner = Expr::Atom(Atom::Symbol(Symbol::from_name("x")));
        let arg = Expr::call(system_expr("Evaluate"), vec![inner.clone()]);
        let attributes = AttributeSet::from_attribute(Attribute::HoldAll);
        let plan = plan_call_argument_preparation(attributes, 0, arg);

        assert_eq!(
            plan,
            CallArgumentPlan {
                expr: inner,
                mode: CallArgumentMode::Evaluate,
                allow_sequence_splice: true,
            }
        );
    }

    #[test]
    fn planner_allows_raw_sequence_splicing_for_held_arguments() {
        let arg = Expr::call(
            system_expr("Sequence"),
            vec![Expr::integer(1_i64.into()), Expr::integer(2_i64.into())],
        );
        let attributes = AttributeSet::from_attribute(Attribute::HoldAll);
        let plan = plan_call_argument_preparation(attributes, 0, arg.clone());

        assert_eq!(
            plan,
            CallArgumentPlan {
                expr: arg,
                mode: CallArgumentMode::Hold,
                allow_sequence_splice: true,
            }
        );
    }

    #[test]
    fn planner_respects_unevaluated_wrapper_for_nonheld_arguments() {
        let inner = Expr::Atom(Atom::Symbol(Symbol::from_name("x")));
        let arg = Expr::call(system_expr("Unevaluated"), vec![inner.clone()]);
        let plan = plan_call_argument_preparation(AttributeSet::empty(), 0, arg);

        assert_eq!(
            plan,
            CallArgumentPlan {
                expr: inner,
                mode: CallArgumentMode::StrippedUnevaluated,
                allow_sequence_splice: true,
            }
        );
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<CallArgumentMode>();
    assert_traces::<CallArgumentPlan>();
    assert_traces::<CallPreparationMetadata>();
    assert_traces::<PreparedArgumentAccumulator>();
};
