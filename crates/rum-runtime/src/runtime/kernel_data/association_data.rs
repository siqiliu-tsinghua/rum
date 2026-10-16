mod resume_frames;
use resume_frames::*;

use super::super::*;
use crate::runtime::definitions::application::DefinitionResumeRoute;
use crate::runtime::kernel_api::slot_witness;
use crate::runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};
use crate::runtime::native_stack_guard::{NativeStackDepth, run_with_native_stack_guard};
use std::collections::HashMap;
use std::sync::Arc;

pub(in crate::runtime) const ASSOCIATION_INDEX_THRESHOLD: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub struct AssociationEntryForm {
    /// Association entry key expression.
    pub key: Expr,
    /// Association entry value expression.
    pub value: Expr,
    /// Whether the entry preserves a RuleDelayed head.
    pub delayed: bool,
}

pub(in crate::runtime) struct AssociationEntryCursor<'a> {
    entries: AssociationEntryCursorStorage<'a>,
}

enum AssociationEntryCursorStorage<'a> {
    Borrowed(&'a SharedArgs),
    Owned(Vec<(Expr, Expr)>),
}

impl AssociationEntryCursor<'_> {
    pub(in crate::runtime) fn len(&self) -> usize {
        match &self.entries {
            AssociationEntryCursorStorage::Borrowed(args) => args.len(),
            AssociationEntryCursorStorage::Owned(entries) => entries.len(),
        }
    }

    pub(in crate::runtime) fn entry(&self, position: usize) -> Option<(&Expr, &Expr)> {
        match &self.entries {
            AssociationEntryCursorStorage::Borrowed(args) => {
                flat_association_rule_parts(args.get(position)?)
            }
            AssociationEntryCursorStorage::Owned(entries) => {
                entries.get(position).map(|(key, value)| (key, value))
            }
        }
    }

    pub(in crate::runtime) fn iter(&self) -> impl Iterator<Item = (&Expr, &Expr)> {
        (0..self.len()).map(|position| {
            self.entry(position)
                .expect("validated association cursor entries remain rules")
        })
    }
}

pub fn association_entries_from_expr(expr: &Expr) -> Option<Vec<(Expr, Expr)>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Association") {
        return None;
    }

    Some(canonicalize_association_entries(
        parse_association_entries_deep(args)?,
    ))
}

pub fn association_entry_forms_from_expr(expr: &Expr) -> Option<Vec<AssociationEntryForm>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Association") {
        return None;
    }

    Some(canonicalize_association_entry_forms(
        parse_association_entry_forms_deep(args)?,
    ))
}

pub(in crate::runtime) fn keyed_entry_forms_from_expr(
    expr: &Expr,
) -> Option<Vec<AssociationEntryForm>> {
    if let Some(entries) = association_entry_forms_from_expr(expr) {
        return Some(entries);
    }

    rule_list_entry_forms_from_expr(expr)
}

pub(in crate::runtime) fn parse_association_entries_deep(
    items: &[Expr],
) -> Option<Vec<(Expr, Expr)>> {
    let mut entries = Vec::new();
    for item in items {
        entries.extend(parse_association_item_deep(item)?);
    }
    Some(entries)
}

