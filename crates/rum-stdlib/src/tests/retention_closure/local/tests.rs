use super::super::{regions, source};
use super::*;

const FIXTURE: &str = include_str!("../fixtures/local.rs");

fn fixture(text: &str) -> Source {
    source::fixture(&[(
        "local",
        "crates/rum-stdlib/src/tests/retention_closure/fixtures/local.rs",
        text,
    )])
}

fn function<'a>(s: &'a Source, name: &str) -> &'a Function {
    resolve::function(s, &format!("fixture::local::{name}")).unwrap()
}

pub(super) fn save(name: &str, body: &str) {
    let path = source::workspace().join("target/brief-539/falsifiers");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join(name), body).unwrap();
}

#[test]
fn local_raw_type_falsifiers_and_closed_list() {
    let source = fixture(FIXTURE);
    let index = Interfaces::new(&source);
    for (name, body) in [
        ("direct", "let _raw = Vec::<u8>::new(); missing();"),
        ("aliased", "let _raw = Bytes::new(); missing();"),
        ("through_helper", "let _raw = allocator(); missing();"),
    ] {
        let mut marks = Marks::default();
        marks
            .certify(&index, function(&source, "unit_helper"))
            .unwrap();
        let changed = FIXTURE.replace(
            "pub fn unit_helper() {}",
            &format!("pub fn unit_helper() {{ {body} }}"),
        );
        assert_ne!(changed, FIXTURE);
        let mutant = fixture(&changed);
        let other = Interfaces::new(&mutant);
        let report = marks
            .certify(&other, function(&mutant, "unit_helper"))
            .unwrap_err();
        save(&format!("raw-{name}.rs"), &changed);
        save(&format!("raw-{name}.red.txt"), &report.text());
        assert!(
            report.rejected.iter().any(|d| d.kind == Kind::RET001
                && d.site.start.0 > 0
                && d.reason.starts_with("raw owned container produced:")),
            "{}",
            report.text()
        );
        let good = marks
            .certify(&index, function(&source, "unit_helper"))
            .unwrap();
        save(
            &format!("raw-{name}.green.txt"),
            &format!(
                "repaired same covered body; {} rejections\n",
                good.rejected.len()
            ),
        );
    }
    let mut exercised = BTreeSet::new();
    for name in ["vector", "boxed", "string", "btree", "hashmap", "hashset"] {
        let report = check(&index, function(&source, name), &Marks::default());
        assert!(
            report
                .rejected
                .iter()
                .any(|d| d.kind == Kind::RET001
                    && d.reason.starts_with("raw owned container produced:")),
            "{name}: {}",
            report.text()
        );
        for ty in report.facts.values() {
            if let ExactTy::Nominal(id, _) = ty {
                exercised.insert(id.clone());
            }
        }
    }
    assert_eq!(
        exercised,
        types::OWNED_CONTAINERS
            .iter()
            .map(|s| s.to_string())
            .collect(),
        "unexercised owned-container entry"
    );
}

#[test]
fn local_missing_mark_falsifier() {
    let source = fixture(FIXTURE);
    let index = Interfaces::new(&source);
    let mut marks = Marks::default();
    let bad = marks
        .certify(&index, function(&source, "missing"))
        .unwrap_err();
    save("missing-mark.red.txt", &bad.text());
    assert!(
        bad.text().contains("callee has no covered mark"),
        "{}",
        bad.text()
    );
    assert_eq!(marks.count(), 0, "rejected runs cannot mint marks");
    marks
        .certify(&index, function(&source, "unit_helper"))
        .unwrap();
    marks.certify(&index, function(&source, "missing")).unwrap();
    save(
        "missing-mark.green.txt",
        "helper linted independently, then caller; 2 fixture marks, 0 production marks\n",
    );
}

#[test]
fn local_unknown_type_falsifier() {
    let source = fixture(FIXTURE);
    let index = Interfaces::new(&source);
    let bad = check(
        &index,
        function(&source, "unknown_literal"),
        &Marks::default(),
    );
    save("unknown-type.red.txt", &bad.text());
    assert!(bad.text().contains("expression exact type is unknown"));
    let good = check(&index, function(&source, "scalar"), &Marks::default());
    assert!(good.rejected.is_empty(), "{}", good.text());
    save(
        "unknown-type.green.txt",
        "explicit u8 parameter and result are established; 0 rejections\n",
    );
}

