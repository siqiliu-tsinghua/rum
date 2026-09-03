use super::*;
use std::collections::HashSet;

impl Session {
    pub(in crate::runtime) fn call_stability_uncached_with_bloom(
        &mut self,
        head: &SharedExpr,
        args: &SharedArgs,
    ) -> (bool, StabilityBloom) {
        let (head_stable, mut bloom) = self.cached_expr_stability_with_bloom(head);
        if !head_stable {
            return (false, bloom);
        }

        if let Some((stable, data_bloom)) =
            self.data_constructor_stability_with_bloom(head, args, bloom)
        {
            return (stable, data_bloom);
        }

        if self.call_head_has_evaluation_effect(head, args) {
            return (false, bloom);
        }

        for arg in args.iter() {
            let (arg_stable, arg_bloom) = self.cached_expr_stability_with_bloom(arg);
            bloom.union_assign(arg_bloom);
            if !arg_stable {
                return (false, bloom);
            }
        }

        (true, bloom)
    }

    fn data_constructor_stability_with_bloom(
        &mut self,
        head: &Expr,
        args: &[Expr],
        mut bloom: StabilityBloom,
    ) -> Option<(bool, StabilityBloom)> {
        let symbol = expr_as_symbol(head)?;
        if symbol.context() != SYSTEM_CONTEXT {
            return None;
        }

        match symbol.name() {
            "Association" => Some(self.association_data_stability_with_bloom(args, bloom)),
            "Failure" => Some(self.failure_data_stability_with_bloom(args, bloom)),
            "Missing" | "Success" | "TerminatedEvaluation" => {
                Some(self.all_args_stability_with_bloom(args, bloom))
            }
            "HoldAllComplete" | "HoldComplete" => Some((true, bloom)),
            "Function" | "Hold" | "HoldAll" | "HoldPattern" => {
                Some((hold_all_literal_args_are_stable(args), bloom))
            }
            "Rule" => {
                let [lhs, rhs] = args else {
                    return Some((false, bloom));
                };
                let (lhs_stable, lhs_bloom) = self.cached_expr_stability_with_bloom(lhs);
                bloom.union_assign(lhs_bloom);
                if !lhs_stable {
                    return Some((false, bloom));
                }
                let (rhs_stable, rhs_bloom) = self.cached_expr_stability_with_bloom(rhs);
                bloom.union_assign(rhs_bloom);
                Some((rhs_stable, bloom))
            }
            "RuleDelayed" => {
                let [lhs, _rhs] = args else {
                    return Some((false, bloom));
                };
                let (lhs_stable, lhs_bloom) = self.cached_expr_stability_with_bloom(lhs);
                bloom.union_assign(lhs_bloom);
                Some((lhs_stable, bloom))
            }
            _ => None,
        }
    }

    fn all_args_stability_with_bloom(
        &mut self,
        args: &[Expr],
        mut bloom: StabilityBloom,
    ) -> (bool, StabilityBloom) {
        for arg in args {
            let (arg_stable, arg_bloom) = self.cached_expr_stability_with_bloom(arg);
            bloom.union_assign(arg_bloom);
            if !arg_stable {
                return (false, bloom);
            }
        }

        (true, bloom)
    }

    fn failure_data_stability_with_bloom(
        &mut self,
        args: &[Expr],
        bloom: StabilityBloom,
    ) -> (bool, StabilityBloom) {
        let [_, details] = args else {
            return self.all_args_stability_with_bloom(args, bloom);
        };
        if failure_details_would_normalize(details) {
            return (false, bloom);
        }

        self.all_args_stability_with_bloom(args, bloom)
    }

    fn association_data_stability_with_bloom(
        &mut self,
        args: &[Expr],
        mut bloom: StabilityBloom,
    ) -> (bool, StabilityBloom) {
        let mut linear_keys: Vec<Expr> =
            Vec::with_capacity(args.len().min(ASSOCIATION_INDEX_THRESHOLD + 1));
        let mut indexed_keys: Option<HashSet<Expr>> = None;
        for arg in args {
            let Some(entry) = parse_association_entry_form(arg) else {
                return (false, bloom);
            };

            if let Some(keys) = indexed_keys.as_mut() {
                if !keys.insert(entry.key) {
                    return (false, bloom);
                }
            } else {
                if linear_keys.iter().any(|key| key == &entry.key) {
                    return (false, bloom);
                }
                linear_keys.push(entry.key);
                if linear_keys.len() > ASSOCIATION_INDEX_THRESHOLD {
                    let mut keys = HashSet::with_capacity(args.len());
                    keys.extend(linear_keys.drain(..));
                    indexed_keys = Some(keys);
                }
            }

            let (arg_stable, arg_bloom) = self.cached_expr_stability_with_bloom(arg);
            bloom.union_assign(arg_bloom);
            if !arg_stable {
                return (false, bloom);
            }
        }

        (true, bloom)
    }

    pub(in crate::runtime) fn symbol_is_atom_evaluation_stable(&mut self, symbol: &Symbol) -> bool {
        if self.has_dynamic_own_value_binding(symbol) {
            return false;
        }

        if let Some(entry) = self.stable_symbol_cache.get(symbol) {
            if entry.generation == self.symbol_cache_generation {
                return entry.stable;
            }
        }

        let stable = self.symbol_is_atom_evaluation_stable_uncached(symbol);
        self.stable_symbol_cache.insert(
            *symbol,
            SymbolStabilityCacheEntry {
                generation: self.symbol_cache_generation,
                stable,
            },
        );
        stable
    }

