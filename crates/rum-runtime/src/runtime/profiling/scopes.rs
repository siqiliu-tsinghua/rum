use super::*;

#[cold]
#[inline(never)]
fn lexical_profile_context_drop_enabled() {
    LEXICAL_CONTEXT_STACK.with(|stack| {
        stack.borrow_mut().pop();
    });
    LEXICAL_CONTEXT_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
}

impl Drop for LexicalProfileContextGuard {
    #[inline(always)]
    fn drop(&mut self) {
        if self.active {
            lexical_profile_context_drop_enabled();
        }
    }
}

#[cold]
#[inline(never)]
fn lexical_profile_arg_site_drop_enabled() {
    ARG_PREFILTER_SITE_STACK.with(|stack| {
        stack.borrow_mut().pop();
    });
    ARG_PREFILTER_SITE_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
}

impl Drop for LexicalProfileArgSiteGuard {
    #[inline(always)]
    fn drop(&mut self) {
        if self.active {
            lexical_profile_arg_site_drop_enabled();
        }
    }
}

#[cold]
#[inline(never)]
fn expression_rebuild_source_drop_enabled() {
    EXPRESSION_REBUILD_SOURCE_STACK.with(|stack| {
        stack.borrow_mut().pop();
    });
    EXPRESSION_REBUILD_SOURCE_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
}

impl Drop for ExpressionRebuildSourceGuard {
    #[inline(always)]
    fn drop(&mut self) {
        if self.active {
            expression_rebuild_source_drop_enabled();
        }
    }
}

#[cold]
#[inline(never)]
fn lexical_profile_context_enabled(context: &'static str) -> LexicalProfileContextGuard {
    LEXICAL_CONTEXT_STACK.with(|stack| {
        stack.borrow_mut().push(context);
    });
    LEXICAL_CONTEXT_DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
    LexicalProfileContextGuard { active: true }
}

#[inline]
pub(in crate::runtime) fn lexical_profile_context(
    context: &'static str,
) -> LexicalProfileContextGuard {
    if !lexical_profile_shapes_enabled() && !lexical_profile_sources_enabled() {
        LexicalProfileContextGuard { active: false }
    } else {
        lexical_profile_context_enabled(context)
    }
}

#[cold]
#[inline(never)]
fn lexical_profile_arg_prefilter_site_enabled(
    parent_head: &Expr,
    index: usize,
) -> LexicalProfileArgSiteGuard {
    let site = PresenceArgSite {
        parent: presence_root_shape(parent_head),
        index: index.min(usize::from(u16::MAX)) as u16,
    };
    ARG_PREFILTER_SITE_STACK.with(|stack| {
        stack.borrow_mut().push(site);
    });
    ARG_PREFILTER_SITE_DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
    LexicalProfileArgSiteGuard { active: true }
}

#[inline]
pub(in crate::runtime) fn lexical_profile_arg_prefilter_site(
    parent_head: &Expr,
    index: usize,
) -> LexicalProfileArgSiteGuard {
    if !lexical_profile_shapes_enabled() {
        LexicalProfileArgSiteGuard { active: false }
    } else {
        lexical_profile_arg_prefilter_site_enabled(parent_head, index)
    }
}

#[cold]
#[inline(never)]
fn expression_rebuild_source_symbol_enabled(symbol: Symbol) -> ExpressionRebuildSourceGuard {
    EXPRESSION_REBUILD_SOURCE_STACK.with(|stack| {
        stack.borrow_mut().push(symbol);
    });
    EXPRESSION_REBUILD_SOURCE_DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
    ExpressionRebuildSourceGuard { active: true }
}

#[inline]
pub(in crate::runtime) fn expression_rebuild_source_symbol(
    symbol: Symbol,
) -> ExpressionRebuildSourceGuard {
    if !lexical_profile_shapes_enabled()
        && !lexical_profile_provenance_enabled()
        && !rule_rhs_timing_enabled()
    {
        ExpressionRebuildSourceGuard { active: false }
    } else {
        expression_rebuild_source_symbol_enabled(symbol)
    }
}
