//! Closed concrete instances used by the four probe bodies. These are call
//! contracts, never a general trait-dispatch or generic-function admission.
use super::*;

pub(super) const EXPR: &str = "rum_core::expr::Expr";
pub(super) const ARGS: &str = "rum_core::expr::SharedArgs";
pub(super) const ERROR: &str = "rum_runtime::runtime::session::session_state::EvalError";
pub(super) const CONTEXT: &str =
    "rum_runtime::runtime::accounting_mode::retained::RetentionContext";
pub(super) const VALUES: &str = "rum_runtime::runtime::accounting_mode::retained::RetainedValues";
pub(super) const RESULT: &str = "rum_runtime::runtime::accounting_mode::retained::RetainedResult";
pub(super) const CALLBACK: &str = "rum_runtime::runtime::accounting_mode::bridge::CallbackValue";
pub(super) const FAILURE: &str = "rum_runtime::runtime::accounting_mode::bridge::CallbackFailure";
pub(super) const SCOPED: &str = "rum_runtime::runtime::accounting_mode::retained::scoped";
pub(super) const PROBE: &str = "rum_runtime::runtime::accounting_mode::tests::ProbeContinuation";

pub(super) fn nominal(id: &str) -> ExactTy {
    ExactTy::Nominal(id.into(), vec![])
}
pub(super) fn instance(id: &str, mode: &ExactTy) -> ExactTy {
    ExactTy::Nominal(id.into(), vec![mode.clone()])
}
pub(super) fn result(ok: ExactTy, error: ExactTy) -> ExactTy {
    ExactTy::Nominal("std::result::Result".into(), vec![ok, error])
}
pub(super) fn scalar(id: &str) -> ExactTy {
    ExactTy::Scalar(id.into())
}
pub(super) fn borrow(ty: ExactTy, mutable: bool) -> ExactTy {
    ExactTy::Reference(Box::new(ty), mutable)
}
pub(super) fn base(mut t: &ExactTy) -> &ExactTy {
    while let ExactTy::Reference(inner, _) = t {
        t = inner;
    }
    t
}

pub(super) fn key(target: &str, mode: &ExactTy) -> String {
    format!("{target}::<{}>", mode.spelling())
}

pub(super) fn signature<'a>(source: &'a Source, target: &str) -> Option<&'a Function> {
    let [f] = source.functions.get(target)?.as_slice() else {
        return None;
    };
    (!f.conditional).then_some(f)
}

/// Every addition has its own nineteen-field record. Effects are a closed
/// vocabulary; no arbitrary record text can turn an unknown call inert.
pub(super) fn extra_targets() -> BTreeMap<String, String> {
    let mut rows = BTreeMap::from([
        ("P01".into(), format!("<{EXPR} as std::cmp::PartialEq>::eq")),
        ("P02".into(), format!("{EXPR}::small_integer")),
        (
            "P03".into(),
            format!("{EXPR}::string::<std::string::String>"),
        ),
        ("P04".into(), "str::repeat".into()),
        (
            "P05".into(),
            format!("<{ARGS} as std::clone::Clone>::clone"),
        ),
        ("P06".into(), "std::option::Option<usize>::is_some".into()),
    ]);
    for (n, mode) in types::MODES.iter().enumerate() {
        let mode = nominal(mode);
        for (i, target) in [
            format!("{CONTEXT}::reserve"),
            format!("{CONTEXT}::push"),
            format!("{CONTEXT}::evaluate"),
            format!("{CONTEXT}::finish_list"),
            format!("{CONTEXT}::resume_values"),
            format!("{CALLBACK}::adopt"),
            format!("{FAILURE}::suspend::<{PROBE},literal>"),
            SCOPED.into(),
            "std::result::Result::and_then::<CallbackValue::adopt>".into(),
        ]
        .iter()
        .enumerate()
        {
            rows.insert(format!("P{:02}{n}", i + 7), key(target, &mode));
        }
    }
    rows
}

