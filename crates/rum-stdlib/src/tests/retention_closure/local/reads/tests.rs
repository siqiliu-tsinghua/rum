use super::super::super::{regions, source};
use super::*;

pub(super) fn records() -> Vec<super::super::contracts::Contract> {
    super::super::contracts::read(include_str!("../../inventory/local-contracts.tsv")).unwrap()
}

pub(super) fn function(source: &Source, body: &str) -> Function {
    let mut f = resolve::function(
        source,
        &format!("{}::evaluate_runtime_option_expr", regions::SESSION),
    )
    .unwrap()
    .clone();
    f.sig = signature("fn evaluate_runtime_option_expr(expr: &Expr) -> bool");
    f.body = body.into();
    f
}

pub(super) fn evidence(name: &str, red: &Report, green: &Report, reason: &str) {
    assert!(red.text().contains(reason), "{name}: {}", red.text());
    assert!(green.rejected.is_empty(), "{name}: {}", green.text());
    let directory = source::workspace().join(".temp/brief-542/falsifiers");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join(format!("{name}.txt")),
        format!("RED\n{}GREEN 0 rejections\n", red.text()),
    )
    .unwrap();
    println!("READ542_CONTROL {name} red=reject green=clean");
}

pub(super) const READS: &str = "{
    let Expr::Call { head, args } = expr else { return false; };
    let name = args[0].clone();
    is_system_symbol_name(head, \"RuleDelayed\") && args.len() == 2
}";

#[test]
fn local_option_borrowed_pattern_controls() {
    let source = source::production();
    let index = Interfaces::shared_production();
    let marks = Marks::contracts(records()).unwrap();
    let f = function(&source, READS);
    let green = check(&index, &f, &marks);
    for (name, body, reason) in [
        (
            "wrong-field",
            READS.replace("head, args", "wrong: head, args"),
            "wrong borrowed field",
        ),
        (
            "explicit-ref",
            READS.replace("head, args", "ref head, args"),
            "reference or ownership mode",
        ),
        (
            "partial-move",
            READS.replace("head, args", "head, mut args"),
            "reference or ownership mode",
        ),
        (
            "nondiverging-else",
            READS.replace("return false;", "false;"),
            "must diverge",
        ),
        (
            "one-else-path-returns",
            READS.replace("return false;", "if true { return false; }"),
            "must diverge",
        ),
        (
            "conditional-rhs-return",
            READS.replace("return false;", "true && { return false; };"),
            "must diverge",
        ),
        (
            "field-type-mismatch",
            READS.replace("head, args", "head: args, args: head"),
            "call argument differs",
        ),
    ] {
        let red = check(&index, &function(&source, &body), &marks);
        evidence(name, &red, &green, reason);
    }
    let mut extra_ref = f.clone();
    extra_ref.sig = signature("fn evaluate_runtime_option_expr(expr: &&Expr) -> bool");
    evidence(
        "extra-reference",
        &check(&index, &extra_ref, &marks),
        &green,
        "exactly &Expr",
    );
    let mut owned = function(&source, READS);
    owned.sig = signature("fn evaluate_runtime_option_expr(expr: Expr) -> bool");
    evidence(
        "owned-pattern",
        &check(&index, &owned, &marks),
        &green,
        "exactly &Expr",
    );
    let expected = [
        protocol::borrow(protocol::nominal(SHARED), false),
        protocol::borrow(protocol::nominal(protocol::ARGS), false),
    ];
    for ty in expected {
        assert!(green.bindings.values().any(|t| t == &ty));
    }
    for id in ["I01", "I02", "I04", "B04", "B05", "B13"] {
        assert!(
            green
                .marks
                .contains_key(&super::super::contracts::target(id)),
            "{id}: {}",
            green.text()
        );
    }
    let mut changed = source.clone();
    changed.types.get_mut(protocol::EXPR).unwrap()[0]
        .variants
        .get_mut("Call")
        .unwrap()[0]
        .ty = type_text("SharedArgs");
    let altered = Interfaces::new(&changed);
    evidence(
        "changed-field-schema",
        &check(&altered, &f, &marks),
        &green,
        "field schema changed",
    );
    let mut imported = source.clone();
    let mut noise = source::fixture(&[(
        "noise",
        "noise.rs",
        "pub trait Noise { fn len(&self) -> usize; }",
    )]);
    imported.traits.append(&mut noise.traits);
    // Use the source index's own import/binding representation.
    let binding = source.modules[&f.module]
        .bindings
        .values()
        .flatten()
        .next()
        .unwrap()
        .clone();
    let mut binding = binding;
    binding.target = "fixture::noise::Noise".into();
    imported
        .modules
        .get_mut(&f.module)
        .unwrap()
        .bindings
        .insert("Noise".into(), BTreeSet::from([binding]));
    evidence(
        "imported-trait",
        &check(&Interfaces::new(&imported), &f, &marks),
        &green,
        "imported trait candidate",
    );

    let cleanup_body = "{ let Expr::Call { head, args } = expr else { { let retired = raw; } return false; }; { let retired = raw; } is_system_symbol_name(head, \"RuleDelayed\") }";
    let mut cleanup = function(&source, cleanup_body);
    cleanup.sig = signature("fn evaluate_runtime_option_expr(expr: &Expr, raw: Vec<u8>) -> bool");
    let restored = check(&index, &cleanup, &marks);
    // Failure's move/drop must not retire the owner on the continuing path.
    cleanup.body = cleanup_body.replace(
        "; { let retired = raw; } is_system",
        "; unknown(); is_system",
    );
    evidence(
        "else-cleanup-keeps-success-owner",
        &check(&index, &cleanup, &marks),
        &restored,
        "live across call",
    );
}

