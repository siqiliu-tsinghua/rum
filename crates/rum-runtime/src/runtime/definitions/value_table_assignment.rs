use super::super::*;
use super::definition_lhs_preparation::{
    DefinitionLhsPreparationContinuation, DefinitionLhsPreparationContinuationResult,
    PreparedDefinitionLhs, PreparedDefinitionStore,
};
use crate::runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) enum ValueTableKind {
    OwnValues,
    DownValues,
    UpValues,
    SubValues,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
struct AssignedValueTableRule {
    source: Expr,
    lhs: Expr,
    rhs: Expr,
    delayed: bool,
}

#[derive(Clone, FrameRootTrace)]
struct PreparedValueTableRule {
    store: PreparedDefinitionStore,
    rhs: Expr,
    delayed: bool,
}

#[derive(Clone, Default, FrameRootTrace)]
struct PreparedValueTableRuleAccumulator {
    tail: AccumulatorChain<PreparedValueTableRuleNode>,
    len: usize,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct PreparedValueTableRuleNode {
    #[accumulator_link]
    previous: AccumulatorChain<PreparedValueTableRuleNode>,
    rule: PreparedValueTableRule,
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct ValueTableAssignmentState {
    kind: ValueTableKind,
    target: Symbol,
    rhs: Expr,
    rules: Arc<[AssignedValueTableRule]>,
    cursor: usize,
    prepared: PreparedValueTableRuleAccumulator,
}

impl PreparedValueTableRuleAccumulator {
    fn appended(&self, rule: PreparedValueTableRule) -> Self {
        Self {
            tail: AccumulatorChain::new(Some(Arc::new(PreparedValueTableRuleNode {
                previous: self.tail.clone(),
                rule,
            }))),
            len: self.len + 1,
        }
    }

    fn into_vec_allow_shared(self) -> Vec<PreparedValueTableRule> {
        let mut rules = Vec::with_capacity(self.len);
        let mut node = self.tail.into_inner();
        while let Some(current) = node {
            match Arc::try_unwrap(current) {
                Ok(current) => {
                    rules.push(current.rule);
                    node = current.previous.into_inner();
                }
                Err(current) => {
                    rules.push(current.rule.clone());
                    node = current.previous.clone().into_inner();
                }
            }
        }
        rules.reverse();
        rules
    }
}

impl ValueTableKind {
    fn inspector_name(self) -> &'static str {
        match self {
            Self::OwnValues => "OwnValues",
            Self::DownValues => "DownValues",
            Self::UpValues => "UpValues",
            Self::SubValues => "SubValues",
        }
    }

    fn definition_target(self, symbol: Symbol) -> TaggedDefinitionTarget {
        match self {
            Self::OwnValues => TaggedDefinitionTarget::OwnValue(symbol),
            Self::DownValues => TaggedDefinitionTarget::DownValue(symbol),
            Self::UpValues => TaggedDefinitionTarget::UpValue(symbol),
            Self::SubValues => TaggedDefinitionTarget::SubValue(symbol),
        }
    }
}

impl Session {
    pub(in crate::runtime) fn normalize_value_table_target_expr(
        &self,
        expr: &Expr,
    ) -> Option<(ValueTableKind, Symbol)> {
        let Expr::Call { head, args } = expr else {
            return None;
        };
        if args.len() != 1 {
            return None;
        }

        let inspector = expr_as_symbol(head)?;
        if inspector.context() != SYSTEM_CONTEXT {
            return None;
        }

        let kind = match inspector.name() {
            "OwnValues" => ValueTableKind::OwnValues,
            "DownValues" => ValueTableKind::DownValues,
            "UpValues" => ValueTableKind::UpValues,
            "SubValues" => ValueTableKind::SubValues,
            _ => return None,
        };
        let target = *expr_as_symbol(&args[0])?;
        Some((kind, target))
    }

    /// Replaces one value table after validating the complete serialized table.
    /// Invalid assignments use the dedicated `<Inspector>::setval` diagnostic.
    pub(in crate::runtime) fn replace_value_table_from_expr(
        &mut self,
        kind: ValueTableKind,
        target: Symbol,
        rhs: Expr,
    ) -> Result<Expr, EvalError> {
        let rules = match parse_assigned_value_table(kind, target, &rhs) {
            Ok(rules) => rules,
            Err(message) => {
                self.push_message(kind.inspector_name(), "setval", message);
                return Ok(rhs);
            }
        };

        let state = ValueTableAssignmentState {
            kind,
            target,
            rhs,
            rules: Arc::from(rules),
            cursor: 0,
            prepared: PreparedValueTableRuleAccumulator::default(),
        };
        let Some(rule) = state.rules.first() else {
            return Ok(self.finish_prepared_value_table_assignment(state));
        };

        self.prepare_definition_lhs(
            rule.lhs.clone(),
            DefinitionLhsPreparationContinuation::ValueTableEntry { state },
        )
    }

    pub(super) fn resume_prepared_value_table_entry(
        &mut self,
        mut state: ValueTableAssignmentState,
        prepared: PreparedDefinitionLhs,
    ) -> Result<DefinitionLhsPreparationContinuationResult, EvalError> {
        let rule = state
            .rules
            .get(state.cursor)
            .expect("value-table preparation cursor should name the current entry");
        let expected_target = state.kind.definition_target(state.target);
        let store = match prepared.into_store_for_owner(expected_target) {
            Ok(store) => store,
            Err(lhs) => {
                self.push_message(
                state.kind.inspector_name(),
                "setval",
                format!(
                    "Cannot assign rule {} to {}[{}]: its left-hand side {} does not define the target's {} table.",
                    rule.source,
                    state.kind.inspector_name(),
                    state.target,
                    lhs,
                    state.kind.inspector_name()
                ),
            );
                return Ok(DefinitionLhsPreparationContinuationResult::Complete(
                    state.rhs,
                ));
            }
        };

        // Append the completed entry before advancing the cursor. A suspended
        // continuation therefore resumes the current entry exactly once, while
        // every earlier prepared rule remains in the Arc-backed prefix.
        state.prepared = state.prepared.appended(PreparedValueTableRule {
            store,
            rhs: rule.rhs.clone(),
            delayed: rule.delayed,
        });
        state.cursor += 1;

        if let Some(next) = state.rules.get(state.cursor) {
            return Ok(DefinitionLhsPreparationContinuationResult::PrepareNext {
                lhs: next.lhs.clone(),
                continuation: DefinitionLhsPreparationContinuation::ValueTableEntry { state },
            });
        }

        Ok(DefinitionLhsPreparationContinuationResult::Complete(
            self.finish_prepared_value_table_assignment(state),
        ))
    }

    fn finish_prepared_value_table_assignment(&mut self, state: ValueTableAssignmentState) -> Expr {
        let ValueTableAssignmentState {
            kind,
            target,
            rhs,
            prepared,
            ..
        } = state;

        // Lhs preparation can run user code, so protection must be checked again
        // after every entry is ready and before the first table mutation.
        if !self.ensure_symbol_not_protected_for_write("Set", &target) {
            return rhs;
        }

        // Parsing, preparation, target-kind validation, and the protection
        // recheck above all complete before this first mutation.
        self.clear_value_table(kind, &target);
        for rule in prepared.into_vec_allow_shared() {
            self.store_prepared_definition(rule.store, rule.rhs, rule.delayed);
        }
        rhs
    }

    fn clear_value_table(&mut self, kind: ValueTableKind, target: &Symbol) {
        let Some(definition) = self.definitions.get_mut(target) else {
            return;
        };

        let (changed, removed_up_values) = match kind {
            ValueTableKind::OwnValues => (definition.clear_own_values(), false),
            ValueTableKind::DownValues => {
                let changed = !definition.down_values.is_empty();
                definition.down_values.clear();
                definition.note_down_values_rebuilt(target);
                (changed, false)
            }
            ValueTableKind::UpValues => {
                let changed = !definition.up_values.is_empty();
                definition.up_values.clear();
                definition.note_up_values_rebuilt();
                (changed, changed)
            }
            ValueTableKind::SubValues => {
                let changed = !definition.sub_values.is_empty();
                definition.sub_values.clear();
                definition.note_sub_values_rebuilt();
                (changed, false)
            }
        };
        let definition_is_empty = definition.is_empty();

        if definition_is_empty {
            self.definitions.remove(target);
        }
        if removed_up_values {
            self.refresh_has_up_value_definitions();
        }
        if changed {
            if kind == ValueTableKind::OwnValues {
                self.installed_system_own_values.remove(target);
            }
            self.bump_evaluation_epoch_for_symbol(target);
        }
        self.refresh_definition_memory();
    }
}

fn parse_assigned_value_table(
    kind: ValueTableKind,
    target: Symbol,
    rhs: &Expr,
) -> Result<Vec<AssignedValueTableRule>, String> {
    let Expr::Call { head, args } = rhs else {
        return Err(format!(
            "{}[{}] must be assigned a List of Rule or RuleDelayed entries; received {}.",
            kind.inspector_name(),
            target,
            rhs
        ));
    };
    if !is_system_symbol_name(head, "List") {
        return Err(format!(
            "{}[{}] must be assigned a List of Rule or RuleDelayed entries; received {}.",
            kind.inspector_name(),
            target,
            rhs
        ));
    }

    let mut rules = Vec::with_capacity(args.len());
    for rule_expr in args {
        let Expr::Call {
            head: rule_head,
            args: rule_args,
        } = rule_expr
        else {
            return Err(malformed_value_table_rule_message(kind, target, rule_expr));
        };
        if rule_args.len() != 2 {
            return Err(malformed_value_table_rule_message(kind, target, rule_expr));
        }

        let delayed = if is_system_symbol_name(rule_head, "RuleDelayed") {
            true
        } else if is_system_symbol_name(rule_head, "Rule") {
            false
        } else {
            return Err(malformed_value_table_rule_message(kind, target, rule_expr));
        };

        let lhs = strip_hold_pattern(&rule_args[0]);

        // Inspectors serialize every entry as RuleDelayed, even an immediate own
        // value. Its rhs is already evaluated, so the round-trip safely restores it
        // as a delayed own value.
        rules.push(AssignedValueTableRule {
            source: rule_expr.clone(),
            lhs: lhs.clone(),
            rhs: rule_args[1].clone(),
            delayed,
        });
    }

    Ok(rules)
}

fn malformed_value_table_rule_message(kind: ValueTableKind, target: Symbol, rule: &Expr) -> String {
    format!(
        "Cannot assign entry {} to {}[{}]: expected Rule[lhs, rhs] or RuleDelayed[lhs, rhs].",
        rule,
        kind.inspector_name(),
        target
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::suspension::AbandonReason;
    use crate::runtime::test_builtins::session_with_test_control_builtins;

    #[test]
    fn abandoning_prepared_value_table_keeps_old_definitions_and_skips_pending_lhs() {
        let mut session = session_with_test_control_builtins();
        session
            .eval_text("f[0] := old; hits = 0; kind := (hits++; InputString[\"type\"])")
            .unwrap();
        let before = session.eval_text("DownValues[f]").unwrap().value;
        let failure = session
            .eval_request(
                "DownValues[f] = {HoldPattern[f[1]] :> one, HoldPattern[f[2]] :> two, \
                 HoldPattern[f[1 + 1, g[x_kind]]] :> three}",
            )
            .unwrap_err();
        assert!(matches!(
            failure.error,
            SessionRequestError::InputRequired { request } if request.prompt == "type"
        ));
        assert_eq!(
            session.abandon_pending_suspension(AbandonReason::CancelledRequest),
            AbandonReason::CancelledRequest
        );
        assert!(session.pending_input_request().is_none());
        assert_eq!(session.eval_text("DownValues[f]").unwrap().value, before);
        assert_eq!(
            session
                .eval_text("{hits, f[0], f[1], f[2]}")
                .unwrap()
                .value
                .to_string(),
            "List[1, old, f[1], f[2]]"
        );
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<AssignedValueTableRule>();
    assert_traces::<PreparedValueTableRule>();
    assert_traces::<PreparedValueTableRuleAccumulator>();
    assert_traces::<PreparedValueTableRuleNode>();
    assert_traces::<ValueTableAssignmentState>();
    assert_traces::<ValueTableKind>();
};
