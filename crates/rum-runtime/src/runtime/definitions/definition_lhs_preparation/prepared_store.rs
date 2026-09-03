use super::super::super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

/// A definition identity that has passed through the shared lhs preparation
/// pipeline. Its fields stay private so storage cannot be called with an
/// independently supplied owner and lhs.
#[derive(Clone, FrameRootTrace)]
pub(in crate::runtime::definitions) struct PreparedDefinitionStore {
    lhs: Expr,
    owner: TaggedDefinitionTarget,
}

impl PreparedDefinitionStore {
    fn new(lhs: Expr, owner: TaggedDefinitionTarget) -> Self {
        Self { lhs, owner }
    }

    pub(in crate::runtime::definitions) fn symbol(&self) -> &Symbol {
        self.owner.symbol()
    }

    fn into_parts(self) -> (TaggedDefinitionTarget, Expr) {
        (self.owner, self.lhs)
    }
}

/// A definition identity prepared under current state and classified for
/// removal. Like the store token, it cannot be assembled from a raw lhs by a
/// source-language consumer.
pub(in crate::runtime::definitions) struct PreparedDefinitionRemoval {
    lhs: Expr,
    owner: TaggedDefinitionTarget,
}

impl PreparedDefinitionRemoval {
    fn new(lhs: Expr, owner: TaggedDefinitionTarget) -> Self {
        Self { lhs, owner }
    }

    pub(in crate::runtime::definitions) fn symbol(&self) -> &Symbol {
        self.owner.symbol()
    }

    fn into_parts(self) -> (TaggedDefinitionTarget, Expr) {
        (self.owner, self.lhs)
    }
}

pub(in crate::runtime::definitions) enum PreparedDefinitionAssignment {
    Message(MessageNameKey),
    SyntaxInformation { lhs: Expr, symbol: Symbol },
    Definition(PreparedDefinitionStore),
    Invalid(Expr),
}

pub(in crate::runtime::definitions) enum PreparedUnsetDefinitionTarget {
    Message(MessageNameKey),
    Options(Symbol),
    SyntaxInformation(Symbol),
    OwnValue(Symbol),
    Definition(PreparedDefinitionRemoval),
    TaggedNoop(Symbol),
}

impl PreparedUnsetDefinitionTarget {
    pub(in crate::runtime::definitions) fn symbol(&self) -> &Symbol {
        match self {
            Self::Message(key) => &key.symbol,
            Self::Options(symbol)
            | Self::SyntaxInformation(symbol)
            | Self::OwnValue(symbol)
            | Self::TaggedNoop(symbol) => symbol,
            Self::Definition(prepared) => prepared.symbol(),
        }
    }
}

impl PreparedDefinitionLhs {
    pub(in crate::runtime::definitions) fn into_assignment(self) -> PreparedDefinitionAssignment {
        let Self { lhs, target, .. } = self;
        match target {
            Some(DefinitionAssignmentTarget::Message(key)) => {
                PreparedDefinitionAssignment::Message(key)
            }
            Some(DefinitionAssignmentTarget::SyntaxInformation(symbol)) => {
                PreparedDefinitionAssignment::SyntaxInformation { lhs, symbol }
            }
            Some(DefinitionAssignmentTarget::Definition(owner)) => {
                PreparedDefinitionAssignment::Definition(PreparedDefinitionStore::new(lhs, owner))
            }
            None => PreparedDefinitionAssignment::Invalid(lhs),
        }
    }

    pub(in crate::runtime::definitions) fn into_tagged_store(
        self,
        tag: &Symbol,
    ) -> Result<PreparedDefinitionStore, TagDefinitionError> {
        let lhs = self.lhs;
        let owner = classify_tagged_definition(tag, &lhs)?;
        Ok(PreparedDefinitionStore::new(lhs, owner))
    }

