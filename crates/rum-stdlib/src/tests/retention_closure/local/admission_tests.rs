use super::super::source;
use super::*;

fn run(text: &str) -> Report {
    let source = source::fixture(&[("admissions", "admissions.rs", text)]);
    let index = Interfaces::new(&source);
    let mut marks = Marks::default();
    if let Ok(f) = resolve::function(&source, "fixture::admissions::barrier") {
        marks.certify(&index, f).unwrap();
    }
    check(
        &index,
        resolve::function(&source, "fixture::admissions::root").unwrap(),
        &marks,
    )
}

fn evidence(name: &str, red: &Report, green: &Report) {
    assert!(!red.rejected.is_empty());
    assert!(green.rejected.is_empty(), "{}", green.text());
    let dir = source::workspace().join("target/brief-540/falsifiers");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(format!("{name}.red-green.txt")),
        format!("RED\n{}GREEN 0 rejections\n", red.text()),
    )
    .unwrap();
}

#[test]
fn local_retention_is_temporal_not_construction() {
    let bad = run("fn barrier() {} fn root(raw: Vec<u8>) { barrier(); }");
    assert!(
        bad.rejected
            .iter()
            .any(|r| r.kind == Kind::RET001 && r.reason.contains("live across call"))
    );
    let good = run("fn barrier() {} fn root(raw: Vec<u8>) { { let retired = raw; } barrier(); }");
    evidence("raw-move-before-call", &bad, &good);
    let wildcard = run("fn barrier() {} fn root(raw: Vec<u8>) { let _ = raw; barrier(); }");
    evidence("wildcard-does-not-move", &wildcard, &good);
    let shadow = run("fn barrier() {} fn root(raw: Vec<u8>) { let raw=1usize; barrier(); }");
    evidence("shadow-does-not-drop", &shadow, &good);
    let returned = run("fn root(raw: Vec<u8>) -> Vec<u8> { raw }");
    assert!(returned.rejected.is_empty(), "{}", returned.text());
    let dropped = run("fn root(raw: Vec<u8>) { let _ = raw; }");
    assert!(dropped.rejected.is_empty(), "{}", dropped.text());
    let borrowed = run("fn barrier() {} fn root(raw: Vec<u8>) { let _borrow = &raw; barrier(); }");
    evidence("borrow-keeps-owner", &borrowed, &good);
    let partial = run(
        "fn barrier() {} fn root(raw: (Vec<u8>,Vec<u8>)) { { let retired = raw.0; } barrier(); }",
    );
    evidence("partial-move-keeps-sibling", &partial, &good);
}

#[test]
fn local_result_and_operator_edges() {
    let same = run("fn root(x: Result<usize,u8>) -> Result<usize,u8> { let y=x?; Ok(y+1) }");
    let converted = run("fn root(x: Result<usize,u8>) -> Result<usize,i32> { let y=x?; Ok(y+1) }");
    assert!(converted.text().contains("identical concrete Result error"));
    evidence("try-identical-error", &converted, &same);
    let unknown = run("fn root(x: u64) -> u64 { x+1 }");
    assert!(
        unknown.text().contains("overloaded operator")
            || unknown.text().contains("exact type is unknown")
    );
    evidence("primitive-operator-edge", &unknown, &same);
    let closure = run("fn root() { let _f=|| 1usize; }");
    assert!(closure.text().contains("closure has no admission rule"));
    evidence("escaping-closure", &closure, &same);
    let signed = run("fn root() { for _i in 0i32..2i32 {} }");
    assert!(signed.text().contains("range bounds must both be usize"));
    evidence("range-exact-usize", &signed, &same);
}

#[test]
fn local_pending_argument_and_wrapped_payload_controls() {
    fn with_contracts(text: &str) -> Report {
        let source = source::fixture(&[("admissions", "admissions.rs", text)]);
        let index = Interfaces::new(&source);
        let mut marks = Marks::contracts(
            contracts::read(include_str!("../inventory/local-contracts.tsv")).unwrap(),
        )
        .unwrap();
        marks
            .certify(
                &index,
                resolve::function(&source, "fixture::admissions::barrier").unwrap(),
            )
            .unwrap();
        check(
            &index,
            resolve::function(&source, "fixture::admissions::root").unwrap(),
            &marks,
        )
    }
    let red =
        with_contracts("fn barrier() {} fn root() { let _ = (\"x\".repeat(1usize), barrier()); }");
    assert!(red.text().contains("live across call"), "{}", red.text());
    let green =
        with_contracts("fn barrier() {} fn root() { let _ = (barrier(), \"x\".repeat(1usize)); }");
    evidence("pending-tuple-operand", &red, &green);
    let red = with_contracts(
        "fn barrier() {} fn root() { let borrowed = &\"x\".repeat(1usize); barrier(); }",
    );
    evidence("temporary-borrow-lifetime", &red, &green);
    let red = with_contracts(
        "trait Noise { fn repeat(&self, n:usize) -> String; } fn barrier() {} fn root() { let _ = \"x\".repeat(1usize); }",
    );
    assert!(red.text().contains("imported trait candidate"));
    evidence("competing-method-trait", &red, &green);

    let source = source::test_module(
        source::production(),
        "crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs",
        "rum_runtime::runtime::accounting_mode::tests",
    );
    let f = resolve::function(
        &source,
        "rum_runtime::runtime::accounting_mode::tests::start",
    )
    .unwrap();
    let marks = Marks::contracts(
        contracts::read(include_str!("../inventory/local-contracts.tsv")).unwrap(),
    )
    .unwrap();
    let index = Interfaces::new(&source);
    let green = check(&index, f, &marks);
    let mut changed = f.clone();
    changed.body=syn::parse_str("{ let payload = Expr::string(\"prefix538\".repeat(150_000)); cx.reserve(&mut values, 32)?; cx.push(&mut values, payload)?; drive(cx, values, args, 1) }").unwrap();
    let red = check(&index, &changed, &marks);
    assert!(red.text().contains("live across call"), "{}", red.text());
    evidence("wrapper-does-not-hide-payload", &red, &green);
    changed.body=syn::parse_str("{ let payload = if args[0] == Expr::small_integer(1) { Expr::string(\"x\".repeat(2)) } else { Expr::small_integer(2) }; cx.reserve(&mut values, 32)?; cx.push(&mut values, payload)?; drive(cx, values, args, 1) }").unwrap();
    let red = check(&index, &changed, &marks);
    assert!(red.text().contains("live across call"), "{}", red.text());
    evidence("branch-preserves-payload", &red, &green);
    changed.body = syn::parse_str(
        "{ let copied = args.clone(); cx.reserve(&mut values, 32)?; drive(cx, values, args, 1) }",
    )
    .unwrap();
    let red = check(&index, &changed, &marks);
    assert!(red.text().contains("live across call"), "{}", red.text());
    evidence("shared-clone-has-no-receipt", &red, &green);
}