    fn symbol_is_atom_evaluation_stable_uncached(&self, symbol: &Symbol) -> bool {
        if symbol.context() == SYSTEM_CONTEXT
            && matches!(
                symbol.name(),
                "$Context"
                    | "$ContextPath"
                    | "$Packages"
                    | "$InputFileName"
                    | "$Path"
                    | "$MessageList"
            )
        {
            return false;
        }

        if self.declared_stubs.contains(symbol)
            && !self
                .loaded_packages
                .iter()
                .any(|loaded| loaded == symbol.context())
        {
            return false;
        }

        if self.removed_symbols.contains(symbol) {
            return false;
        }

        if self.has_dynamic_own_value_binding(symbol) {
            return false;
        }

        self.definitions.get(symbol).is_none_or(|definition| {
            definition.own_value.is_none()
                && definition.delayed_own_value.is_none()
                && definition.up_values.is_empty()
        })
    }

    fn call_head_has_evaluation_effect(&mut self, head: &Expr, args: &[Expr]) -> bool {
        if let Some(symbol) = expr_as_symbol(head) {
            if symbol.is_system_name(heads::structure::LIST)
                && args.iter().any(|arg| is_system_symbol_name(arg, "Nothing"))
            {
                return true;
            }

            if self.symbol_call_args_have_no_evaluation_effect(symbol, args) {
                return false;
            }
            return self.symbol_call_head_has_evaluation_effect(symbol, args.is_empty());
        }

        let attributes = self.cached_attribute_set_for_head(head);
        if attributes.contains(Attribute::Listable)
            || attributes.contains(Attribute::Orderless)
            || attributes.contains(Attribute::Flat)
        {
            return true;
        }

        if parse_function(head).is_some()
            || parse_composition(head).is_some()
            || is_failure_expr(head)
            || is_success_expr(head)
            || has_association_head(head)
            || is_callable_operator_wrapper(head)
            || is_callable_failsafe_wrapper(head)
        {
            return true;
        }

        true
    }

    fn symbol_call_args_have_no_evaluation_effect(
        &mut self,
        symbol: &Symbol,
        args: &[Expr],
    ) -> bool {
        if self.has_up_value_definitions
            || symbol.context() == SYSTEM_CONTEXT && symbol.name() == heads::patterns::DEFAULT
        {
            return false;
        }

        let attributes = self.cached_attribute_set_for_symbol(symbol);
        if attributes.contains(Attribute::Listable)
            || attributes.contains(Attribute::Orderless)
            || attributes.contains(Attribute::Flat)
            || !matches!(attributes.hold_attribute(), HoldAttribute::None)
        {
            return false;
        }

        if self
            .cached_builtin_dispatch_spec_for_symbol(symbol)
            .is_some()
        {
            return false;
        }

        let Some(definition) = self.definitions.get(symbol) else {
            return false;
        };
        if !definition.up_values.is_empty()
            || !definition.sub_values.is_empty()
            || definition.down_values.is_empty()
        {
            return false;
        }

        definition
            .down_value_shallow_signatures()
            .is_some_and(|signatures| {
                signatures
                    .iter()
                    .all(|signature| !signature.could_match_args(args))
            })
    }

    fn symbol_call_head_has_evaluation_effect(
        &mut self,
        symbol: &Symbol,
        args_empty: bool,
    ) -> bool {
        let key = SymbolCallEffectKey {
            symbol: symbol.clone(),
            args_empty,
        };
        if let Some(entry) = self.symbol_call_effect_cache.get(&key) {
            if entry.generation == self.symbol_cache_generation {
                return entry.has_effect;
            }
        }

        let has_effect = self.symbol_call_head_has_evaluation_effect_uncached(symbol, args_empty);
        self.symbol_call_effect_cache.insert(
            key,
            CallEffectCacheEntry {
                generation: self.symbol_cache_generation,
                has_effect,
            },
        );
        has_effect
    }

    fn symbol_call_head_has_evaluation_effect_uncached(
        &mut self,
        symbol: &Symbol,
        args_empty: bool,
    ) -> bool {
        let attributes = self.cached_attribute_set_for_symbol(symbol);
        if attributes.contains(Attribute::Listable)
            || attributes.contains(Attribute::Orderless)
            || attributes.contains(Attribute::Flat)
        {
            return true;
        }

        if self
            .cached_builtin_dispatch_spec_for_symbol(symbol)
            .is_some()
        {
            return true;
        }

        self.definitions.get(symbol).is_some_and(|definition| {
            !definition.down_values.is_empty()
                || !definition.sub_values.is_empty()
                || (!args_empty && !definition.up_values.is_empty())
        })
    }
}

fn hold_all_literal_args_are_stable(args: &[Expr]) -> bool {
    args.iter()
        .all(|arg| !is_direct_hold_escape_wrapper(arg) && sequence_items(arg).is_none())
}

fn is_direct_hold_escape_wrapper(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, args }
            if args.len() == 1
                && (is_system_symbol_name(head, "Evaluate")
                    || is_system_symbol_name(head, "Unevaluated"))
    )
}

fn failure_details_would_normalize(expr: &Expr) -> bool {
    if association_entries_from_expr(expr).is_some() {
        return false;
    }

    if parse_association_entry_form(expr).is_some() {
        return true;
    }

    let Expr::Call { head, args } = expr else {
        return false;
    };
    is_system_symbol_name(head, heads::structure::LIST)
        && args.iter().all(immediate_rule_entry_expr)
}

fn immediate_rule_entry_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Rule") && args.len() == 2
    )
}