    pub(in crate::runtime::definitions) fn into_store_for_owner(
        self,
        expected_owner: TaggedDefinitionTarget,
    ) -> Result<PreparedDefinitionStore, Expr> {
        let lhs = self.lhs;
        if classify_tagged_definition(expected_owner.symbol(), &lhs) != Ok(expected_owner.clone()) {
            return Err(lhs);
        }
        Ok(PreparedDefinitionStore::new(lhs, expected_owner))
    }

    pub(in crate::runtime::definitions) fn into_upset_stores(
        self,
    ) -> Result<Vec<PreparedDefinitionStore>, Expr> {
        let lhs = self.lhs;
        let targets = collect_upset_target_symbols(&lhs);
        if targets.is_empty() {
            return Err(lhs);
        }

        let last = targets.len() - 1;
        let mut final_lhs = Some(lhs);
        Ok(targets
            .into_iter()
            .enumerate()
            .map(|(index, symbol)| {
                let store_lhs = if index == last {
                    final_lhs
                        .take()
                        .expect("the final inferred UpSet owner consumes the canonical lhs")
                } else {
                    final_lhs
                        .as_ref()
                        .expect("the canonical UpSet lhs remains until the final owner")
                        .clone()
                };
                PreparedDefinitionStore::new(store_lhs, TaggedDefinitionTarget::UpValue(symbol))
            })
            .collect())
    }

    pub(in crate::runtime::definitions) fn into_tagged_removal(
        self,
        tag: &Symbol,
    ) -> Result<PreparedDefinitionRemoval, TagDefinitionError> {
        let lhs = self.lhs;
        let owner = classify_tagged_definition(tag, &lhs)?;
        Ok(PreparedDefinitionRemoval::new(lhs, owner))
    }
}

impl Session {
    #[doc(hidden)]
    pub fn protected_or_known_prepared_definition_target_symbol(
        &self,
        prepared: &PreparedDefinitionLhs,
    ) -> Option<Symbol> {
        let Some(DefinitionAssignmentTarget::Definition(target)) = &prepared.target else {
            return None;
        };
        let symbol = *target.symbol();
        (self.symbol_is_protected(&symbol)
            || is_known_system_symbol(&symbol)
            || self.builtin_spec_for_symbol(&symbol).is_some())
        .then_some(symbol)
    }

    #[doc(hidden)]
    pub fn store_prepared_definition_lhs_immediate(
        &mut self,
        prepared: PreparedDefinitionLhs,
        rhs: Expr,
    ) {
        if let PreparedDefinitionAssignment::Definition(store) = prepared.into_assignment() {
            self.store_prepared_definition(store, rhs, false);
        }
    }

    /// Stores an immediate own value for an already-normalized symbol target.
    /// A symbol has no argument subtree whose identity could require lhs
    /// preparation, so this narrow bridge does not admit a raw rule lhs.
    #[doc(hidden)]
    pub fn store_immediate_symbol_own_value(&mut self, symbol: Symbol, rhs: Expr) {
        store_immediate_symbol_own_value(self, symbol, rhs);
    }

    pub(in crate::runtime::definitions) fn store_prepared_definition(
        &mut self,
        prepared: PreparedDefinitionStore,
        rhs: Expr,
        delayed: bool,
    ) {
        store(self, prepared, rhs, delayed);
    }

    pub(in crate::runtime::definitions) fn remove_prepared_definition(
        &mut self,
        prepared: PreparedDefinitionRemoval,
    ) -> bool {
        remove(self, prepared)
    }

    pub(in crate::runtime) fn store_trusted_system_down_value(
        &mut self,
        trusted: TrustedSystemDownValueLhs,
        rhs: Expr,
    ) {
        store_trusted_system_down_value(self, trusted, rhs);
    }

