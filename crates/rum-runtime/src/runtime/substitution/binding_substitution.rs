use super::super::*;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};

mod iterative;
use iterative::{
    substitute_bindings_changed_iterative, substitute_rule_bindings_changed_iterative,
    substitute_rule_with_binding_item_changed_iterative,
    substitute_rule_with_binding_list_changed_iterative,
};

const RECURSIVE_BINDING_SUBSTITUTION_CALL_DEPTH: usize = 16;

const SUBSTITUTION_SHARING_PROFILE_ENV: &str = "RUM_PROFILE_SUBSTITUTION_SHARING";

static SUBSTITUTION_SHARING_PROFILE_ENABLED: AtomicU8 = AtomicU8::new(0);
static RULE_RHS_SUBSTITUTION_REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_FAST_CLONE_EMPTY_BINDINGS_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_BINDING_HIT_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_BINDING_CALL_HIT_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_BINDING_ATOM_HIT_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_REBUILT_REUSE_SHARED_ARGS_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_REBUILT_REUSE_SHARED_HEAD_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_REBUILT_FRESH_HEAD_ARGS_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_PRESERVED_EXISTING_ARG_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_PRESERVED_EXISTING_CALL_ARG_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_SPLICE_FROM_BINDING_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_ALPHA_RENAME_ID: AtomicU64 = AtomicU64::new(0);

const RULE_RHS_ALPHA_ID_MARKER: u64 = 1_u64 << 63;

pub(in crate::runtime) fn bind_pattern_name(
    name: &Symbol,
    value: Expr,
    bindings: &mut SymbolExprMap,
) -> bool {
    match bindings.get(name) {
        Some(existing) => existing == &value,
        None => {
            bindings.insert(name.clone(), value);
            true
        }
    }
}

pub(in crate::runtime) fn sequence_expr(values: Vec<Expr>) -> Expr {
    Expr::call(system_expr("Sequence"), values)
}

fn record_rule_rhs_rebuild(source: &'static str, expr: Expr) -> Expr {
    let start = rule_rhs_timing_start();
    record_profile_expression_rebuild(source, &expr);
    record_profile_rule_rhs_timing("record_rebuild", start, None);
    expr
}

pub(in crate::runtime) fn substitute_bindings(expr: &Expr, bindings: &SymbolExprMap) -> Expr {
    substitute_bindings_changed(expr, bindings).unwrap_or_else(|| expr.clone())
}

fn substitute_bindings_changed(expr: &Expr, bindings: &SymbolExprMap) -> Option<Expr> {
    substitute_bindings_changed_bounded(expr, bindings, RECURSIVE_BINDING_SUBSTITUTION_CALL_DEPTH)
}

fn substitute_bindings_changed_bounded(
    expr: &Expr,
    bindings: &SymbolExprMap,
    remaining_call_depth: usize,
) -> Option<Expr> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => bindings.get(symbol).cloned(),
        Expr::Atom(_) => None,
        Expr::Call { head, args } => {
            if remaining_call_depth == 0 {
                return substitute_bindings_changed_iterative(expr, bindings);
            }
            let child_call_depth = remaining_call_depth - 1;
            let substituted_head =
                substitute_bindings_changed_bounded(head, bindings, child_call_depth);
            let mut substituted_args = None;

            for (index, arg) in args.iter().enumerate() {
                if let Some(substituted_arg) =
                    substitute_bindings_changed_bounded(arg, bindings, child_call_depth)
                {
                    substituted_args
                        .get_or_insert_with(|| args[..index].to_vec())
                        .push(substituted_arg);
                } else if let Some(substituted_args) = &mut substituted_args {
                    substituted_args.push(arg.clone());
                }
            }

            match (substituted_head, substituted_args) {
                (None, None) => None,
                (Some(head), None) => Some(Expr::call_with_shared_args(head, args.clone())),
                (None, Some(args)) => Some(Expr::call_shared(head.clone(), SharedArgs::new(args))),
                (Some(head), Some(args)) => Some(Expr::call(head, args)),
            }
        }
    }
}

pub(in crate::runtime) fn substitute_rule_bindings(expr: &Expr, bindings: &SymbolExprMap) -> Expr {
    let start = rule_rhs_timing_start();
    let substituted = substitute_rule_bindings_changed(expr, bindings, &SymbolExprMap::default());
    let changed = substituted.is_some();
    let result = substituted.unwrap_or_else(|| expr.clone());
    record_profile_rule_rhs_timing("top_substitute", start, Some(changed));
    result
}