fn parse_association_item_deep(expr: &Expr) -> Option<Vec<(Expr, Expr)>> {
    if let Some(entry) = parse_association_entry(expr) {
        return Some(vec![entry]);
    }

    if let Some(entries) = association_entries_from_expr(expr) {
        return Some(entries);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    parse_association_entries_deep(args)
}

pub fn render_association(entries: Vec<(Expr, Expr)>) -> Expr {
    let entries = canonicalize_association_entries(entries);
    Expr::call(
        system_expr("Association"),
        entries
            .into_iter()
            .map(|(key, value)| Expr::call(system_expr("Rule"), vec![key, value]))
            .collect(),
    )
}

pub fn render_association_entry_forms_preserving_heads(entries: Vec<AssociationEntryForm>) -> Expr {
    render_association_entry_forms(entries)
}

pub(in crate::runtime) fn association_value_by_key(expr: &Expr, key: &Expr) -> Option<Expr> {
    association_entries_from_expr(expr)?
        .into_iter()
        .find(|(entry_key, _)| entry_key == key)
        .map(|(_, value)| value)
}

pub(in crate::runtime) fn association_position_and_value_by_key(
    expr: &Expr,
    key: &Expr,
) -> Option<(isize, Expr)> {
    association_entries_from_expr(expr)?
        .into_iter()
        .enumerate()
        .find(|(_, (entry_key, _))| entry_key == key)
        .map(|(index, (_, value))| (index as isize + 1, value))
}

pub(in crate::runtime) fn normalize_key_value_pattern_spec(spec: &Expr) -> Option<Expr> {
    if let Some(entry) = parse_association_entry_form(spec) {
        return Some(Expr::call(
            system_expr("List"),
            vec![association_entry_form_expr(entry)],
        ));
    }

    let Expr::Call { head, args } = spec else {
        return None;
    };
    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    let entries = parse_association_entry_forms(args)?;
    Some(Expr::call(
        system_expr("List"),
        entries
            .into_iter()
            .map(association_entry_form_expr)
            .collect(),
    ))
}

pub fn list_items(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, heads::structure::LIST) => {
            Some(args)
        }
        _ => None,
    }
}

fn canonicalize_association_entries(entries: Vec<(Expr, Expr)>) -> Vec<(Expr, Expr)> {
    let mut canonical: Vec<(Expr, Expr)> = Vec::new();
    let mut indexed_keys: Option<HashMap<Expr, usize>> = None;
    let entry_count = entries.len();

    for (key, value) in entries {
        if let Some(key_indices) = indexed_keys.as_mut() {
            if let Some(&existing_index) = key_indices.get(&key) {
                canonical[existing_index].1 = value;
            } else {
                key_indices.insert(key.clone(), canonical.len());
                canonical.push((key, value));
            }
            continue;
        }

        if let Some(existing_index) = canonical
            .iter()
            .position(|(existing_key, _)| existing_key == &key)
        {
            canonical[existing_index].1 = value;
        } else {
            canonical.push((key, value));
            if canonical.len() > ASSOCIATION_INDEX_THRESHOLD {
                let mut key_indices = HashMap::with_capacity(entry_count);
                key_indices.extend(
                    canonical
                        .iter()
                        .enumerate()
                        .map(|(index, (key, _))| (key.clone(), index)),
                );
                indexed_keys = Some(key_indices);
            }
        }
    }

    canonical
}

pub(in crate::runtime) fn parse_association_entry(expr: &Expr) -> Option<(Expr, Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    Some((args[0].clone(), args[1].clone()))
}

pub fn parse_association_entry_form(expr: &Expr) -> Option<AssociationEntryForm> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    Some(AssociationEntryForm {
        key: args[0].clone(),
        value: args[1].clone(),
        delayed: is_system_symbol_name(head, "RuleDelayed"),
    })
}

fn rule_list_entry_forms_from_expr(expr: &Expr) -> Option<Vec<AssociationEntryForm>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    parse_association_entry_forms(args)
}

fn parse_association_entry_forms(items: &[Expr]) -> Option<Vec<AssociationEntryForm>> {
    items.iter().map(parse_association_entry_form).collect()
}

pub(in crate::runtime) fn parse_association_entry_forms_deep(
    items: &[Expr],
) -> Option<Vec<AssociationEntryForm>> {
    let mut entries = Vec::new();
    for item in items {
        entries.extend(parse_association_item_forms_deep(item)?);
    }
    Some(entries)
}