    pub(in crate::runtime::definitions) fn prepared_unset_target(
        &self,
        prepared: PreparedDefinitionLhs,
    ) -> Result<PreparedUnsetDefinitionTarget, DefinitionTargetNormalizationError> {
        let lhs = prepared.lhs;
        if let Some(key) = self.message_name_key_from_expr(&lhs) {
            return Ok(PreparedUnsetDefinitionTarget::Message(key));
        }

        if let Some(symbol) = self.normalize_options_target_expr(&lhs) {
            return Ok(PreparedUnsetDefinitionTarget::Options(symbol));
        }

        if let Some(symbol) = self.normalize_syntax_information_target_expr(&lhs) {
            return Ok(PreparedUnsetDefinitionTarget::SyntaxInformation(symbol));
        }

        if let Some(symbol) = expr_as_symbol(&lhs).copied() {
            return Ok(PreparedUnsetDefinitionTarget::OwnValue(symbol));
        }

        if let Expr::Call { head, args } = &lhs {
            if is_system_symbol_name(head, "TagSet") && args.len() == 2 {
                let tag = self.normalize_tag_target(&args[0])?;
                return Ok(match classify_tagged_definition(&tag, &args[1]) {
                    Ok(owner) => PreparedUnsetDefinitionTarget::Definition(
                        PreparedDefinitionRemoval::new(args[1].clone(), owner),
                    ),
                    Err(_) => PreparedUnsetDefinitionTarget::TaggedNoop(tag),
                });
            }
        }

        if let Some(symbol) = downvalue_head_symbol(&lhs).copied() {
            return Ok(PreparedUnsetDefinitionTarget::Definition(
                PreparedDefinitionRemoval::new(lhs, TaggedDefinitionTarget::DownValue(symbol)),
            ));
        }

        if let Some(symbol) = subvalue_base_symbol(&lhs).copied() {
            return Ok(PreparedUnsetDefinitionTarget::Definition(
                PreparedDefinitionRemoval::new(lhs, TaggedDefinitionTarget::SubValue(symbol)),
            ));
        }

        Err(DefinitionTargetNormalizationError::InvalidUnsetTarget)
    }
}

