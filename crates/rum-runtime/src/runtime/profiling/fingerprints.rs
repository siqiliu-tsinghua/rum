use super::*;

pub(super) fn presence_target_origin(symbol: Option<&Symbol>) -> PresenceTargetOrigin {
    let Some(symbol) = symbol else {
        return PresenceTargetOrigin::None;
    };

    if let Some(identity) = symbol.identity() {
        if identity & RULE_RHS_ALPHA_ID_MARKER != 0 {
            PresenceTargetOrigin::RuleRhsAlphaIdentity
        } else {
            PresenceTargetOrigin::OtherIdentity
        }
    } else if is_generated_dollar_name(symbol.name()) {
        PresenceTargetOrigin::GeneratedDollarName
    } else {
        PresenceTargetOrigin::PlainSymbol
    }
}

fn is_generated_dollar_name(name: &str) -> bool {
    let Some((_, suffix)) = name.rsplit_once('$') else {
        return false;
    };
    !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
}

fn rule_rhs_alpha_id(symbol: &Symbol) -> Option<u64> {
    symbol
        .identity()
        .filter(|identity| identity & RULE_RHS_ALPHA_ID_MARKER != 0)
        .map(|identity| identity & !RULE_RHS_ALPHA_ID_MARKER)
}

pub(super) fn max_rule_rhs_alpha_id(expr: &Expr) -> Option<u64> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => rule_rhs_alpha_id(symbol),
        Expr::Atom(_) => None,
        Expr::Call { head, args, .. } => {
            let mut max_id = max_rule_rhs_alpha_id(head);
            for arg in args.iter() {
                if let Some(arg_id) = max_rule_rhs_alpha_id(arg) {
                    max_id = Some(max_id.map_or(arg_id, |current| current.max(arg_id)));
                }
            }
            max_id
        }
    }
}

pub(super) fn presence_alpha_relation(
    target: Option<&Symbol>,
    provenance: ExpressionProvenanceValue,
) -> PresenceAlphaRelation {
    let Some(target) = target else {
        return PresenceAlphaRelation::NoTarget;
    };
    let Some(target_id) = rule_rhs_alpha_id(target) else {
        return PresenceAlphaRelation::TargetNotRuleRhsAlpha;
    };

    match provenance.max_rule_rhs_alpha_id {
        Some(max_id) if max_id < target_id => PresenceAlphaRelation::ExprOlderThanTarget,
        Some(_) => PresenceAlphaRelation::ExprSameOrNewerThanTarget,
        None if provenance.kind == ExpressionProvenanceKind::Unknown => {
            PresenceAlphaRelation::Unknown
        }
        None => PresenceAlphaRelation::ExprHasNoRuleRhsAlpha,
    }
}

pub(super) fn structural_text_and_exact_fingerprints(expr: &Expr) -> (u64, u64) {
    let mut text_hasher = DefaultHasher::new();
    let mut exact_hasher = DefaultHasher::new();
    let _nodes = hash_expr_text_and_exact(expr, &mut text_hasher, &mut exact_hasher);
    (text_hasher.finish(), exact_hasher.finish())
}

pub(super) fn structural_text_exact_fingerprints_and_nodes(expr: &Expr) -> (u64, u64, u64) {
    let mut text_hasher = DefaultHasher::new();
    let mut exact_hasher = DefaultHasher::new();
    let nodes = hash_expr_text_and_exact(expr, &mut text_hasher, &mut exact_hasher);
    (text_hasher.finish(), exact_hasher.finish(), nodes)
}

fn hash_expr_text_and_exact(
    expr: &Expr,
    text_state: &mut impl Hasher,
    exact_state: &mut impl Hasher,
) -> u64 {
    match expr {
        Expr::Atom(atom) => {
            hash_atom_text_and_exact(atom, text_state, exact_state);
            1
        }
        Expr::Call { head, args, .. } => {
            4_u8.hash(text_state);
            4_u8.hash(exact_state);
            let mut nodes =
                1_u64.saturating_add(hash_expr_text_and_exact(head, text_state, exact_state));
            args.len().hash(text_state);
            args.len().hash(exact_state);
            for arg in args.iter() {
                nodes =
                    nodes.saturating_add(hash_expr_text_and_exact(arg, text_state, exact_state));
            }
            nodes
        }
    }
}

fn hash_atom_text_and_exact(
    atom: &Atom,
    text_state: &mut impl Hasher,
    exact_state: &mut impl Hasher,
) {
    match atom {
        Atom::Integer(value) => {
            0_u8.hash(text_state);
            value.hash(text_state);
            0_u8.hash(exact_state);
            value.hash(exact_state);
        }
        Atom::Rational(value) => {
            1_u8.hash(text_state);
            value.hash(text_state);
            1_u8.hash(exact_state);
            value.hash(exact_state);
        }
        Atom::String(value) => {
            2_u8.hash(text_state);
            value.hash(text_state);
            2_u8.hash(exact_state);
            value.hash(exact_state);
        }
        Atom::Symbol(symbol) => {
            3_u8.hash(text_state);
            symbol.context().hash(text_state);
            symbol.name().hash(text_state);
            3_u8.hash(exact_state);
            symbol.hash(exact_state);
        }
    }
}
