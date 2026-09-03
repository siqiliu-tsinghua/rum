use super::super::*;
use super::unique_symbols::UniqueSymbolCollector;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, Debug, PartialEq, Eq, rum_derive::FrameRootTrace)]
pub(in crate::runtime) enum TaggedDefinitionTarget {
    OwnValue(Symbol),
    DownValue(Symbol),
    SubValue(Symbol),
    UpValue(Symbol),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum TagDefinitionError {
    TagNotFound,
    TagPositionInvalid,
}

#[derive(Clone, Debug, PartialEq, Eq, rum_derive::FrameRootTrace)]
pub(in crate::runtime) enum DefinitionAssignmentTarget {
    Message(MessageNameKey),
    SyntaxInformation(Symbol),
    Definition(TaggedDefinitionTarget),
}

impl TaggedDefinitionTarget {
    pub(in crate::runtime) fn symbol(&self) -> &Symbol {
        match self {
            Self::OwnValue(symbol)
            | Self::DownValue(symbol)
            | Self::SubValue(symbol)
            | Self::UpValue(symbol) => symbol,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum DefinitionTargetNormalizationError {
    InvalidClearTarget,
    InvalidTagTarget,
    InvalidUnsetTarget,
}

impl DefinitionTargetNormalizationError {
    pub(in crate::runtime) fn message(self) -> &'static str {
        match self {
            Self::InvalidClearTarget => {
                "expected one or more symbols, symbol-name strings, or lists of them"
            }
            Self::InvalidTagTarget => {
                "expected a symbol or symbol-name string tag as the first argument"
            }
            Self::InvalidUnsetTarget => {
                "expected a symbol, definition lhs, message name, or tagged definition target"
            }
        }
    }
}

pub(in crate::runtime) fn collect_with_binding_symbols(expr: &Expr, locals: &mut BTreeSet<Symbol>) {
    let Expr::Call { head, args } = expr else {
        return;
    };

    if !is_system_symbol_name(head, "List") {
        return;
    }

    for item in args {
        if let Some((symbol, _)) = parse_module_binding_item(item) {
            locals.insert(symbol);
        }
    }
}

pub(in crate::runtime) fn default_value_lhs_targets_symbol(lhs: &Expr, symbol: &Symbol) -> bool {
    let Expr::Call { head, args } = lhs else {
        return false;
    };

    is_system_symbol_name(head, "Default")
        && matches!(args.first(), Some(Expr::Atom(Atom::Symbol(target))) if target == symbol)
}

impl Session {
    pub(in crate::runtime) fn normalize_tag_target(
        &self,
        expr: &Expr,
    ) -> Result<Symbol, DefinitionTargetNormalizationError> {
        self.normalize_symbol_target_expr(expr, true)
            .ok_or(DefinitionTargetNormalizationError::InvalidTagTarget)
    }

    pub(in crate::runtime) fn normalize_clear_targets(
        &self,
        args: &[Expr],
    ) -> Result<Vec<Symbol>, DefinitionTargetNormalizationError> {
        if args.is_empty() {
            return Err(DefinitionTargetNormalizationError::InvalidClearTarget);
        }

        let mut symbols = Vec::new();

        for arg in args {
            let targets = self
                .normalize_clear_target_expr(arg)
                .ok_or(DefinitionTargetNormalizationError::InvalidClearTarget)?;
            symbols.extend(targets);
        }

        Ok(symbols)
    }

    fn normalize_clear_target_expr(&self, expr: &Expr) -> Option<Vec<Symbol>> {
        match expr {
            Expr::Atom(Atom::String(pattern)) if pattern.contains('*') => {
                self.resolve_names_matching_expr(expr)
            }
            Expr::Call { head, args } if is_system_symbol_name(head, "List") => args
                .iter()
                .map(|item| self.normalize_clear_target_expr(item))
                .collect::<Option<Vec<_>>>()
                .map(|items| items.into_iter().flatten().collect()),
            _ => self.normalize_symbol_target_exprs(expr, true),
        }
    }

    pub(in crate::runtime) fn normalize_definition_assignment_target(
        &self,
        lhs: &Expr,
    ) -> Option<DefinitionAssignmentTarget> {
        if let Some(key) = self.message_name_key_from_expr(lhs) {
            return Some(DefinitionAssignmentTarget::Message(key));
        }

        if let Some(symbol) = self.normalize_syntax_information_target_expr(lhs) {
            return Some(DefinitionAssignmentTarget::SyntaxInformation(symbol));
        }

        if let Some(symbol) = expr_as_symbol(lhs).cloned() {
            return Some(DefinitionAssignmentTarget::Definition(
                TaggedDefinitionTarget::OwnValue(symbol),
            ));
        }

        classify_definition_lhs(lhs).map(DefinitionAssignmentTarget::Definition)
    }

    /// Returns the immediate own value stored for a symbol.
    pub fn definition_own_value(&self, symbol: &Symbol) -> Option<Expr> {
        self.definitions
            .get(symbol)
            .and_then(|definition| definition.own_value.clone())
    }

    pub(in crate::runtime) fn normalize_options_target_expr(&self, expr: &Expr) -> Option<Symbol> {
        let Expr::Call { head, args } = expr else {
            return None;
        };

        if !is_system_symbol_name(head, "Options") || args.len() != 1 {
            return None;
        }

        self.normalize_symbol_target_expr(&args[0], true)
    }

    pub(in crate::runtime) fn normalize_syntax_information_target_expr(
        &self,
        expr: &Expr,
    ) -> Option<Symbol> {
        let Expr::Call { head, args } = expr else {
            return None;
        };

        if !is_system_symbol_name(head, "SyntaxInformation") || args.len() != 1 {
            return None;
        }

        self.normalize_symbol_target_expr(&args[0], true)
    }
}

pub(in crate::runtime) fn classify_definition_lhs(lhs: &Expr) -> Option<TaggedDefinitionTarget> {
    if let Some(symbol) = expr_as_symbol(lhs).cloned() {
        return Some(TaggedDefinitionTarget::OwnValue(symbol));
    }

    if let Some(symbol) = subvalue_base_symbol(lhs).cloned() {
        return Some(TaggedDefinitionTarget::SubValue(symbol));
    }

    downvalue_head_symbol(lhs)
        .cloned()
        .map(TaggedDefinitionTarget::DownValue)
}

pub(in crate::runtime) fn classify_tagged_definition(
    tag: &Symbol,
    lhs: &Expr,
) -> Result<TaggedDefinitionTarget, TagDefinitionError> {
    if let Expr::Call { head, args } = strip_hold_pattern(lhs) {
        if is_system_symbol_name(head, "Condition") && !args.is_empty() {
            return classify_tagged_definition(tag, &args[0]);
        }
    }

    if expr_as_symbol(lhs).is_some_and(|symbol| symbol == tag) {
        return Ok(TaggedDefinitionTarget::OwnValue(tag.clone()));
    }

    if subvalue_base_symbol(lhs).is_some_and(|symbol| symbol == tag) {
        return Ok(TaggedDefinitionTarget::SubValue(tag.clone()));
    }

    if downvalue_head_symbol(lhs).is_some_and(|symbol| symbol == tag) {
        return Ok(TaggedDefinitionTarget::DownValue(tag.clone()));
    }

    if matches!(strip_hold_pattern(lhs), Expr::Call { args, .. } if args.iter().any(|arg| valid_upvalue_tag_position(arg, tag)))
    {
        return Ok(TaggedDefinitionTarget::UpValue(tag.clone()));
    }

    if expr_contains_symbol(lhs, tag) {
        Err(TagDefinitionError::TagPositionInvalid)
    } else {
        Err(TagDefinitionError::TagNotFound)
    }
}

pub(in crate::runtime) fn valid_upvalue_tag_position(expr: &Expr, tag: &Symbol) -> bool {
    upvalue_tag_symbol_at_position(expr).is_some_and(|symbol| symbol == tag)
}

fn upvalue_tag_symbol_at_position(expr: &Expr) -> Option<&Symbol> {
    expr_as_symbol(expr).or_else(|| match expr {
        Expr::Call { head, .. } => expr_as_symbol(head),
        _ => None,
    })
}

pub(in crate::runtime) fn expr_contains_symbol(expr: &Expr, target: &Symbol) -> bool {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => symbol == target,
        Expr::Atom(_) => false,
        Expr::Call { head, args } => {
            expr_contains_symbol(head, target)
                || args.iter().any(|arg| expr_contains_symbol(arg, target))
        }
    }
}

pub(in crate::runtime) fn collect_upvalue_candidate_symbols(args: &[Expr]) -> Vec<Symbol> {
    let mut symbols = UniqueSymbolCollector::with_capacity_hint(args.len());
    let mut has_nested_argument = false;

    for arg in args {
        has_nested_argument |= matches!(arg, Expr::Call { .. });
        if let Some(symbol) = upvalue_tag_symbol_at_position(arg) {
            symbols.push(*symbol);
        }
    }

    let candidate_symbols = symbols.into_symbols();
    if candidate_symbols.len() < 2 || !has_nested_argument {
        return candidate_symbols;
    }

    preserve_upvalue_candidate_matching_order(args, candidate_symbols)
}

#[cold]
fn preserve_upvalue_candidate_matching_order(
    args: &[Expr],
    candidate_symbols: Vec<Symbol>,
) -> Vec<Symbol> {
    // Candidate membership comes only from valid tag positions. Preserve the
    // established first-seen matching order when one of those same symbols
    // also appeared earlier at a deeper, non-candidate position.
    let candidate_index = (candidate_symbols.len() > 16).then(|| {
        candidate_symbols
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>()
    });
    let mut ordered_symbols = UniqueSymbolCollector::with_capacity_hint(candidate_symbols.len());
    for arg in args {
        collect_upvalue_candidate_symbols_in_matching_order(
            arg,
            &candidate_symbols,
            candidate_index.as_ref(),
            &mut ordered_symbols,
        );
    }
    ordered_symbols.into_symbols()
}

fn collect_upvalue_candidate_symbols_in_matching_order(
    expr: &Expr,
    candidate_symbols: &[Symbol],
    candidate_index: Option<&std::collections::HashSet<Symbol>>,
    ordered_symbols: &mut UniqueSymbolCollector,
) {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => {
            let is_candidate = candidate_index.map_or_else(
                || candidate_symbols.contains(symbol),
                |index| index.contains(symbol),
            );
            if is_candidate {
                ordered_symbols.push(*symbol);
            }
        }
        Expr::Atom(_) => {}
        Expr::Call { head, args } => {
            collect_upvalue_candidate_symbols_in_matching_order(
                head,
                candidate_symbols,
                candidate_index,
                ordered_symbols,
            );
            for arg in args {
                collect_upvalue_candidate_symbols_in_matching_order(
                    arg,
                    candidate_symbols,
                    candidate_index,
                    ordered_symbols,
                );
            }
        }
    }
}

pub(in crate::runtime) fn collect_upset_target_symbols(lhs: &Expr) -> Vec<Symbol> {
    let lhs = strip_hold_pattern(lhs);
    let capacity_hint = match lhs {
        Expr::Call { args, .. } => args.len(),
        Expr::Atom(_) => 0,
    };
    let mut symbols = UniqueSymbolCollector::with_capacity_hint(capacity_hint);
    collect_upset_target_symbols_from_lhs(lhs, &mut symbols);
    symbols.into_symbols()
}

fn collect_upset_target_symbols_from_lhs(expr: &Expr, symbols: &mut UniqueSymbolCollector) {
    if let Expr::Call { head, args } = expr {
        if is_system_symbol_name(head, "Condition") && !args.is_empty() {
            collect_upset_target_symbols_from_lhs(&args[0], symbols);
            return;
        }

        for arg in args {
            collect_upset_target_symbol_from_arg(arg, symbols);
        }
    }
}

fn collect_upset_target_symbol_from_arg(expr: &Expr, symbols: &mut UniqueSymbolCollector) {
    if let Some((_, inner)) = pattern_named_pattern(expr) {
        collect_upset_target_symbol_from_arg(inner, symbols);
        return;
    }

    if let Some((inner, _)) = pattern_condition(expr) {
        collect_upset_target_symbol_from_arg(inner, symbols);
        return;
    }

    if let Some((inner, _)) = pattern_optional(expr) {
        collect_upset_target_symbol_from_arg(inner, symbols);
        return;
    }

    if pattern_blank_head(expr).is_some() {
        return;
    }

    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => symbols.push(*symbol),
        Expr::Call { head, .. }
            if is_system_symbol_name(head, "BlankSequence")
                || is_system_symbol_name(head, "BlankNullSequence") => {}
        Expr::Call { head, .. } => {
            if let Some(symbol) = expr_as_symbol(head) {
                symbols.push(*symbol);
            }
        }
        Expr::Atom(_) => {}
    }
}