fn store(session: &mut Session, prepared: PreparedDefinitionStore, rhs: Expr, delayed: bool) {
    let (owner, lhs) = prepared.into_parts();
    let symbol = match owner {
        TaggedDefinitionTarget::OwnValue(symbol) => {
            let (stored_rhs, guards) = split_definition_rhs_guards(rhs.clone());
            let definition = session.definitions.entry(symbol).or_default();
            definition.store_own_value(stored_rhs, guards, delayed);

            if delayed {
                session.push_definition_stored_trace(
                    &symbol,
                    DefinitionKind::DelayedOwnValue,
                    || rhs.clone(),
                );
            } else {
                session.push_definition_stored_trace(&symbol, DefinitionKind::OwnValue, || {
                    rhs.clone()
                });
            }
            symbol
        }
        TaggedDefinitionTarget::DownValue(symbol) => {
            let stores_default_value =
                symbol.context() == SYSTEM_CONTEXT && symbol.name() == heads::patterns::DEFAULT;
            let (stored_rhs, guards) = split_definition_rhs_guards(rhs.clone());
            let rhs_may_splice_without_bindings = rule_rhs_may_splice_without_bindings(&stored_rhs);
            let matcher = session.compile_down_value_matcher(&lhs, &guards);
            let matcher_attribute_revision = session.attribute_revision;
            let lhs_is_ground =
                guards.is_empty() && !expr_contains_pattern_syntax(strip_hold_pattern(&lhs));
            let definition = session.definitions.entry(symbol).or_default();
            let down_value = DownValue {
                lhs: lhs.clone(),
                rhs: stored_rhs,
                guards,
                delayed,
                lhs_is_ground,
                rhs_may_splice_without_bindings,
                matcher,
                matcher_attribute_revision,
            };
            if stores_default_value {
                replace_or_append_default_value(&mut definition.down_values, &lhs, down_value);
                definition.note_down_values_rebuilt(&symbol);
            } else {
                let (store_index, replaced_was_ground) =
                    replace_or_insert_down_value(definition, &lhs, down_value);
                definition.note_down_value_stored(
                    &symbol,
                    lhs_is_ground,
                    replaced_was_ground,
                    store_index,
                );
            }
            session.push_definition_stored_trace(&symbol, DefinitionKind::DownValue, || {
                Expr::call(
                    system_expr(if delayed { "SetDelayed" } else { "Set" }),
                    vec![lhs.clone(), rhs.clone()],
                )
            });
            symbol
        }
        TaggedDefinitionTarget::SubValue(symbol) => {
            let (stored_rhs, guards) = split_definition_rhs_guards(rhs.clone());
            let rhs_may_splice_without_bindings = rule_rhs_may_splice_without_bindings(&stored_rhs);
            let matcher = session.compile_down_value_matcher(&lhs, &guards);
            let matcher_attribute_revision = session.attribute_revision;
            let definition = session.definitions.entry(symbol).or_default();
            replace_or_insert_sub_value(
                definition,
                &lhs,
                SubValue {
                    lhs: lhs.clone(),
                    rhs: stored_rhs,
                    delayed,
                    rhs_may_splice_without_bindings,
                    matcher,
                    matcher_attribute_revision,
                    guards,
                },
            );
            session.push_definition_stored_trace(&symbol, DefinitionKind::SubValue, || {
                Expr::call(
                    system_expr(if delayed { "SetDelayed" } else { "Set" }),
                    vec![lhs.clone(), rhs.clone()],
                )
            });
            symbol
        }
        TaggedDefinitionTarget::UpValue(symbol) => {
            let (stored_rhs, guards) = split_definition_rhs_guards(rhs.clone());
            let rhs_may_splice_without_bindings = rule_rhs_may_splice_without_bindings(&stored_rhs);
            let matcher = session.compile_down_value_matcher(&lhs, &guards);
            let matcher_attribute_revision = session.attribute_revision;
            let definition = session.definitions.entry(symbol).or_default();
            replace_or_insert_up_value(
                definition,
                &lhs,
                UpValue {
                    lhs: lhs.clone(),
                    rhs: stored_rhs,
                    delayed,
                    rhs_may_splice_without_bindings,
                    matcher,
                    matcher_attribute_revision,
                    guards,
                },
            );
            session.has_up_value_definitions = true;
            session.push_definition_stored_trace(&symbol, DefinitionKind::UpValue, || {
                Expr::call(
                    system_expr(if delayed { "SetDelayed" } else { "Set" }),
                    vec![lhs.clone(), rhs.clone()],
                )
            });
            symbol
        }
    };

    session.installed_system_own_values.remove(&symbol);
    session.bump_evaluation_epoch_for_symbol(&symbol);
}

fn store_trusted_system_down_value(
    session: &mut Session,
    trusted: TrustedSystemDownValueLhs,
    rhs: Expr,
) {
    let (symbol, lhs) = trusted.into_parts();
    debug_assert_eq!(
        classify_definition_lhs(&lhs),
        Some(TaggedDefinitionTarget::DownValue(symbol)),
        "trusted system-catalog downvalue lhs must classify to its declared owner"
    );
    store(
        session,
        PreparedDefinitionStore {
            lhs,
            owner: TaggedDefinitionTarget::DownValue(symbol),
        },
        rhs,
        false,
    );
}

fn store_immediate_symbol_own_value(session: &mut Session, symbol: Symbol, rhs: Expr) {
    store(
        session,
        PreparedDefinitionStore {
            lhs: Expr::Atom(Atom::Symbol(symbol)),
            owner: TaggedDefinitionTarget::OwnValue(symbol),
        },
        rhs,
        false,
    );
}