fn parse_association_item_forms_deep(expr: &Expr) -> Option<Vec<AssociationEntryForm>> {
    if let Some(entry) = parse_association_entry_form(expr) {
        return Some(vec![entry]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if is_system_symbol_name(head, "Association")
        || is_system_symbol_name(head, heads::structure::LIST)
    {
        return parse_association_entry_forms_deep(args);
    }

    None
}

fn render_association_entry_forms(entries: Vec<AssociationEntryForm>) -> Expr {
    let entries = canonicalize_association_entry_forms(entries);
    Expr::call(
        system_expr("Association"),
        entries
            .into_iter()
            .map(association_entry_form_expr)
            .collect(),
    )
}

pub(in crate::runtime) fn association_entry_form_expr(entry: AssociationEntryForm) -> Expr {
    Expr::call(
        system_expr(if entry.delayed { "RuleDelayed" } else { "Rule" }),
        vec![entry.key, entry.value],
    )
}

fn canonicalize_association_entry_forms(
    entries: Vec<AssociationEntryForm>,
) -> Vec<AssociationEntryForm> {
    let mut canonical: Vec<AssociationEntryForm> = Vec::new();
    let mut indexed_keys: Option<HashMap<Expr, usize>> = None;
    let entry_count = entries.len();

    for entry in entries {
        if let Some(key_indices) = indexed_keys.as_mut() {
            if let Some(&existing_index) = key_indices.get(&entry.key) {
                canonical[existing_index] = entry;
            } else {
                key_indices.insert(entry.key.clone(), canonical.len());
                canonical.push(entry);
            }
            continue;
        }

        if let Some(existing_index) = canonical
            .iter()
            .position(|existing| existing.key == entry.key)
        {
            canonical[existing_index] = entry;
        } else {
            canonical.push(entry);
            if canonical.len() > ASSOCIATION_INDEX_THRESHOLD {
                let mut key_indices = HashMap::with_capacity(entry_count);
                key_indices.extend(
                    canonical
                        .iter()
                        .enumerate()
                        .map(|(index, entry)| (entry.key.clone(), index)),
                );
                indexed_keys = Some(key_indices);
            }
        }
    }

    canonical
}

pub(in crate::runtime) fn invalid_association_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a valid Association.")
}

pub(in crate::runtime) fn has_association_head(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { head, .. } if is_system_symbol_name(head, "Association"))
}

impl Session {
    pub(in crate::runtime) fn apply_association_to_args(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Option<Expr> {
        match args {
            [] => association_entries_from_expr(head).map(|_| head.clone()),
            [key] => self.lookup_association_key(head, key),
            _ => self.apply_association_to_key_path(head, args),
        }
    }

    fn lookup_association_key(&mut self, expr: &Expr, key: &Expr) -> Option<Expr> {
        Some(
            self.lookup_association_key_value(expr, key)?
                .unwrap_or_else(|| missing_key_absent_expr(key.clone())),
        )
    }

    pub(in crate::runtime) fn lookup_association_key_value(
        &mut self,
        expr: &Expr,
        key: &Expr,
    ) -> Option<Option<Expr>> {
        if let Some(value) = self.lookup_flat_association_key(expr, key) {
            return Some(value);
        }

        let entries = association_entries_from_expr(expr)?;
        Some(
            entries
                .iter()
                .find(|(entry_key, _)| entry_key == key)
                .map(|(_, value)| value.clone()),
        )
    }

    pub(in crate::runtime) fn association_entry_cursor<'a>(
        &mut self,
        expr: &'a Expr,
    ) -> Option<AssociationEntryCursor<'a>> {
        let Expr::Call { head, args } = expr else {
            return None;
        };
        if !is_system_symbol_name(head, "Association") {
            return None;
        }

        if self.flat_association_args_are_valid(args) {
            return Some(AssociationEntryCursor {
                entries: AssociationEntryCursorStorage::Borrowed(args),
            });
        }