pub(super) fn effect(id: &str) -> &'static str {
    match id {
        "B04" | "P02" | "P03" | "P04" | "P05" => "produces-owned;inert",
        "B05" | "I01" | "I04" | "P01" | "P06" => "borrows;inert",
        _ => "may-run-guest-or-grow",
    }
}

pub(super) struct Call {
    pub target: String,
    pub inputs: Vec<ExactTy>,
    pub output: ExactTy,
    pub borrow_receiver: bool,
}

pub(super) fn method(source: &Source, ty: &ExactTy, name: &str) -> Option<Call> {
    if matches!(ty, ExactTy::Reference(t, _) if matches!(&**t, ExactTy::Reference(..))) {
        // Additional reference slots may select Clone for a reference itself.
        // The probe needs at most one built-in reference dereference.
        return None;
    }
    let owner = base(ty);
    if let ExactTy::Nominal(id, _) = owner {
        if source.opaque_inherent_owners.contains(id) {
            return None;
        }
        if [EXPR, ARGS].contains(&id.as_str())
            && source.functions.contains_key(&format!("{id}::{name}"))
        {
            return None;
        }
    }
    let (target, inputs, output, borrow_receiver) = match (owner, name) {
        (ExactTy::Nominal(id, args), "is_some")
            if id == "std::option::Option" && args == &[scalar("usize")] =>
        {
            (
                "std::option::Option<usize>::is_some".into(),
                vec![],
                scalar("bool"),
                true,
            )
        }
        (ExactTy::Scalar(id), "repeat") if id == "str" => (
            "str::repeat".into(),
            vec![scalar("usize")],
            nominal("std::string::String"),
            true,
        ),
        (ExactTy::Nominal(id, args), "len") if id == ARGS && args.is_empty() => {
            (contracts::target("B05"), vec![], scalar("usize"), true)
        }
        (ExactTy::Nominal(id, args), "clone")
            if [EXPR, ARGS].contains(&id.as_str()) && args.is_empty() =>
        {
            (
                format!("<{id} as std::clone::Clone>::clone"),
                vec![],
                owner.clone(),
                true,
            )
        }
        (ExactTy::Nominal(id, args), name)
            if id == CONTEXT
                && args.len() == 1
                && types::is_mode(&args[0])
                && [
                    "reserve",
                    "push",
                    "evaluate",
                    "finish_list",
                    "resume_values",
                ]
                .contains(&name) =>
        {
            let target = format!("{id}::{name}");
            let f = signature(source, &target)?;
            types::attributes(&f.attrs).ok()?;
            if f.sig.unsafety.is_some()
                || f.sig.asyncness.is_some()
                || f.sig.abi.is_some()
                || f.sig.variadic.is_some()
            {
                return None;
            }
            let f = types::with_mode(source, f, Some(&args[0])).ok()?;
            let parameters = types::exact_parameters(source, &f).ok()?;
            (
                key(&target, &args[0]),
                parameters.iter().skip(1).map(|(_, t)| t.clone()).collect(),
                types::exact_output(source, &f).ok()?,
                matches!(parameters[0].1, ExactTy::Reference(..)),
            )
        }
        _ => return None,
    };
    Some(Call {
        target,
        inputs,
        output,
        borrow_receiver,
    })
}

/// Imported local traits may introduce earlier receiver slots. Refuse them
/// conservatively, even without a matching impl. The finite protocol scope is
/// also fingerprinted and every selected probe call is checked against rustc.
pub(super) fn competing_trait(source: &Source, f: &Function, name: &str) -> bool {
    let mut module = Some(f.module.as_str());
    while let Some(id) = module {
        let scope = &source.modules[id];
        for binding in scope.bindings.values().flatten() {
            if let Some(definitions) = source.traits.get(&binding.target) {
                if definitions.iter().any(|d| {
                    d.item
                        .items
                        .iter()
                        .any(|item| matches!(item, syn::TraitItem::Fn(m) if m.sig.ident == name))
                }) {
                    return true;
                }
            }
        }
        module = id.rsplit_once("::").map(|(parent, _)| parent);
    }
    false
}