fn remove(session: &mut Session, prepared: PreparedDefinitionRemoval) -> bool {
    let (owner, lhs) = prepared.into_parts();
    let symbol = *owner.symbol();
    let changed = match owner {
        TaggedDefinitionTarget::OwnValue(_) => session
            .definitions
            .get_mut(&symbol)
            .is_some_and(SymbolDefinition::clear_own_values),
        TaggedDefinitionTarget::DownValue(_) => remove_prepared_down_value(session, &symbol, &lhs),
        TaggedDefinitionTarget::SubValue(_) => remove_prepared_sub_value(session, &symbol, &lhs),
        TaggedDefinitionTarget::UpValue(_) => remove_prepared_up_value(session, &symbol, &lhs),
    };

    if changed && matches!(owner, TaggedDefinitionTarget::OwnValue(_)) {
        session.installed_system_own_values.remove(&symbol);
        session.bump_evaluation_epoch_for_symbol(&symbol);
    }
    session.prune_definition_if_empty(&symbol);
    changed
}

fn remove_prepared_down_value(session: &mut Session, symbol: &Symbol, lhs: &Expr) -> bool {
    let Some(definition) = session.definitions.get_mut(symbol) else {
        return false;
    };

    let original_len = definition.down_values.len();
    definition
        .down_values
        .retain(|entry| !lhs_patterns_equivalent(&entry.lhs, lhs));
    let changed = definition.down_values.len() != original_len;
    if changed {
        definition.note_down_values_rebuilt(symbol);
        session.bump_evaluation_epoch_for_symbol(symbol);
    }
    changed
}

fn remove_prepared_up_value(session: &mut Session, symbol: &Symbol, lhs: &Expr) -> bool {
    let changed;
    let symbol_has_no_more_up_values;

    {
        let Some(definition) = session.definitions.get_mut(symbol) else {
            return false;
        };

        let original_len = definition.up_values.len();
        definition
            .up_values
            .retain(|entry| !lhs_patterns_equivalent(&entry.lhs, lhs));
        changed = definition.up_values.len() != original_len;
        if changed {
            definition.note_up_values_rebuilt();
        }
        symbol_has_no_more_up_values = definition.up_values.is_empty();
    }

    if changed {
        if symbol_has_no_more_up_values {
            session.refresh_has_up_value_definitions();
        }
        session.bump_evaluation_epoch_for_symbol(symbol);
    }
    changed
}

fn remove_prepared_sub_value(session: &mut Session, symbol: &Symbol, lhs: &Expr) -> bool {
    let Some(definition) = session.definitions.get_mut(symbol) else {
        return false;
    };

    let original_len = definition.sub_values.len();
    definition
        .sub_values
        .retain(|entry| !lhs_patterns_equivalent(&entry.lhs, lhs));
    let changed = definition.sub_values.len() != original_len;
    if changed {
        definition.note_sub_values_rebuilt();
        session.bump_evaluation_epoch_for_symbol(symbol);
    }
    changed
}

fn replace_or_insert_down_value(
    definition: &mut SymbolDefinition,
    lhs: &Expr,
    new_entry: DownValue,
) -> (usize, Option<bool>) {
    if let Some(index) =
        definition.equivalent_down_value_index(lhs, &new_entry.guards, new_entry.lhs_is_ground)
    {
        let replaced_was_ground = definition.down_values[index].lhs_is_ground;
        definition.down_values[index] = new_entry;
        return (index, Some(replaced_was_ground));
    }

    let new_specificity = definition_specificity(lhs);
    let index = definition.down_values.last().map_or(0, |last| {
        if new_specificity <= definition_specificity(&last.lhs) {
            definition.down_values.len()
        } else {
            definition
                .down_values
                .partition_point(|entry| definition_specificity(&entry.lhs) >= new_specificity)
        }
    });
    definition.down_values.insert(index, new_entry);
    (index, None)
}

fn replace_or_insert_sub_value(definition: &mut SymbolDefinition, lhs: &Expr, new_entry: SubValue) {
    let lookup = definition.prepare_sub_value_store(lhs);
    let equivalent_index = guarded_definition_identity_index(
        &definition.sub_values,
        lhs,
        &new_entry.guards,
        lookup.equivalent_index(),
        |entry| (&entry.lhs, &entry.guards),
    );
    let (store_index, replaced) = replace_or_insert_definition_by_specificity(
        &mut definition.sub_values,
        lhs,
        new_entry,
        |entry| &entry.lhs,
        equivalent_index,
    );
    definition.note_sub_value_stored(store_index, replaced, lookup);
}