pub(in crate::runtime) fn substitute_rule_bindings_for_rhs(
    expr: &Expr,
    bindings: &SymbolExprMap,
    _may_splice_without_bindings: bool,
) -> Expr {
    let start = rule_rhs_timing_start();
    record_substitution_sharing_counter(
        &RULE_RHS_SUBSTITUTION_REQUEST_COUNT,
        "rule_rhs_substitution_requests",
    );
    let result = if bindings.is_empty() {
        record_substitution_sharing_counter(
            &RULE_RHS_FAST_CLONE_EMPTY_BINDINGS_COUNT,
            "rule_rhs_fast_clone_empty_bindings",
        );
        let result = expr.clone();
        record_profile_rule_rhs_timing("fast_clone_empty", rule_rhs_timing_start(), Some(false));
        result
    } else {
        substitute_rule_bindings(expr, bindings)
    };
    record_profile_rule_rhs_timing("request", start, None);
    result
}

pub(in crate::runtime) fn rule_rhs_may_splice_without_bindings(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, args } => {
            rule_rhs_may_splice_without_bindings(head)
                || args.iter().any(|arg| {
                    sequence_items(arg).is_some() || rule_rhs_may_splice_without_bindings(arg)
                })
        }
        Expr::Atom(_) => false,
    }
}

fn substitute_rule_bindings_changed(
    expr: &Expr,
    bindings: &SymbolExprMap,
    alpha_renames: &SymbolExprMap,
) -> Option<Expr> {
    substitute_rule_bindings_changed_bounded(
        expr,
        bindings,
        alpha_renames,
        RECURSIVE_BINDING_SUBSTITUTION_CALL_DEPTH,
    )
}

fn substitute_rule_bindings_changed_bounded(
    expr: &Expr,
    bindings: &SymbolExprMap,
    alpha_renames: &SymbolExprMap,
    remaining_call_depth: usize,
) -> Option<Expr> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => {
            if let Some(value) = bindings.get(symbol) {
                record_rule_rhs_binding_hit(value);
                record_profile_rule_rhs_binding_value(value);
                Some(value.clone())
            } else if let Some(value) = alpha_renames.get(symbol) {
                record_rule_rhs_binding_hit(value);
                Some(value.clone())
            } else {
                None
            }
        }
        Expr::Atom(_) => None,
        Expr::Call { head, args } => {
            if remaining_call_depth == 0 {
                return substitute_rule_bindings_changed_iterative(expr, bindings, alpha_renames);
            }
            let child_call_depth = remaining_call_depth - 1;

            if is_system_symbol_name(head, "Module") && args.len() == 2 {
                substitute_rule_scoped_call_changed(
                    head,
                    args,
                    bindings,
                    alpha_renames,
                    child_call_depth,
                )
            } else if is_system_symbol_name(head, "With") && args.len() == 2 {
                substitute_rule_with_call_changed(
                    head,
                    args,
                    bindings,
                    alpha_renames,
                    child_call_depth,
                )
            } else if is_system_symbol_name(head, "Function") {
                substitute_rule_function_call_changed(
                    head,
                    args,
                    bindings,
                    alpha_renames,
                    child_call_depth,
                )
            } else {
                substitute_rule_call_changed_with_arg_alphas(
                    head,
                    args,
                    bindings,
                    alpha_renames,
                    |_| alpha_renames,
                    child_call_depth,
                )
            }
        }
    }
}

fn substitute_rule_scoped_call_changed(
    head: &Expr,
    args: &SharedArgs,
    bindings: &SymbolExprMap,
    alpha_renames: &SymbolExprMap,
    remaining_call_depth: usize,
) -> Option<Expr> {
    let start = rule_rhs_timing_start();
    let Some(local_symbols) = parse_module_binding_list(&args[0]).map(scoped_binding_symbols)
    else {
        let result = substitute_rule_call_changed_with_arg_alphas(
            head,
            args,
            bindings,
            alpha_renames,
            |_| alpha_renames,
            remaining_call_depth,
        );
        record_profile_rule_rhs_timing("scoped_call", start, Some(result.is_some()));
        return result;
    };
    let scoped_alpha_renames =
        alpha_renames_with_fresh_locals(alpha_renames, bindings, local_symbols);

    let result = substitute_rule_call_changed_with_arg_alphas(
        head,
        args,
        bindings,
        alpha_renames,
        |_| &scoped_alpha_renames,
        remaining_call_depth,
    );
    record_profile_rule_rhs_timing("scoped_call", start, Some(result.is_some()));
    result
}