pub(in crate::runtime) fn lhs_patterns_equivalent(lhs: &Expr, rhs: &Expr) -> bool {
    canonicalize_pattern_names(lhs) == canonicalize_pattern_names(rhs)
}

pub(in crate::runtime) fn canonicalize_pattern_names(expr: &Expr) -> Expr {
    let mut mapping = BTreeMap::new();
    let mut next_id = 0_u64;
    collect_pattern_name_mapping(expr, &mut mapping, &mut next_id);
    substitute_pattern_name_mapping(expr, &mapping)
}

pub(in crate::runtime) fn collect_pattern_name_mapping(
    expr: &Expr,
    mapping: &mut BTreeMap<Symbol, Symbol>,
    next_id: &mut u64,
) {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            if let Some(symbol) = expr_as_symbol(&args[0]) {
                mapping.entry(symbol.clone()).or_insert_with(|| {
                    *next_id += 1;
                    Symbol::new("Pattern`", format!("p{next_id}"))
                });
            }

            collect_pattern_name_mapping(&args[1], mapping, next_id);
        }
        Expr::Call { head, args } => {
            collect_pattern_name_mapping(head, mapping, next_id);

            for arg in args {
                collect_pattern_name_mapping(arg, mapping, next_id);
            }
        }
        Expr::Atom(_) => {}
    }
}