#[test]
fn local_option_short_circuit_ownership_controls() {
    fn run(body: &str) -> Report {
        let code = format!(
            "fn barrier() -> bool {{ true }} fn root(raw: Vec<u8>, flag: bool) -> bool {body}"
        );
        let source = source::fixture(&[("reads", "reads.rs", &code)]);
        let index = Interfaces::new(&source);
        let mut marks = Marks::default();
        marks
            .certify(
                &index,
                resolve::function(&source, "fixture::reads::barrier").unwrap(),
            )
            .unwrap();
        check(
            &index,
            resolve::function(&source, "fixture::reads::root").unwrap(),
            &marks,
        )
    }
    let green = run("{ { let retired = raw; } flag && barrier() }");
    for (name, body, reason) in [
        (
            "rhs-moves-owner",
            "{ flag && { let retired = raw; true } }",
            "branch ownership states differ",
        ),
        ("rhs-hazard", "{ flag && barrier() }", "live across call"),
        (
            "rhs-return-keeps-skipped-owner",
            "{ flag && { let retired = raw; return true; }; barrier() }",
            "live across call",
        ),
        (
            "rhs-new-owner",
            "{ flag && { raw = Vec::new(); true } }",
            "callee has no covered mark",
        ),
    ] {
        evidence(name, &run(body), &green, reason);
    }
    let terminating = run("{ flag && { let retired = raw; return true; } }");
    assert!(terminating.rejected.is_empty(), "{}", terminating.text());
    let source = source::production();
    let index = Interfaces::shared_production();
    let marks = Marks::contracts(records()).unwrap();
    let good = check(&index, &function(&source, READS), &marks);
    evidence(
        "owned-clone-no-receipt",
        &check(
            &index,
            &function(
                &source,
                "{ let Expr::Call { head, args } = expr else { return false; }; let copied = args[0].clone(); unknown(copied); false }",
            ),
            &marks,
        ),
        &good,
        "live across call",
    );
}

#[test]
fn local_option_read_contract_controls() {
    use rum_core::{Expr, SharedExpr};
    use std::ops::Deref;
    let head = SharedExpr::new(Expr::symbol("System`RuleDelayed"));
    let borrowed = <SharedExpr as Deref>::deref(&head);
    assert!(std::ptr::eq(borrowed, head.as_ref()));
    assert!(!std::ptr::eq(borrowed, &Expr::symbol("System`RuleDelayed")));
    // The helper itself is private to runtime: its independent compiler
    // harness executes the None/Some false/Some true and borrowed identity
    // controls inside a scratch copy of its lexical module.
    let source = source::production();
    let index = Interfaces::shared_production();
    let f = function(&source, READS);
    let original = records();
    let green = check(&index, &f, &Marks::contracts(original.clone()).unwrap());
    for id in ["I02", "B13"] {
        for pending in [false, true] {
            let mut changed = original.clone();
            let record = changed.iter_mut().find(|c| c.id == id).unwrap();
            if pending {
                record.status = "Pending".into();
            } else {
                record.fingerprint.push('0');
            }
            let red = check(&index, &f, &Marks::contracts(changed).unwrap());
            evidence(
                &format!("{id}-{}", if pending { "pending" } else { "stale" }),
                &red,
                &green,
                if pending {
                    "Pending contract"
                } else {
                    "stale contract fingerprint"
                },
            );
        }
    }
    let mut changed = source.clone();
    for (module, item, _, _) in &mut changed.trait_impls {
        let mut parsed = ast::<syn::ItemImpl>(item);
        if types::exact(&source, module, &parsed.self_ty).ok() == Some(protocol::nominal(SHARED)) {
            for method in &mut parsed.items {
                if let syn::ImplItem::Fn(method) = method {
                    if method.sig.ident == "deref" {
                        method.block = syn::parse_quote!({ panic!() });
                    }
                }
            }
        }
        *item = tokens(&parsed);
    }
    evidence(
        "changed-deref",
        &check(
            &Interfaces::new(&changed),
            &f,
            &Marks::contracts(original.clone()).unwrap(),
        ),
        &green,
        "Deref implementation changed",
    );
    let mut changed = Source::clone(source);
    changed.functions.get_mut(HELPER).unwrap()[0].body = block("{ false }");
    evidence(
        "changed-helper",
        &check(
            &Interfaces::new(&changed),
            &f,
            &Marks::contracts(original).unwrap(),
        ),
        &green,
        "helper changed",
    );
}