fn substitute_rule_with_call_changed(
    head: &Expr,
    args: &SharedArgs,
    bindings: &SymbolExprMap,
    alpha_renames: &SymbolExprMap,
    remaining_call_depth: usize,
) -> Option<Expr> {
    let start = rule_rhs_timing_start();
    let Some(local_symbols) = parse_with_binding_list(&args[0]).map(with_binding_symbols) else {
        let result = substitute_rule_call_changed_with_arg_alphas(
            head,
            args,
            bindings,
            alpha_renames,
            |_| alpha_renames,
            remaining_call_depth,
        );
        record_profile_rule_rhs_timing("with_call", start, Some(result.is_some()));
        return result;
    };
    let scoped_alpha_renames =
        alpha_renames_with_fresh_locals(alpha_renames, bindings, local_symbols);

    let substituted_head = substitute_rule_bindings_changed_bounded(
        head,
        bindings,
        alpha_renames,
        remaining_call_depth,
    );
    let substituted_bindings = substitute_rule_with_binding_list_changed(
        &args[0],
        bindings,
        alpha_renames,
        &scoped_alpha_renames,
        remaining_call_depth,
    );
    let substituted_body = substitute_rule_bindings_changed_bounded(
        &args[1],
        bindings,
        &scoped_alpha_renames,
        remaining_call_depth,
    );

    if substituted_head.is_none() && substituted_bindings.is_none() && substituted_body.is_none() {
        record_profile_rule_rhs_timing("with_call", start, Some(false));
        return None;
    }

    let result = Some(record_rule_rhs_rebuild(
        "rule_rhs_with_call",
        Expr::call(
            substituted_head.unwrap_or_else(|| head.clone()),
            vec![
                substituted_bindings.unwrap_or_else(|| args[0].clone()),
                substituted_body.unwrap_or_else(|| args[1].clone()),
            ],
        ),
    ));
    record_profile_rule_rhs_timing("with_call", start, Some(true));
    result
}

fn substitute_rule_function_call_changed(
    head: &Expr,
    args: &SharedArgs,
    bindings: &SymbolExprMap,
    alpha_renames: &SymbolExprMap,
    remaining_call_depth: usize,
) -> Option<Expr> {
    let start = rule_rhs_timing_start();
    let Some(params) = explicit_function_parameters_from_args(args) else {
        let result = substitute_rule_call_changed_with_arg_alphas(
            head,
            args,
            bindings,
            alpha_renames,
            |_| alpha_renames,
            remaining_call_depth,
        );
        record_profile_rule_rhs_timing("function_call", start, Some(result.is_some()));
        return result;
    };
    let scoped_alpha_renames = alpha_renames_with_fresh_locals(alpha_renames, bindings, params);

    let result = substitute_rule_call_changed_with_arg_alphas(
        head,
        args,
        bindings,
        alpha_renames,
        |index| {
            if index <= 1 {
                &scoped_alpha_renames
            } else {
                alpha_renames
            }
        },
        remaining_call_depth,
    );
    record_profile_rule_rhs_timing("function_call", start, Some(result.is_some()));
    result
}