pub(in crate::runtime) fn substitute_pattern_name_mapping(
    expr: &Expr,
    mapping: &BTreeMap<Symbol, Symbol>,
) -> Expr {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => mapping
            .get(symbol)
            .cloned()
            .map(|symbol| Expr::Atom(Atom::Symbol(symbol)))
            .unwrap_or_else(|| Expr::Atom(Atom::Symbol(symbol.clone()))),
        Expr::Atom(atom) => Expr::Atom(atom.clone()),
        Expr::Call { head, args } => Expr::call(
            substitute_pattern_name_mapping(head, mapping),
            args.iter()
                .map(|arg| substitute_pattern_name_mapping(arg, mapping))
                .collect(),
        ),
    }
}

pub(in crate::runtime) fn downvalue_head_symbol(expr: &Expr) -> Option<&Symbol> {
    match strip_hold_pattern(expr) {
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && !args.is_empty() =>
        {
            downvalue_head_symbol(&args[0])
        }
        Expr::Call { head, .. } => expr_as_symbol(head),
        _ => None,
    }
}

pub(in crate::runtime) fn subvalue_base_symbol(expr: &Expr) -> Option<&Symbol> {
    match strip_hold_pattern(expr) {
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && !args.is_empty() =>
        {
            subvalue_base_symbol(&args[0])
        }
        Expr::Call { head, .. } => base_head_symbol_of_nested_call(head),
        _ => None,
    }
}