#[test]
fn local_side_door_and_covered_freshness() {
    let source = fixture(FIXTURE);
    let index = Interfaces::new(&source);
    let mut changed = source.clone();
    let helper = changed
        .functions
        .get_mut("fixture::local::unit_helper")
        .unwrap()
        .first_mut()
        .unwrap();
    helper.body = syn::parse_str("{ let _raw = Vec::<u8>::new(); missing(); }").unwrap();
    let other = Interfaces::new(&changed);
    let a = check(&index, function(&source, "missing"), &Marks::default());
    let b = check(&other, function(&changed, "missing"), &Marks::default());
    save(
        "side-door.red.txt",
        &format!(
            "pure unmarked callee:\n{}raw unmarked callee:\n{}",
            a.text(),
            b.text()
        ),
    );
    assert_eq!(
        a.rejected, b.rejected,
        "caller verdict depends on callee body"
    );
    assert!(
        check(&index, function(&source, "unit_helper"), &Marks::default())
            .rejected
            .is_empty()
    );
    assert!(
        check(&other, function(&changed, "unit_helper"), &Marks::default())
            .rejected
            .iter()
            .any(|d| d.kind == Kind::RET001)
    );
    assert!(
        other
            .signatures
            .functions
            .values()
            .flatten()
            .all(|f| f.body.stmts.is_empty())
    );
    save(
        "side-door.green.txt",
        "caller diagnostics byte-identical with pure/raw callee; callee's own verdict changes; signature-only input\n",
    );
    let mut marks = Marks::default();
    marks
        .certify(&index, function(&source, "unit_helper"))
        .unwrap();
    marks.certify(&index, function(&source, "missing")).unwrap();
    let stale = check(&other, function(&changed, "missing"), &marks);
    save("covered-stale.red.txt", &stale.text());
    assert!(stale.text().contains("stale covered mark"));
    let own = marks
        .certify(&other, function(&changed, "unit_helper"))
        .unwrap_err();
    assert!(
        own.rejected.iter().any(|d| d.kind == Kind::RET001),
        "re-lint raw body despite prior mark"
    );
    let fresh = check(&index, function(&source, "missing"), &marks);
    assert!(fresh.rejected.is_empty());
    save(
        "covered-stale.green.txt",
        "original fingerprint remains valid; changed helper cannot re-earn mark\n",
    );
}

#[test]
fn local_contract_falsifiers() {
    let source = source::production();
    let index = Interfaces::new(&source);
    let records = contracts::read(include_str!("../inventory/local-contracts.tsv")).unwrap();
    assert_eq!(
        records
            .iter()
            .filter(|c| c.status == "Discharged" && !matches!(c.id.as_str(), "I02" | "B13"))
            .count(),
        56 // the original 31 plus B10/B11 and 23 explicit private rule records
    );
    assert_eq!(
        records
            .iter()
            .filter(|c| matches!(c.id.as_str(), "I02" | "B13") && c.status == "Discharged")
            .map(|c| c.id.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["I02", "B13"])
    );
    let marks = Marks::contracts(records.clone()).unwrap();
    for record in &records {
        let verdict = marks.lookup(&index, &record.target);
        if record.status == "Pending" {
            assert!(verdict.unwrap_err().contains("Pending"));
        } else {
            assert!(verdict.is_ok(), "{}: {verdict:?}", record.id);
        }
    }
    save(
        "pending.red.txt",
        &marks.lookup(&index, &contracts::target("B01")).unwrap_err(),
    );
    save(
        "pending.green.txt",
        "B01 remains Pending and unusable; only installed discharged records resolve to contract marks\n",
    );
    let attempted = include_str!("../inventory/local-contracts.tsv").replacen(
        "\tPending\t",
        "\tDischarged\t",
        1,
    );
    assert!(
        contracts::read(&attempted)
            .unwrap_err()
            .contains("only B03/B09/B12")
    );
    let mut stale = records;
    stale
        .iter_mut()
        .find(|c| c.id == "B09")
        .unwrap()
        .fingerprint
        .push('0');
    let stale = Marks::contracts(stale).unwrap();
    let mut caller = resolve::function(
        &source,
        &format!("{}::evaluate_runtime_option_expr", regions::SESSION),
    )
    .unwrap()
    .clone();
    caller.body = syn::parse_str("{ self.evaluate(expr); }").unwrap();
    let report = check(&index, &caller, &stale);
    save("contract-stale.red.txt", &report.text());
    assert!(
        report.text().contains("stale contract fingerprint B09"),
        "{}",
        report.text()
    );
    let good = check(&index, &caller, &marks);
    assert!(
        good.marks.contains_key(&contracts::target("B09")),
        "{}",
        good.text()
    );
    save(
        "contract-stale.green.txt",
        &format!(
            "B09 mark lookup succeeds; remaining caller obligations (not waived):\n{}",
            good.text()
        ),
    );
    let wildcard = include_str!("../inventory/local-contracts.tsv").replacen(
        &contracts::target("B01"),
        "*",
        1,
    );
    assert!(contracts::read(&wildcard).is_err());
}