fn substitute_rule_call_changed_with_arg_alphas<'a, F>(
    head: &Expr,
    args: &SharedArgs,
    bindings: &SymbolExprMap,
    head_alpha_renames: &SymbolExprMap,
    mut arg_alpha_renames: F,
    remaining_call_depth: usize,
) -> Option<Expr>
where
    F: FnMut(usize) -> &'a SymbolExprMap,
{
    let start = rule_rhs_timing_start();
    let substituted_head = substitute_rule_bindings_changed_bounded(
        head,
        bindings,
        head_alpha_renames,
        remaining_call_depth,
    );
    let mut substituted_args = None;

    for (index, arg) in args.iter().enumerate() {
        let arg_alpha_renames = arg_alpha_renames(index);
        if let Some(substituted_arg) = substitute_rule_bindings_changed_bounded(
            arg,
            bindings,
            arg_alpha_renames,
            remaining_call_depth,
        ) {
            let output_args = substituted_args.get_or_insert_with(|| {
                record_rule_rhs_preserved_args(&args[..index]);
                args[..index].to_vec()
            });
            // Structural splicing belongs only to a Sequence value inserted by
            // a directly bound pattern symbol. A literal Sequence call in the
            // RHS remains one argument, even when substitution rebuilt it.
            let bound_sequence_items = if matches!(arg, Expr::Atom(Atom::Symbol(_))) {
                sequence_items(&substituted_arg)
            } else {
                None
            };
            if let Some(sequence_items) = bound_sequence_items {
                record_substitution_sharing_counter(
                    &RULE_RHS_SPLICE_FROM_BINDING_COUNT,
                    "rule_rhs_splices_from_binding",
                );
                output_args.extend(sequence_items.iter().cloned());
            } else {
                output_args.push(substituted_arg);
            }
        } else if let Some(substituted_args) = &mut substituted_args {
            record_rule_rhs_preserved_arg(arg);
            substituted_args.push(arg.clone());
        }
    }

    let result = rebuild_rule_call(head, args, substituted_head, substituted_args);
    record_profile_rule_rhs_timing("call_args", start, Some(result.is_some()));
    result
}

fn substitute_rule_with_binding_list_changed(
    expr: &Expr,
    bindings: &SymbolExprMap,
    alpha_renames: &SymbolExprMap,
    scoped_alpha_renames: &SymbolExprMap,
    remaining_call_depth: usize,
) -> Option<Expr> {
    if remaining_call_depth == 0 && matches!(expr, Expr::Call { .. }) {
        return substitute_rule_with_binding_list_changed_iterative(
            expr,
            bindings,
            alpha_renames,
            scoped_alpha_renames,
        );
    }

    let start = rule_rhs_timing_start();
    let Expr::Call { head, args } = expr else {
        let result = substitute_rule_bindings_changed_bounded(
            expr,
            bindings,
            alpha_renames,
            remaining_call_depth,
        );
        record_profile_rule_rhs_timing("with_binding_list", start, Some(result.is_some()));
        return result;
    };

    if !is_system_symbol_name(head, "List") {
        let result = substitute_rule_bindings_changed_bounded(
            expr,
            bindings,
            alpha_renames,
            remaining_call_depth,
        );
        record_profile_rule_rhs_timing("with_binding_list", start, Some(result.is_some()));
        return result;
    }

    let child_call_depth = remaining_call_depth.saturating_sub(1);
    let substituted_head =
        substitute_rule_bindings_changed_bounded(head, bindings, alpha_renames, child_call_depth);
    let mut substituted_items = None;
    let mut item_rhs_alpha_renames = alpha_renames.clone();

    for (index, item) in args.iter().enumerate() {
        let substituted_item = substitute_rule_with_binding_item_changed(
            item,
            bindings,
            &item_rhs_alpha_renames,
            scoped_alpha_renames,
            child_call_depth,
        );

        if let Some(substituted_item) = substituted_item {
            substituted_items
                .get_or_insert_with(|| args[..index].to_vec())
                .push(substituted_item);
        } else if let Some(substituted_items) = &mut substituted_items {
            substituted_items.push(item.clone());
        }

        if let Some((symbol, _)) = parse_with_binding_item(item) {
            if let Some(alpha) = scoped_alpha_renames.get(&symbol) {
                item_rhs_alpha_renames.insert(symbol, alpha.clone());
            }
        }
    }

    let result = match (substituted_head, substituted_items) {
        (None, None) => None,
        (Some(head), None) => Some(record_rule_rhs_rebuild(
            "rule_rhs_with_binding_list",
            Expr::call_with_shared_args(head, args.clone()),
        )),
        (None, Some(args)) => Some(record_rule_rhs_rebuild(
            "rule_rhs_with_binding_list",
            Expr::call_shared(
                SharedExpr::from(head.as_ref().clone()),
                SharedArgs::new(args),
            ),
        )),
        (Some(head), Some(args)) => Some(record_rule_rhs_rebuild(
            "rule_rhs_with_binding_list",
            Expr::call(head, args),
        )),
    };
    record_profile_rule_rhs_timing("with_binding_list", start, Some(result.is_some()));
    result
}

