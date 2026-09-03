use super::super::*;
use std::{cell::RefCell, hash::BuildHasherDefault};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceCacheKey {
    head: usize,
    args: usize,
}

type PresenceCache = HashMap<PresenceCacheKey, bool, BuildHasherDefault<RuntimeCacheHasher>>;

pub(in crate::runtime) struct LexicalPresenceCache {
    cache: RefCell<PresenceCache>,
}

impl LexicalPresenceCache {
    pub(in crate::runtime) fn new() -> Self {
        Self {
            cache: RefCell::new(PresenceCache::default()),
        }
    }

    pub(in crate::runtime) fn may_contain_symbol(&self, expr: &Expr, target: &Symbol) -> bool {
        if lexical_profile_enabled() {
            let profile_start = if lexical_profile_detail_enabled() {
                lexical_profile_start()
            } else {
                None
            };
            let (result, nodes) = expr_may_contain_symbol_counted(expr, target, &self.cache);
            record_profile_single_presence_scan(profile_start, expr, target, result, nodes);
            return result;
        }

        expr_may_contain_symbol_fast(expr, target, &self.cache)
    }

    pub(in crate::runtime) fn may_contain_any_binding(
        &self,
        expr: &Expr,
        bindings: &SymbolExprMap,
    ) -> bool {
        if lexical_profile_enabled() {
            let profile_start = if lexical_profile_detail_enabled() {
                lexical_profile_start()
            } else {
                None
            };
            let (result, nodes) = expr_may_contain_any_binding_counted(expr, bindings, &self.cache);
            record_profile_multi_presence_scan(profile_start, expr, result, nodes);
            return result;
        }

        expr_may_contain_any_binding_fast(expr, bindings, &self.cache)
    }
}

fn call_presence_key(head: &SharedExpr, args: &SharedArgs) -> PresenceCacheKey {
    PresenceCacheKey {
        head: head.ptr_key(),
        args: args.ptr_key(),
    }
}

fn expr_may_contain_symbol_fast(
    expr: &Expr,
    target: &Symbol,
    cache: &RefCell<PresenceCache>,
) -> bool {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => symbol == target,
        Expr::Atom(_) => false,
        Expr::Call { head, args } => {
            let key = call_presence_key(head, args);
            if let Some(result) = cache.borrow().get(&key).copied() {
                return result;
            }

            let result = expr_may_contain_symbol_fast(head, target, cache)
                || args
                    .iter()
                    .any(|arg| expr_may_contain_symbol_fast(arg, target, cache));
            cache.borrow_mut().insert(key, result);
            result
        }
    }
}

fn expr_may_contain_symbol_counted(
    expr: &Expr,
    target: &Symbol,
    cache: &RefCell<PresenceCache>,
) -> (bool, u64) {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => (symbol == target, 1),
        Expr::Atom(_) => (false, 1),
        Expr::Call { head, args } => {
            let key = call_presence_key(head, args);
            if let Some(result) = cache.borrow().get(&key).copied() {
                return (result, 0);
            }

            let (head_result, mut nodes) = expr_may_contain_symbol_counted(head, target, cache);
            if head_result {
                cache.borrow_mut().insert(key, true);
                return (true, nodes);
            }

            for arg in args.iter() {
                let (arg_result, arg_nodes) = expr_may_contain_symbol_counted(arg, target, cache);
                nodes = nodes.saturating_add(arg_nodes);
                if arg_result {
                    cache.borrow_mut().insert(key, true);
                    return (true, nodes);
                }
            }

            cache.borrow_mut().insert(key, false);
            (false, nodes)
        }
    }
}

fn expr_may_contain_any_binding_fast(
    expr: &Expr,
    bindings: &SymbolExprMap,
    cache: &RefCell<PresenceCache>,
) -> bool {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => bindings.contains_key(symbol),
        Expr::Atom(_) => false,
        Expr::Call { head, args } => {
            let key = call_presence_key(head, args);
            if let Some(result) = cache.borrow().get(&key).copied() {
                return result;
            }

            let result = expr_may_contain_any_binding_fast(head, bindings, cache)
                || args
                    .iter()
                    .any(|arg| expr_may_contain_any_binding_fast(arg, bindings, cache));
            cache.borrow_mut().insert(key, result);
            result
        }
    }
}

fn expr_may_contain_any_binding_counted(
    expr: &Expr,
    bindings: &SymbolExprMap,
    cache: &RefCell<PresenceCache>,
) -> (bool, u64) {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => (bindings.contains_key(symbol), 1),
        Expr::Atom(_) => (false, 1),
        Expr::Call { head, args } => {
            let key = call_presence_key(head, args);
            if let Some(result) = cache.borrow().get(&key).copied() {
                return (result, 0);
            }

            let (head_result, mut nodes) =
                expr_may_contain_any_binding_counted(head, bindings, cache);
            if head_result {
                cache.borrow_mut().insert(key, true);
                return (true, nodes);
            }

            for arg in args.iter() {
                let (arg_result, arg_nodes) =
                    expr_may_contain_any_binding_counted(arg, bindings, cache);
                nodes = nodes.saturating_add(arg_nodes);
                if arg_result {
                    cache.borrow_mut().insert(key, true);
                    return (true, nodes);
                }
            }

            cache.borrow_mut().insert(key, false);
            (false, nodes)
        }
    }
}