#[test]
fn local_inventory_marks_are_earned() {
    let source = fixture(FIXTURE);
    let index = Interfaces::new(&source);
    let mut marks = Marks::default();
    let mut seen = BTreeSet::new();
    for id in include_str!("../inventory/local-covered.tsv")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        assert!(seen.insert(id), "duplicate covered candidate");
        assert!(
            id.starts_with("fixture::"),
            "production promotion forbidden"
        );
        marks
            .certify(&index, resolve::function(&source, id).unwrap())
            .unwrap();
    }
    assert_eq!(marks.count(), seen.len());
    save("earned-marks.tsv", &marks.inventory());
    let mut forged = function(&source, "unit_helper").clone();
    forged.body = syn::parse_str("{ () }").unwrap();
    assert!(
        marks
            .certify(&index, &forged)
            .unwrap_err()
            .text()
            .contains("differs from indexed fingerprint")
    );
}

#[test]
fn local_scope_fingerprint_falsifier() {
    // Both sources compile as standalone Rust. The public inherent method's
    // identity, signature and body text stay fixed, but `provider` changes its
    // binding when the impl moves. This is a lexical dependency, not an edge
    // to traverse during the caller's run.
    let template = "pub struct Thing;\nmod first {\n    pub fn provider() {}\nFIRST\n}\nmod second {\n    pub fn provider() { let _ = Vec::<u8>::new(); }\nSECOND\n}\npub fn caller(value: &Thing) { value.read539(); }\n";
    let implementation = "    impl super::Thing {\n        pub fn read539(&self) {\n            provider();\n        }\n    }";
    let before = template
        .replace("FIRST", implementation)
        .replace("SECOND", "");
    let after = template
        .replace("FIRST", "")
        .replace("SECOND", implementation);
    save("module-before.rs", &before);
    save("module-after.rs", &after);
    let a = fixture(&before);
    let b = fixture(&after);
    let first = Interfaces::new(&a);
    let second = Interfaces::new(&b);
    let mut marks = Marks::default();
    for id in [
        "fixture::local::first::provider",
        "fixture::local::Thing::read539",
        "fixture::local::caller",
    ] {
        marks
            .certify(&first, resolve::function(&a, id).unwrap())
            .unwrap();
    }
    let changed = check(&second, function(&b, "caller"), &marks);
    save("module-relocation.red.txt", &changed.text());
    assert!(
        changed.text().contains("stale covered mark"),
        "moving a method changed name resolution without expiring its mark: {}",
        changed.text()
    );
    let restored = check(&first, function(&a, "caller"), &marks);
    assert!(restored.rejected.is_empty(), "{}", restored.text());
    save(
        "module-relocation.green.txt",
        "restoring the defining module restores the independently earned caller mark\n",
    );
}

#[test]
fn local_admissions_never_exceed_a1_a2() {
    let drop = "struct Owner; impl Drop for Owner { fn drop(&mut self) { let _ = Vec::<u8>::new(); } } pub fn root(owner: Owner) {}";
    for (name, text) in [
        ("unused-drop", drop),
        (
            "mutable-borrow",
            "pub fn root() { let mut n = 1u8; let _ = &mut n; }",
        ),
        ("uninitialized-local", "pub fn root() { let value: u8; }"),
    ] {
        let source = fixture(text);
        let index = Interfaces::new(&source);
        let f = function(&source, "root");
        assert!(
            super::super::effects::check(&source, &f.id, &[]).is_err(),
            "A1 control changed for {name}"
        );
        let report = check(&index, f, &Marks::default());
        save(&format!("admission-{name}.red.txt"), &report.text());
        assert!(
            !report.rejected.is_empty(),
            "new admission beyond A1/A2: {name}"
        );
    }
    let source = fixture(
        "struct Owner; impl Drop for Owner { fn drop(&mut self) { let _ = Vec::<u8>::new(); } } pub fn root(owner: &Owner) {}",
    );
    let index = Interfaces::new(&source);
    let report = check(&index, function(&source, "root"), &Marks::default());
    assert!(
        report.rejected.is_empty(),
        "a borrow does not invoke the owner's destructor: {}",
        report.text()
    );
    save(
        "admission.green.txt",
        "borrowed-owner control passes; unused owned destructor, mutable borrow and uninitialized local stay rejected as in A1\n",
    );
}