        Some(AssociationEntryCursor {
            entries: AssociationEntryCursorStorage::Owned(association_entries_from_expr(expr)?),
        })
    }

    fn flat_association_args_are_valid(&mut self, args: &SharedArgs) -> bool {
        match {
            let result = self.association_lookup_cache.probe_args(args);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::AssociationLookupCache,
            );
            result
        } {
            AssociationLookupCacheProbe::Indexed(_) => true,
            AssociationLookupCacheProbe::Build => {
                if !args
                    .iter()
                    .all(|item| flat_association_rule_parts(item).is_some())
                {
                    return false;
                }
                {
                    let result = self.association_lookup_cache.insert_index(
                        args,
                        args.iter().map(|item| {
                            flat_association_rule_parts(item)
                                .expect("validated flat association rule")
                                .0
                        }),
                    );
                    self.refresh_memory_owner(
                        crate::runtime::debug_tools::resources::MemoryOwner::AssociationLookupCache,
                    );
                    result
                };
                true
            }
            AssociationLookupCacheProbe::Scan => args
                .iter()
                .all(|item| flat_association_rule_parts(item).is_some()),
        }
    }

    fn lookup_flat_association_key(&mut self, expr: &Expr, key: &Expr) -> Option<Option<Expr>> {
        let Expr::Call { head, args } = expr else {
            return None;
        };
        if !is_system_symbol_name(head, "Association") {
            return None;
        }

        match {
            let result = self.association_lookup_cache.probe(args, key);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::AssociationLookupCache,
            );
            result
        } {
            AssociationLookupCacheProbe::Indexed(position) => {
                return match position {
                    Some(position) => {
                        let (_, value) = flat_association_rule_parts(&args[position])?;
                        Some(Some(value.clone()))
                    }
                    None => Some(None),
                };
            }
            AssociationLookupCacheProbe::Build => {
                if !args
                    .iter()
                    .all(|item| flat_association_rule_parts(item).is_some())
                {
                    return None;
                }
                {
                    let result = self.association_lookup_cache.insert_index(
                        args,
                        args.iter().map(|item| {
                            flat_association_rule_parts(item)
                                .expect("validated flat association rule")
                                .0
                        }),
                    );
                    self.refresh_memory_owner(
                        crate::runtime::debug_tools::resources::MemoryOwner::AssociationLookupCache,
                    );
                    result
                };
                let AssociationLookupCacheProbe::Indexed(position) = ({
                    let result = self.association_lookup_cache.probe(args, key);
                    self.refresh_memory_owner(
                        crate::runtime::debug_tools::resources::MemoryOwner::AssociationLookupCache,
                    );
                    result
                }) else {
                    unreachable!("a newly inserted association index must be reusable");
                };
                return match position {
                    Some(position) => {
                        let (_, value) = flat_association_rule_parts(&args[position])?;
                        Some(Some(value.clone()))
                    }
                    None => Some(None),
                };
            }
            AssociationLookupCacheProbe::Scan => {}
        }

        let mut value = None;
        for item in args.iter().rev() {
            let (entry_key, entry_value) = flat_association_rule_parts(item)?;
            if value.is_none() {
                self.association_lookup_cache.record_linear_comparison();
                if entry_key == key {
                    value = Some(entry_value.clone());
                }
            }
        }

        Some(value)
    }

    fn apply_association_to_key_path(&mut self, head: &Expr, keys: &[Expr]) -> Option<Expr> {
        let mut current = head.clone();
        for (index, key) in keys.iter().enumerate() {
            let Some(value) = self.lookup_association_key(&current, key) else {
                return None;
            };

            if is_missing_expr(&value) || index + 1 == keys.len() {
                return Some(value);
            }

            if !has_association_head(&value) {
                return None;
            }
            current = value;
        }

        Some(current)
    }

    #[cfg(test)]
    pub(in crate::runtime) fn association_lookup_work(&self) -> AssociationLookupWork {
        self.association_lookup_cache.work()
    }
}

pub(in crate::runtime) fn flat_association_rule_parts(expr: &Expr) -> Option<(&Expr, &Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }
    Some((&args[0], &args[1]))
}

pub fn missing_key_absent_expr(key: Expr) -> Expr {
    Expr::call(system_expr("Missing"), vec![Expr::string("KeyAbsent"), key])
}

pub fn is_missing_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { head, .. } if is_system_symbol_name(head, "Missing"))
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
enum AssociationSetLValueTarget {
    FunctionCall {
        symbol: Symbol,
        selectors: Vec<Expr>,
    },
    Part {
        symbol: Symbol,
        selectors: Vec<Expr>,
    },
}

#[derive(Clone, FrameRootTrace)]
struct AssociationSelectorAccumulator {
    prefix: SharedArgs,
    tail: AccumulatorChain<AssociationSelectorNode>,
    len: usize,
}

#[derive(Clone, FrameRootTrace, AccumulatorChainNode)]
struct AssociationSelectorNode {
    #[accumulator_link]
    previous: AccumulatorChain<AssociationSelectorNode>,
    value: Expr,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedAssociationSetLValue {
    symbol: Symbol,
    selectors: Arc<[Expr]>,
    rhs: Expr,
    cursor: usize,
    evaluated: AssociationSelectorAccumulator,
}

impl AssociationSelectorAccumulator {
    fn freeze(values: Vec<Expr>) -> Self {
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
            tail: AccumulatorChain::new(Some(Arc::new(AssociationSelectorNode {
                previous: self.tail.clone(),
                value,
            }))),
            len: self.len + 1,
        }
    }