fn substitute_rule_with_binding_item_changed(
    expr: &Expr,
    bindings: &SymbolExprMap,
    rhs_alpha_renames: &SymbolExprMap,
    scoped_alpha_renames: &SymbolExprMap,
    remaining_call_depth: usize,
) -> Option<Expr> {
    if remaining_call_depth == 0 && matches!(expr, Expr::Call { .. }) {
        return substitute_rule_with_binding_item_changed_iterative(
            expr,
            bindings,
            rhs_alpha_renames,
            scoped_alpha_renames,
        );
    }

    let start = rule_rhs_timing_start();
    match expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Set") || is_system_symbol_name(head, "Rule"))
                && args.len() == 2
                && expr_as_symbol(&args[0]).is_some() =>
        {
            let child_call_depth = remaining_call_depth.saturating_sub(1);
            let substituted_head = substitute_rule_bindings_changed_bounded(
                head,
                bindings,
                rhs_alpha_renames,
                child_call_depth,
            );
            let substituted_lhs = substitute_rule_bindings_changed_bounded(
                &args[0],
                bindings,
                scoped_alpha_renames,
                child_call_depth,
            );
            let substituted_rhs = substitute_rule_bindings_changed_bounded(
                &args[1],
                bindings,
                rhs_alpha_renames,
                child_call_depth,
            );

            if substituted_head.is_none() && substituted_lhs.is_none() && substituted_rhs.is_none()
            {
                record_profile_rule_rhs_timing("with_binding_item", start, Some(false));
                return None;
            }

            let result = Some(record_rule_rhs_rebuild(
                "rule_rhs_with_binding_item",
                Expr::call(
                    substituted_head.unwrap_or_else(|| head.as_ref().clone()),
                    vec![
                        substituted_lhs.unwrap_or_else(|| args[0].clone()),
                        substituted_rhs.unwrap_or_else(|| args[1].clone()),
                    ],
                ),
            ));
            record_profile_rule_rhs_timing("with_binding_item", start, Some(true));
            result
        }
        _ => {
            let result = substitute_rule_bindings_changed_bounded(
                expr,
                bindings,
                rhs_alpha_renames,
                remaining_call_depth,
            );
            record_profile_rule_rhs_timing("with_binding_item", start, Some(result.is_some()));
            result
        }
    }
}

fn rebuild_rule_call(
    head: &Expr,
    original_args: &SharedArgs,
    substituted_head: Option<Expr>,
    substituted_args: Option<Vec<Expr>>,
) -> Option<Expr> {
    let start = rule_rhs_timing_start();
    let result = match (substituted_head, substituted_args) {
        (None, None) => None,
        (Some(head), None) => {
            record_substitution_sharing_counter(
                &RULE_RHS_REBUILT_REUSE_SHARED_ARGS_COUNT,
                "rule_rhs_rebuilt_reuse_shared_args",
            );
            Some(record_rule_rhs_rebuild(
                "rule_rhs_call",
                Expr::call_with_shared_args(head, original_args.clone()),
            ))
        }
        (None, Some(args)) => {
            record_substitution_sharing_counter(
                &RULE_RHS_REBUILT_REUSE_SHARED_HEAD_COUNT,
                "rule_rhs_rebuilt_reuse_shared_head",
            );
            Some(record_rule_rhs_rebuild(
                "rule_rhs_call",
                Expr::call_shared(SharedExpr::from(head.clone()), SharedArgs::new(args)),
            ))
        }
        (Some(head), Some(args)) => {
            record_substitution_sharing_counter(
                &RULE_RHS_REBUILT_FRESH_HEAD_ARGS_COUNT,
                "rule_rhs_rebuilt_fresh_head_args",
            );
            Some(record_rule_rhs_rebuild(
                "rule_rhs_call",
                Expr::call(head, args),
            ))
        }
    };
    record_profile_rule_rhs_timing("rebuild_call", start, Some(result.is_some()));
    result
}

fn scoped_binding_symbols(items: Vec<(Symbol, Option<Expr>)>) -> Vec<Symbol> {
    items.into_iter().map(|(symbol, _)| symbol).collect()
}

fn with_binding_symbols(items: Vec<(Symbol, Expr)>) -> Vec<Symbol> {
    items.into_iter().map(|(symbol, _)| symbol).collect()
}

fn explicit_function_parameters_from_args(args: &[Expr]) -> Option<Vec<Symbol>> {
    match args {
        [params, _body] => parse_function_parameters(params),
        [params, _body, _attributes] => parse_function_parameters(params),
        _ => None,
    }
}

