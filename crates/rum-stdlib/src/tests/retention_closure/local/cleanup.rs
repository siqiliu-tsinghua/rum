//! Retention cleanup is not a stack-depth guarantee. Expr/EvalError destruction
//! releases ownership and runs no guest callback. Deep Expr destruction can
//! still overflow the native stack: TODO.md, Standing engineering debt.
use super::*;

pub(super) fn ownership(source: &Source, t: &ExactTy) -> Ty {
    if matches!(t,ExactTy::Nominal(id,_) if source.drop_types.contains(id)) {
        return Ty::Unchecked("unclassified custom Drop".into());
    }
    match t {
        _ if option_domain::copy_member(source, t) => Ty::Scalar(t.spelling()),
        ExactTy::Nominal(id, args)
            if [protocol::EXPR, protocol::ERROR, protocol::ARGS].contains(&id.as_str())
                && args.is_empty() =>
        {
            Ty::Fixed(id.clone())
        }
        ExactTy::Nominal(id, args)
            if [
                protocol::CONTEXT,
                protocol::VALUES,
                protocol::RESULT,
                protocol::CALLBACK,
                protocol::FAILURE,
                option_boundaries::OWNER,
            ]
            .contains(&id.as_str())
                && args.len() == 1
                && types::is_mode(&args[0]) =>
        {
            Ty::Retained(t.spelling())
        }
        ExactTy::Nominal(id, args)
            if id == "std::ops::Range" && args == &[protocol::scalar("usize")] =>
        {
            Ty::Fixed(t.spelling())
        }
        ExactTy::Nominal(id, args) if id == protocol::PROBE && args.is_empty() => {
            if source.drop_types.contains(id) {
                return Ty::Unchecked("continuation acquired custom Drop".into());
            }
            match types::fields(source, t, None) {
                Ok(fs) => Ty::Fields(
                    fs.into_iter()
                        .map(|(n, t)| (n, ownership(source, &t)))
                        .collect(),
                ),
                Err(e) => Ty::Unchecked(e),
            }
        }
        ExactTy::Nominal(id, _)
            if ["std::option::Option", "std::result::Result"].contains(&id.as_str()) =>
        {
            let Ok(vs) = types::variants(source, t) else {
                return Ty::Unchecked("unknown variant cleanup".into());
            };
            let mut states = BTreeMap::new();
            for v in vs {
                let Ok(fs) = types::fields(source, t, Some(&v)) else {
                    return Ty::Unchecked("unknown field cleanup".into());
                };
                states.insert(
                    v,
                    fs.into_iter()
                        .map(|(n, t)| (n, ownership(source, &t)))
                        .collect(),
                );
            }
            Ty::Variants(states)
        }
        ExactTy::Tuple(ts) => Ty::Tuple(ts.iter().map(|t| ownership(source, t)).collect()),
        _ => types::ownership(source, t),
    }
}

pub(super) fn unknown(source: &Source, t: &ExactTy) -> bool {
    if let ExactTy::Nominal(id, args) = t {
        if types::OWNED_CONTAINERS.contains(&id.as_str()) {
            return args.iter().any(|t| unknown(source, t));
        }
    }
    matches!(
        types::retention(&ownership(source, t)),
        Some(Kind::RET002 | Kind::RET004)
    )
}