    fn into_vec(self) -> Vec<Expr> {
        let mut suffix = Vec::with_capacity(self.len.saturating_sub(self.prefix.len()));
        for current in self.tail.into_completion() {
            suffix.push(current.value);
        }
        suffix.reverse();

        let mut values = self.prefix.into_vec();
        values.reserve(suffix.len());
        values.extend(suffix);
        values
    }
}

impl SuspendedAssociationSetLValue {
    fn freeze(
        symbol: Symbol,
        selectors: Vec<Expr>,
        rhs: Expr,
        cursor: usize,
        evaluated: Vec<Expr>,
    ) -> Self {
        Self {
            symbol,
            selectors: Arc::from(selectors),
            rhs,
            cursor,
            evaluated: AssociationSelectorAccumulator::freeze(evaluated),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AssociationSetUpdate {
    Updated(Expr),
    MissingKey(Expr),
    Unchanged,
}

impl AssociationSetLValueTarget {
    fn symbol(&self) -> &Symbol {
        match self {
            Self::FunctionCall { symbol, .. } | Self::Part { symbol, .. } => symbol,
        }
    }

    fn is_direct_single_key_shape(&self) -> bool {
        match self {
            Self::FunctionCall { selectors, .. } => selectors.len() == 1,
            Self::Part { selectors, .. } => {
                matches!(selectors.as_slice(), [selector] if explicit_association_key_selector(selector).is_some())
            }
        }
    }
}

impl Session {
    pub(in crate::runtime) fn try_set_association_lvalue(
        &mut self,
        lhs: Expr,
        rhs: Expr,
    ) -> Result<Option<Expr>, EvalError> {
        let Some(target) = self.normalize_association_set_lvalue_target(&lhs) else {
            return Ok(None);
        };

        let direct_preflight = target.is_direct_single_key_shape()
            && self.association_lvalue_has_valid_direct_proof(target.symbol());
        if !direct_preflight && self.association_lvalue_current(target.symbol()).is_none() {
            return Ok(None);
        }

        if !self.ensure_symbol_not_protected_for_write("Set", target.symbol()) {
            return Ok(Some(rhs));
        }

        let value = match target {
            AssociationSetLValueTarget::FunctionCall { symbol, selectors } => self
                .resume_association_set_lvalue_selector_evaluation(
                    symbol,
                    selectors,
                    rhs,
                    Vec::new(),
                    0,
                )?,
            AssociationSetLValueTarget::Part { symbol, selectors } => {
                let direct_single_key = matches!(
                    selectors.as_slice(),
                    [selector] if explicit_association_key_selector(selector).is_some()
                );
                let evaluated_result = self.evaluate_part_selectors_owned(selectors);
                let witness = slot_witness(&evaluated_result);
                let evaluated_selectors = match evaluated_result {
                    Ok(evaluated_selectors) => evaluated_selectors,
                    Err(EvalError::InputRequired { request }) => {
                        return resume_frames::suspend_association_set_part_selectors(
                            self,
                            symbol,
                            direct_single_key,
                            rhs,
                            witness,
                            request,
                        )
                        .map(Some);
                    }
                    Err(error) => return Err(error),
                };
                self.finish_association_set_part_lvalue(
                    symbol,
                    direct_single_key,
                    evaluated_selectors,
                    rhs,
                )?
            }
        };
        Ok(Some(value))
    }

    fn normalize_association_set_lvalue_target(
        &self,
        lhs: &Expr,
    ) -> Option<AssociationSetLValueTarget> {
        let Expr::Call { head, args } = lhs else {
            return None;
        };

        if is_system_symbol_name(head, "Part") && args.len() >= 2 {
            let symbol = self.normalize_symbol_target_expr(&args[0], true)?;
            return Some(AssociationSetLValueTarget::Part {
                symbol,
                selectors: args[1..].to_vec(),
            });
        }

        let symbol = expr_as_symbol(head)?.clone();
        (!args.is_empty()).then_some(AssociationSetLValueTarget::FunctionCall {
            symbol,
            selectors: args.to_vec(),
        })
    }

    fn association_lvalue_current(&self, symbol: &Symbol) -> Option<Expr> {
        let current = self
            .definitions
            .get(symbol)
            .and_then(|definition| definition.own_value.clone())?;
        association_entry_forms_from_expr(&current)?;
        Some(current)
    }

    fn resume_association_set_lvalue_selector_evaluation(
        &mut self,
        symbol: Symbol,
        selectors: Vec<Expr>,
        rhs: Expr,
        mut evaluated_selectors: Vec<Expr>,
        mut index: usize,
    ) -> Result<Expr, EvalError> {
        loop {
            let Some(selector) = selectors.get(index) else {
                return self.finish_association_set_function_lvalue(
                    symbol,
                    &selectors,
                    evaluated_selectors,
                    rhs,
                );
            };

            let selector_result = self.evaluate(selector.clone());
            let witness = slot_witness(&selector_result);
            match selector_result {
                Ok(selector) => {
                    evaluated_selectors.push(selector);
                    index += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    return resume_frames::suspend_association_set_lvalue_selectors(
                        self,
                        symbol,
                        selectors,
                        rhs,
                        index,
                        evaluated_selectors,
                        witness,
                        request,
                    );
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_suspended_association_set_lvalue<Route>(
        &mut self,
        mut state: SuspendedAssociationSetLValue,
        selector: Expr,
        route: Route,
    ) -> Result<Expr, Route::Error>
    where
        Route: DefinitionResumeRoute<ResumeSuspendedAssociationSetLvalueSuspensionFrame>,
    {
        state.evaluated = state.evaluated.appended(selector);
        state.cursor += 1;

        loop {
            let Some(selector) = state.selectors.get(state.cursor) else {
                return self
                    .finish_association_set_function_lvalue(
                        state.symbol,
                        state.selectors.as_ref(),
                        state.evaluated.into_vec(),
                        state.rhs,
                    )
                    .map_err(Route::failure);
            };

            let selector_result = self.evaluate(selector.clone());
            let witness = slot_witness(&selector_result);
            match selector_result {
                Ok(selector) => {
                    state.evaluated = state.evaluated.appended(selector);
                    state.cursor += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    return resume_frames::suspend_suspended_association_set_lvalue(
                        self, state, witness, request, route,
                    );
                }
                Err(error) => return Err(Route::failure(error)),
            }
        }
    }

    fn finish_association_set_function_lvalue(
        &mut self,
        symbol: Symbol,
        raw_selectors: &[Expr],
        evaluated_selectors: Vec<Expr>,
        rhs: Expr,
    ) -> Result<Expr, EvalError> {
        if let Some(key) = direct_association_lvalue_key(raw_selectors, &evaluated_selectors) {
            if self.try_direct_association_rule_update(&symbol, key, &rhs)? {
                return Ok(rhs);
            }
        }

        let Some(current) = self.association_lvalue_current(&symbol) else {
            return Ok(rhs);
        };

        let update = run_with_native_stack_guard(self, |depth| {
            set_association_key_path(current, &evaluated_selectors, rhs.clone(), &depth)
        })?;

        match update {
            AssociationSetUpdate::Updated(updated) => {
                let definition = self.definitions.entry(symbol).or_default();
                definition.own_value = Some(updated);
                definition.delayed_own_value = None;
                self.bump_evaluation_epoch_for_symbol(&symbol);
            }
            AssociationSetUpdate::MissingKey(key) => {
                self.push_message(
                    "Set",
                    "kval",
                    format!("The value for the key {key} does not exist."),
                );
            }
            AssociationSetUpdate::Unchanged => {}
        }

        Ok(rhs)
    }

    fn finish_association_set_part_lvalue(
        &mut self,
        symbol: Symbol,
        direct_single_key: bool,
        evaluated_selectors: EvaluatedPartSelectors,
        rhs: Expr,
    ) -> Result<Expr, EvalError> {
        if direct_single_key {
            if let [selector] = evaluated_selectors.as_slice() {
                if let Some(key) = explicit_association_key_selector(selector) {
                    if self.try_direct_association_rule_update(&symbol, key.clone(), &rhs)? {
                        return Ok(rhs);
                    }
                }
            }
        }

        let Some(current) = self.association_lvalue_current(&symbol) else {
            return Ok(rhs);
        };
        let update = run_with_native_stack_guard(self, |depth| {
            set_association_part_path(current, &evaluated_selectors, rhs.clone(), &depth)
        })?;
        match update {
            AssociationSetUpdate::Updated(updated) => {
                let definition = self.definitions.entry(symbol).or_default();
                definition.own_value = Some(updated);
                definition.delayed_own_value = None;
                self.bump_evaluation_epoch_for_symbol(&symbol);
            }
            AssociationSetUpdate::MissingKey(key) => {
                self.push_message(
                    "Set",
                    "kval",
                    format!("The value for the key {key} does not exist."),
                );
            }
            AssociationSetUpdate::Unchanged => {}
        }

        Ok(rhs)
    }
}

fn direct_association_lvalue_key(
    raw_selectors: &[Expr],
    evaluated_selectors: &[Expr],
) -> Option<Expr> {
    let [selector] = evaluated_selectors else {
        return None;
    };

    (raw_selectors.len() == 1).then(|| selector.clone())
}

fn set_association_part_path(
    current: Expr,
    selectors: &EvaluatedPartSelectors,
    rhs: Expr,
    depth: &NativeStackDepth<'_>,
) -> AssociationSetUpdate {
    if let Some(keys) = association_part_key_path(selectors) {
        return set_association_key_path(current, &keys, rhs, depth);
    }

    AssociationSetUpdate::Unchanged
}

fn association_part_key_path(selectors: &EvaluatedPartSelectors) -> Option<Vec<Expr>> {
    selectors
        .as_slice()
        .iter()
        .map(association_part_key_selector)
        .collect()
}

fn explicit_association_key_selector(selector: &Expr) -> Option<&Expr> {
    match selector {
        Expr::Call { head, args } if is_system_symbol_name(head, "Key") && args.len() == 1 => {
            Some(&args[0])
        }
        _ => None,
    }
}

fn association_part_key_selector(selector: &Expr) -> Option<Expr> {
    if matches!(selector, Expr::Atom(Atom::String(_))) {
        return Some(selector.clone());
    }

    match selector {
        Expr::Call { head, args } if is_system_symbol_name(head, "Key") && args.len() == 1 => {
            Some(args[0].clone())
        }
        _ => None,
    }
}

fn set_association_key_path(
    current: Expr,
    selectors: &[Expr],
    rhs: Expr,
    depth: &NativeStackDepth<'_>,
) -> AssociationSetUpdate {
    let Some((key, rest)) = selectors.split_first() else {
        return AssociationSetUpdate::Unchanged;
    };

    let Some(mut entries) = association_entry_forms_from_expr(&current) else {
        return AssociationSetUpdate::Unchanged;
    };

    if rest.is_empty() {
        upsert_immediate_association_entry(&mut entries, key.clone(), rhs);
        return AssociationSetUpdate::Updated(render_association_entry_forms_preserving_heads(
            entries,
        ));
    }

    let Some(index) = entries.iter().position(|entry| entry.key == *key) else {
        return AssociationSetUpdate::MissingKey(key.clone());
    };

    let Some(child_depth) = depth.child() else {
        return AssociationSetUpdate::Unchanged;
    };
    match set_association_key_path(entries[index].value.clone(), rest, rhs, &child_depth) {
        AssociationSetUpdate::Updated(updated_child) => {
            entries[index].value = updated_child;
            entries[index].delayed = false;
            AssociationSetUpdate::Updated(render_association_entry_forms_preserving_heads(entries))
        }
        other => other,
    }
}

fn upsert_immediate_association_entry(
    entries: &mut Vec<AssociationEntryForm>,
    key: Expr,
    rhs: Expr,
) {
    let updated = AssociationEntryForm {
        key: key.clone(),
        value: rhs,
        delayed: false,
    };

    if let Some(index) = entries.iter().position(|entry| entry.key == key) {
        entries[index] = updated;
    } else {
        entries.push(updated);
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<AssociationEntryForm>();
    assert_traces::<AssociationSetLValueTarget>();
    assert_traces::<AssociationSelectorAccumulator>();
    assert_traces::<SuspendedAssociationSetLValue>();
};
