use super::*;

pub(super) fn record_presence_shape(
    kind: PresenceScanKind,
    expr: &Expr,
    result: bool,
    nodes: u64,
    elapsed_nanos: Option<u64>,
) {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    let key = PresenceShapeKey {
        kind,
        shape: presence_root_shape(expr),
    };
    let context = current_lexical_profile_context();
    let context_key = PresenceContextKey { kind, context };
    let context_shape_key = PresenceContextShapeKey {
        kind,
        context,
        shape: key.shape,
    };
    PRESENCE_SHAPE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.scans = entry.scans.saturating_add(1);
        entry.nodes = entry.nodes.saturating_add(nodes);
        if let Some(elapsed_nanos) = elapsed_nanos {
            entry.timed = entry.timed.saturating_add(1);
            entry.nanos = entry.nanos.saturating_add(elapsed_nanos);
        }
        if result {
            entry.true_count = entry.true_count.saturating_add(1);
        } else {
            entry.false_count = entry.false_count.saturating_add(1);
        }
    });
    PRESENCE_CONTEXT_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(context_key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
    PRESENCE_CONTEXT_SHAPE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(context_shape_key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
    if let Some(site) = current_arg_prefilter_site() {
        let site_key = PresenceArgSiteKey {
            kind,
            site,
            shape: key.shape,
        };
        PRESENCE_ARG_SITE_STATS.with(|stats| {
            let mut stats = stats.borrow_mut();
            let entry = stats.entry(site_key).or_default();
            record_presence_stats(entry, result, nodes, elapsed_nanos);
        });
    }
}

pub(super) fn record_presence_source(
    kind: PresenceScanKind,
    expr: &Expr,
    target_text: Option<Symbol>,
    result: bool,
    nodes: u64,
    elapsed_nanos: Option<u64>,
) {
    if !lexical_profile_sources_enabled() {
        return;
    }

    let Some(source) = evaluation_profile_current_source_symbol() else {
        return;
    };

    let context = current_lexical_profile_context();
    let shape = presence_root_shape(expr);
    let source_key = PresenceSourceKey {
        kind,
        source,
        context,
    };
    let source_shape_key = PresenceSourceShapeKey {
        kind,
        source,
        context,
        shape,
    };
    let source_target_shape_key = PresenceSourceTargetShapeKey {
        kind,
        source,
        context,
        shape,
        target_text,
    };
    PRESENCE_SOURCE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(source_key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
    PRESENCE_SOURCE_SHAPE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(source_shape_key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
    PRESENCE_SOURCE_TARGET_SHAPE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(source_target_shape_key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
    record_presence_source_fingerprint(
        kind,
        source,
        context,
        shape,
        target_text,
        expr,
        result,
        nodes,
        elapsed_nanos,
    );
}

fn record_presence_source_fingerprint(
    kind: PresenceScanKind,
    source: Symbol,
    context: &'static str,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
    expr: &Expr,
    result: bool,
    nodes: u64,
    elapsed_nanos: Option<u64>,
) {
    if !lexical_profile_source_fingerprints_enabled() || nodes < PRESENCE_REUSE_MIN_NODES {
        return;
    }

    let (text_fingerprint, exact_fingerprint) = structural_text_and_exact_fingerprints(expr);
    for (fingerprint_kind, fingerprint) in [
        (PresenceSourceFingerprintKind::Text, text_fingerprint),
        (PresenceSourceFingerprintKind::Exact, exact_fingerprint),
    ] {
        let key = PresenceSourceFingerprintKey {
            fingerprint_kind,
            kind,
            source,
            context,
            shape,
            target_text,
            fingerprint,
        };
        PRESENCE_SOURCE_FINGERPRINT_STATS.with(|stats| {
            let mut stats = stats.borrow_mut();
            let entry = stats.entry(key).or_default();
            record_presence_stats(entry, result, nodes, elapsed_nanos);
        });
    }
}

pub(super) fn record_expression_provenance_from_fingerprint(
    exact_fingerprint: u64,
    kind: ExpressionProvenanceKind,
    source: &'static str,
    source_symbol: Option<Symbol>,
    max_rule_rhs_alpha_id: Option<u64>,
    nodes: u64,
) {
    if !lexical_profile_provenance_enabled() {
        return;
    }

    EXPRESSION_PROVENANCE_BY_EXACT.with(|map| {
        let mut map = map.borrow_mut();
        let entry = map
            .entry(exact_fingerprint)
            .or_insert(ExpressionProvenanceValue {
                kind,
                source,
                source_symbol,
                max_rule_rhs_alpha_id,
                records: 0,
                nodes: 0,
            });
        entry.records = entry.records.saturating_add(1);
        entry.nodes = entry.nodes.saturating_add(nodes);
        entry.max_rule_rhs_alpha_id = match (entry.max_rule_rhs_alpha_id, max_rule_rhs_alpha_id) {
            (Some(lhs), Some(rhs)) => Some(lhs.max(rhs)),
            (Some(lhs), None) => Some(lhs),
            (None, Some(rhs)) => Some(rhs),
            (None, None) => None,
        };
        if kind.priority() > entry.kind.priority() {
            entry.kind = kind;
            entry.source = source;
            entry.source_symbol = source_symbol;
        }
    });
}

pub(super) fn record_presence_provenance(
    kind: PresenceScanKind,
    expr: &Expr,
    target: Option<&Symbol>,
    target_text: Option<Symbol>,
    result: bool,
    nodes: u64,
    elapsed_nanos: Option<u64>,
) {
    if !lexical_profile_provenance_enabled() || nodes < PRESENCE_REUSE_MIN_NODES {
        return;
    }

    let (_, exact_fingerprint) = structural_text_and_exact_fingerprints(expr);
    let provenance = EXPRESSION_PROVENANCE_BY_EXACT.with(|map| {
        map.borrow()
            .get(&exact_fingerprint)
            .copied()
            .unwrap_or(ExpressionProvenanceValue {
                kind: ExpressionProvenanceKind::Unknown,
                source: "-",
                source_symbol: None,
                max_rule_rhs_alpha_id: None,
                records: 0,
                nodes: 0,
            })
    });
    let key = PresenceProvenanceKey {
        kind,
        source: evaluation_profile_current_source_symbol(),
        context: current_lexical_profile_context(),
        target_text,
        target_origin: presence_target_origin(target),
        expr_origin: provenance.kind,
        expr_origin_source: provenance.source,
        expr_origin_symbol: provenance.source_symbol,
        alpha_relation: presence_alpha_relation(target, provenance),
        shape: presence_root_shape(expr),
    };
    PRESENCE_PROVENANCE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
}

pub(super) fn record_presence_binding(
    expr: &Expr,
    target: &Symbol,
    result: bool,
    nodes: u64,
    elapsed_nanos: Option<u64>,
) {
    if !lexical_profile_bindings_enabled() {
        return;
    }

    let key = PresenceBindingKey {
        shape: presence_root_shape(expr),
        target: *target,
    };
    let name_key = PresenceBindingNameKey {
        shape: key.shape,
        target_text: Symbol::new(target.context(), target.name()),
    };
    PRESENCE_BINDING_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.scans = entry.scans.saturating_add(1);
        entry.nodes = entry.nodes.saturating_add(nodes);
        if let Some(elapsed_nanos) = elapsed_nanos {
            entry.timed = entry.timed.saturating_add(1);
            entry.nanos = entry.nanos.saturating_add(elapsed_nanos);
        }
        if result {
            entry.true_count = entry.true_count.saturating_add(1);
        } else {
            entry.false_count = entry.false_count.saturating_add(1);
        }
    });
    PRESENCE_BINDING_NAME_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(name_key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
}

pub(super) fn record_presence_reuse(
    kind: PresenceScanKind,
    expr: &Expr,
    target_text: Option<Symbol>,
    result: bool,
    nodes: u64,
    elapsed_nanos: Option<u64>,
) {
    if !lexical_profile_shapes_enabled() || nodes < PRESENCE_REUSE_MIN_NODES {
        return;
    }

    let Expr::Call { head, args, .. } = expr else {
        return;
    };

    let key = PresenceReuseKey {
        kind,
        shape: presence_root_shape(expr),
        target_text,
        head_ptr: head.ptr_key(),
        args_ptr: args.ptr_key(),
    };
    PRESENCE_REUSE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
    record_presence_fingerprint(kind, expr, target_text, result, nodes, elapsed_nanos);
}

fn record_presence_fingerprint(
    kind: PresenceScanKind,
    expr: &Expr,
    target_text: Option<Symbol>,
    result: bool,
    nodes: u64,
    elapsed_nanos: Option<u64>,
) {
    if !lexical_profile_shapes_enabled() || nodes < PRESENCE_REUSE_MIN_NODES {
        return;
    }

    let (text_fingerprint, exact_fingerprint) = structural_text_and_exact_fingerprints(expr);
    let key = PresenceFingerprintKey {
        kind,
        shape: presence_root_shape(expr),
        target_text,
        fingerprint: text_fingerprint,
    };
    let exact_key = PresenceExactFingerprintKey {
        kind,
        shape: key.shape,
        target_text,
        fingerprint: exact_fingerprint,
    };
    PRESENCE_FINGERPRINT_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
    PRESENCE_EXACT_FINGERPRINT_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(exact_key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
}

pub(super) fn presence_root_shape(expr: &Expr) -> PresenceRootShape {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => PresenceRootShape::AtomSymbol(*symbol),
        Expr::Atom(_) => PresenceRootShape::AtomOther,
        Expr::Call { head, .. } => match head.as_ref() {
            Expr::Atom(Atom::Symbol(symbol)) => PresenceRootShape::CallHeadSymbol(*symbol),
            Expr::Atom(_) => PresenceRootShape::CallHeadAtomOther,
            Expr::Call {
                head: nested_head, ..
            } => match nested_head.as_ref() {
                Expr::Atom(Atom::Symbol(symbol)) => PresenceRootShape::CallHeadCallSymbol(*symbol),
                _ => PresenceRootShape::CallHeadCallOther,
            },
        },
    }
}

pub(super) fn current_lexical_profile_context() -> &'static str {
    if LEXICAL_CONTEXT_DEPTH.with(Cell::get) == 0 {
        return "root";
    }

    LEXICAL_CONTEXT_STACK.with(|stack| stack.borrow().last().copied().unwrap_or("root"))
}

pub(super) fn current_arg_prefilter_site() -> Option<PresenceArgSite> {
    if ARG_PREFILTER_SITE_DEPTH.with(Cell::get) == 0 {
        return None;
    }

    ARG_PREFILTER_SITE_STACK.with(|stack| stack.borrow().last().copied())
}

pub(super) fn current_expression_rebuild_source_symbol() -> Option<Symbol> {
    if EXPRESSION_REBUILD_SOURCE_DEPTH.with(Cell::get) == 0 {
        return None;
    }

    EXPRESSION_REBUILD_SOURCE_STACK.with(|stack| stack.borrow().last().copied())
}

fn record_presence_stats(
    stats: &mut PresenceShapeStats,
    result: bool,
    nodes: u64,
    elapsed_nanos: Option<u64>,
) {
    stats.scans = stats.scans.saturating_add(1);
    stats.nodes = stats.nodes.saturating_add(nodes);
    if let Some(elapsed_nanos) = elapsed_nanos {
        stats.timed = stats.timed.saturating_add(1);
        stats.nanos = stats.nanos.saturating_add(elapsed_nanos);
    }
    if result {
        stats.true_count = stats.true_count.saturating_add(1);
    } else {
        stats.false_count = stats.false_count.saturating_add(1);
    }
}