fn replace_or_insert_up_value(definition: &mut SymbolDefinition, lhs: &Expr, new_entry: UpValue) {
    let lookup = definition.prepare_up_value_store(lhs);
    let equivalent_index = guarded_definition_identity_index(
        &definition.up_values,
        lhs,
        &new_entry.guards,
        lookup.equivalent_index(),
        |entry| (&entry.lhs, &entry.guards),
    );
    let (store_index, replaced) = replace_or_insert_definition_by_specificity(
        &mut definition.up_values,
        lhs,
        new_entry,
        |entry| &entry.lhs,
        equivalent_index,
    );
    definition.note_up_value_stored(store_index, replaced, lookup);
}

fn guarded_definition_identity_index<T, F>(
    entries: &[T],
    lhs: &Expr,
    guards: &[Expr],
    lhs_equivalent_index: Option<usize>,
    identity_of: F,
) -> Option<usize>
where
    F: Fn(&T) -> (&Expr, &[Expr]),
{
    let index = lhs_equivalent_index?;
    let (stored_lhs, stored_guards) = identity_of(&entries[index]);
    if down_value_patterns_equivalent(stored_lhs, stored_guards, lhs, guards) {
        return Some(index);
    }

    entries
        .iter()
        .enumerate()
        .find_map(|(candidate_index, entry)| {
            (candidate_index != index)
                .then(|| identity_of(entry))
                .and_then(|(stored_lhs, stored_guards)| {
                    down_value_patterns_equivalent(stored_lhs, stored_guards, lhs, guards)
                        .then_some(candidate_index)
                })
        })
}

fn replace_or_insert_definition_by_specificity<T, F>(
    entries: &mut Vec<T>,
    lhs: &Expr,
    new_entry: T,
    lhs_of: F,
    equivalent_index: Option<usize>,
) -> (usize, bool)
where
    F: Fn(&T) -> &Expr,
{
    if let Some(index) = equivalent_index {
        entries[index] = new_entry;
        return (index, true);
    }

    let new_specificity = definition_specificity(lhs);
    let index = entries.last().map_or(0, |last| {
        if new_specificity <= definition_specificity(lhs_of(last)) {
            entries.len()
        } else {
            entries
                .partition_point(|entry| definition_specificity(lhs_of(entry)) >= new_specificity)
        }
    });
    entries.insert(index, new_entry);
    (index, false)
}

fn definition_specificity(expr: &Expr) -> usize {
    match expr {
        Expr::Atom(_) => 4,
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            definition_specificity(&args[1])
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Blank") => {
            if args.is_empty() {
                1
            } else {
                2
            }
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Optional")
                || is_system_symbol_name(head, "Repeated")
                || is_system_symbol_name(head, "RepeatedNull"))
                && !args.is_empty() =>
        {
            definition_specificity(&args[0])
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Condition")
                || is_system_symbol_name(head, "PatternTest"))
                && !args.is_empty() =>
        {
            1 + definition_specificity(&args[0])
        }
        Expr::Call { head, args } => {
            3 + definition_specificity(head)
                + args.iter().map(definition_specificity).sum::<usize>()
        }
    }
}

fn replace_or_append_default_value(entries: &mut Vec<DownValue>, lhs: &Expr, new_entry: DownValue) {
    let equivalent_index = if new_entry.guards.is_empty() {
        entries
            .iter()
            .position(|entry| entry.guards.is_empty() && lhs_patterns_equivalent(&entry.lhs, lhs))
    } else {
        entries.iter().position(|entry| {
            down_value_patterns_equivalent(&entry.lhs, &entry.guards, lhs, &new_entry.guards)
        })
    };

    if let Some(index) = equivalent_index {
        entries[index] = new_entry;
    } else {
        entries.push(new_entry);
    }
}