pub(in crate::runtime) fn base_head_symbol_of_nested_call(expr: &Expr) -> Option<&Symbol> {
    match expr {
        Expr::Call { head, .. } => match &**head {
            Expr::Call { .. } => base_head_symbol_of_nested_call(head),
            _ => expr_as_symbol(head),
        },
        _ => None,
    }
}

pub(in crate::runtime) fn normalize_rewrite_rules(expr: &Expr) -> Option<Vec<RewriteRule>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            let mut rules = Vec::new();

            for item in args {
                let rule = normalize_single_rewrite_rule(item)?;
                rules.push(rule);
            }

            Some(rules)
        }
        _ => normalize_single_rewrite_rule(expr).map(|rule| vec![rule]),
    }
}

pub(in crate::runtime) fn normalize_single_rewrite_rule(expr: &Expr) -> Option<RewriteRule> {
    match expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            let delayed = is_system_symbol_name(head, "RuleDelayed");
            let (rhs, guards) = split_definition_rhs_guards(args[1].clone());
            let rhs_may_splice_without_bindings = rule_rhs_may_splice_without_bindings(&rhs);

            Some(RewriteRule {
                lhs: args[0].clone(),
                rhs,
                delayed,
                rhs_may_splice_without_bindings,
                matcher: compile_matcher(&args[0]),
                guards,
            })
        }
        _ => None,
    }
}