fn alpha_renames_with_fresh_locals(
    alpha_renames: &SymbolExprMap,
    bindings: &SymbolExprMap,
    locals: Vec<Symbol>,
) -> SymbolExprMap {
    let start = rule_rhs_timing_start();
    let mut scoped = alpha_renames.clone();
    for local in locals {
        if bindings.contains_key(&local) {
            continue;
        }
        let alpha = fresh_rule_rhs_alpha_symbol(&local);
        record_profile_rule_rhs_alpha_rename(&local, &alpha);
        scoped.insert(local, Expr::Atom(Atom::Symbol(alpha)));
    }
    record_profile_rule_rhs_timing("alpha_renames", start, None);
    scoped
}

fn fresh_rule_rhs_alpha_symbol(symbol: &Symbol) -> Symbol {
    let id = RULE_RHS_ALPHA_RENAME_ID.fetch_add(1, Ordering::Relaxed) + 1;
    Symbol::with_identity(
        symbol.context(),
        symbol.name(),
        RULE_RHS_ALPHA_ID_MARKER | id,
    )
}

fn record_rule_rhs_binding_hit(value: &Expr) {
    record_substitution_sharing_counter(&RULE_RHS_BINDING_HIT_COUNT, "rule_rhs_binding_hits");
    match value {
        Expr::Call { .. } => record_substitution_sharing_counter(
            &RULE_RHS_BINDING_CALL_HIT_COUNT,
            "rule_rhs_binding_call_hits",
        ),
        Expr::Atom(_) => record_substitution_sharing_counter(
            &RULE_RHS_BINDING_ATOM_HIT_COUNT,
            "rule_rhs_binding_atom_hits",
        ),
    }
}

fn record_rule_rhs_preserved_args(args: &[Expr]) {
    for arg in args {
        record_rule_rhs_preserved_arg(arg);
    }
}

fn record_rule_rhs_preserved_arg(arg: &Expr) {
    record_substitution_sharing_counter(
        &RULE_RHS_PRESERVED_EXISTING_ARG_COUNT,
        "rule_rhs_preserved_existing_args",
    );
    if matches!(arg, Expr::Call { .. }) {
        record_substitution_sharing_counter(
            &RULE_RHS_PRESERVED_EXISTING_CALL_ARG_COUNT,
            "rule_rhs_preserved_existing_call_args",
        );
    }
}

#[inline]
fn substitution_sharing_profile_enabled() -> bool {
    match SUBSTITUTION_SHARING_PROFILE_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_substitution_sharing_profile_enabled(),
    }
}

#[cold]
fn initialize_substitution_sharing_profile_enabled() -> bool {
    let enabled = std::env::var_os(SUBSTITUTION_SHARING_PROFILE_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    SUBSTITUTION_SHARING_PROFILE_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[inline]
fn record_substitution_sharing_counter(counter: &AtomicU64, name: &str) {
    if !substitution_sharing_profile_enabled() {
        return;
    }

    record_substitution_sharing_counter_slow(counter, name);
}

#[cold]
#[inline(never)]
fn record_substitution_sharing_counter_slow(counter: &AtomicU64, name: &str) {
    let value = counter.fetch_add(1, Ordering::Relaxed) + 1;
    if value.is_power_of_two() {
        eprintln!("rum-profile substitution-sharing {name}={value}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_rhs_substitution_reuses_bound_call_shared_nodes() {
        let value = Expr::call(
            system_expr("g"),
            vec![Expr::call(
                system_expr("h"),
                vec![Expr::integer(BigInt::from(1_u8))],
            )],
        );
        let mut bindings = SymbolExprMap::default();
        bindings.insert(Symbol::from_name("x"), value.clone());
        let rhs = Expr::call(
            system_expr("f"),
            vec![
                Expr::Atom(Atom::Symbol(Symbol::from_name("x"))),
                Expr::Atom(Atom::Symbol(Symbol::from_name("x"))),
            ],
        );

        let substituted = substitute_rule_bindings_for_rhs(&rhs, &bindings, false);
        let Expr::Call { args, .. } = substituted else {
            panic!("expected substituted call");
        };

        assert_eq!(args.len(), 2);
        assert_call_shared_nodes_eq(&value, &args[0]);
        assert_call_shared_nodes_eq(&value, &args[1]);
        assert_call_shared_nodes_eq(&args[0], &args[1]);
    }

    #[test]
    fn rule_rhs_substitution_preserves_unchanged_sibling_call_shared_nodes() {
        let bound = Expr::call(system_expr("g"), vec![Expr::integer(BigInt::from(1_u8))]);
        let sibling = Expr::call(system_expr("s"), vec![Expr::integer(BigInt::from(2_u8))]);
        let mut bindings = SymbolExprMap::default();
        bindings.insert(Symbol::from_name("x"), bound);
        let rhs = Expr::call(
            system_expr("f"),
            vec![
                Expr::Atom(Atom::Symbol(Symbol::from_name("x"))),
                sibling.clone(),
            ],
        );

        let substituted = substitute_rule_bindings_for_rhs(&rhs, &bindings, false);
        let Expr::Call { args, .. } = substituted else {
            panic!("expected substituted call");
        };

        assert_eq!(args.len(), 2);
        assert_call_shared_nodes_eq(&sibling, &args[1]);
    }

    #[test]
    fn rule_rhs_sequence_splice_preserves_spliced_item_shared_nodes() {
        let item = Expr::call(system_expr("s"), vec![Expr::integer(BigInt::from(1_u8))]);
        let mut bindings = SymbolExprMap::default();
        bindings.insert(Symbol::from_name("x"), sequence_expr(vec![item.clone()]));
        let rhs = Expr::call(
            system_expr("f"),
            vec![Expr::Atom(Atom::Symbol(Symbol::from_name("x")))],
        );

        let substituted = substitute_rule_bindings_for_rhs(&rhs, &bindings, false);
        let Expr::Call { args, .. } = substituted else {
            panic!("expected substituted call");
        };

        assert_eq!(args.len(), 1);
        assert_call_shared_nodes_eq(&item, &args[0]);
    }

    #[test]
    fn rule_rhs_alpha_rename_does_not_rewrite_inserted_bound_values() {
        let x = Symbol::from_name("x");
        let body = Symbol::from_name("body");
        let bound = Expr::call(system_expr("Hold"), vec![symbol_expr(&x)]);
        let mut bindings = SymbolExprMap::default();
        bindings.insert(body.clone(), bound.clone());
        let rhs = Expr::call(
            system_expr("Module"),
            vec![
                Expr::call(system_expr("List"), vec![symbol_expr(&x)]),
                Expr::call(
                    system_expr("Hold"),
                    vec![symbol_expr(&body), symbol_expr(&x)],
                ),
            ],
        );

        let substituted = substitute_rule_bindings_for_rhs(&rhs, &bindings, false);
        let module_locals = call_arg(&substituted, 0);
        let module_body = call_arg(&substituted, 1);
        let alpha_local = symbol_atom(call_arg(module_locals, 0)).clone();

        assert_eq!(alpha_local.context(), x.context());
        assert_eq!(alpha_local.name(), x.name());
        assert!(alpha_local.identity().is_some());
        assert_eq!(symbol_atom(call_arg(module_body, 1)), &alpha_local);

        let inserted_value = call_arg(module_body, 0);
        assert_call_shared_nodes_eq(&bound, inserted_value);
        let inserted_x = symbol_atom(call_arg(inserted_value, 0));
        assert_eq!(inserted_x, &x);
        assert_eq!(inserted_x.identity(), None);
        assert_ne!(inserted_x, &alpha_local);
    }

    fn symbol_expr(symbol: &Symbol) -> Expr {
        Expr::Atom(Atom::Symbol(symbol.clone()))
    }

    fn call_arg(expr: &Expr, index: usize) -> &Expr {
        let Expr::Call { args, .. } = expr else {
            panic!("expected call expression");
        };
        &args[index]
    }

    fn symbol_atom(expr: &Expr) -> &Symbol {
        let Expr::Atom(Atom::Symbol(symbol)) = expr else {
            panic!("expected symbol atom");
        };
        symbol
    }

    fn assert_call_shared_nodes_eq(expected: &Expr, actual: &Expr) {
        let (
            Expr::Call {
                head: expected_head,
                args: expected_args,
            },
            Expr::Call {
                head: actual_head,
                args: actual_args,
            },
        ) = (expected, actual)
        else {
            panic!("expected both expressions to be calls");
        };

        assert!(SharedExpr::ptr_eq(expected_head, actual_head));
        assert!(SharedArgs::ptr_eq(expected_args, actual_args));
    }
}
