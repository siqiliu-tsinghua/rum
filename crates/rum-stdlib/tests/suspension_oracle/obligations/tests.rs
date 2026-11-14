use std::collections::{BTreeMap, BTreeSet};

use super::{crosswalk::*, discovery::*, identity::*, imports::*};

const FIXTURE: &str = r#"
struct Session;
struct EvalError;
struct Guard;
impl Drop for Guard { fn drop(&mut self) { let _ = 1; } }
fn leaf(s: &mut Session) -> Result<i32, EvalError> { Ok(1) }
fn driver(s: &mut Session) -> Result<i32, EvalError> { let x = leaf(s)?; Ok(x + 1) }
fn repeated(s: &mut Session) -> Result<i32, EvalError> { let a = leaf(s)?; let b = leaf(s)?; Ok(a + b) }
trait TypedFrame { fn resume(self, s: &mut Session) -> Result<i32, EvalError>; }
struct Frame;
impl TypedFrame for Frame { fn resume(self, s: &mut Session) -> Result<i32, EvalError> { leaf(s) } }
fn typed(s: &mut Session, f: Frame) -> Result<i32, EvalError> { f.resume(s) }
fn macro_call(s: &mut Session) -> Result<i32, EvalError> { resume_or_suspend!(s, leaf(s), ok(value) => Ok(value), suspend => Frame {}) }
fn typed_macro(s: &mut Session) -> Result<i32, EvalError> { resume_typed_or_suspend!(s, leaf(s), ok(value) => Ok(value), suspend => Frame {}) }
fn supplemental(s: &mut Session) -> Result<i32, EvalError> { (leaf)(s) }
fn hop(s: &mut Session) -> Result<i32, EvalError> { leaf(s) }
fn root(s: &mut Session) -> Result<i32, EvalError> { hop(s) }
"#;

fn source(text: &str) -> SourceTree {
    SourceTree {
        files: vec![SourceFile {
            path: "fixture.rs".into(),
            module: "fixture".into(),
            text: text.into(),
        }],
    }
}
fn graph(text: &str) -> DiscoveredGraph {
    discover(&source(text), FeatureMode::current())
}
fn snapshot(graph: &DiscoveredGraph) -> AcceptedInputs {
    let mut inputs = AcceptedInputs::default();
    for (ordinal, (key, row)) in graph.rows.iter().enumerate() {
        let alias = format!("fixture.{ordinal:03}");
        let mut row = row.clone();
        row.aliases.push(alias.clone());
        row.policies.push("ForwardResult".into());
        inputs.population.rows.insert(key.clone(), row.clone());
        inputs.population.aliases.insert(
            alias.clone(),
            AliasRecord {
                alias,
                kind: "row".into(),
                target: Some(key.clone()),
                cohort: "fixture".into(),
                owner: "fixture".into(),
                disposition: "Open".into(),
                policies: row.policies,
                keys: 0,
                encoding: 0,
                function: row.legacy_function,
                tokens: row.tokens,
                provenance: "copied-source-fixture".into(),
                paired: false,
                strict_root: false,
                verdict: String::new(),
                key_digest: "-".into(),
            },
        );
    }
    inputs
}
fn red(label: &str, diagnostics: &[Diagnostic], kind: Kind) {
    assert!(
        diagnostics.iter().any(|d| d.kind == kind),
        "{label}: wanted {kind:?}, got {diagnostics:?}"
    );
    assert!(
        !diagnostics
            .iter()
            .any(|d| matches!(d.kind, Kind::PortBlocked | Kind::PinMismatch)),
        "wrong earlier predicate: {diagnostics:?}"
    );
    println!("FALSIFIER {label}: PASS rejected={kind:?}");
}

#[test]
fn obligations_inputs_exact() {
    assert_eq!(
        digest(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        digest(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        digest(&vec![b'a'; 1_000_000]),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
    let inputs = AcceptedInputs::committed().unwrap();
    assert_eq!(inputs.missing.len(), 4);
    assert_eq!(inputs.metrics["accepted_aliases"], 2204);
    assert_eq!(inputs.metrics["original_population"], 2186);
    assert_eq!(inputs.metrics["g13_owned_baseline"], 2027);
    assert_eq!(inputs.metrics["policies"], 25);
    assert_eq!(inputs.metrics["unresolved_pairings"], 24);
    assert!(
        inputs
            .pins
            .iter()
            .any(|(p, n, h)| p == ".temp/g15-c2/c68/required-tuples.tsv"
                && *n == 32_312_096
                && h == "3c2dcdff623890341df3fb71e9d7e3810b22bd233b7ffac80dc7f573c99fcc1f")
    );
    assert!(
        table("a\tb\nx\ty\nx\ty\n", "a\tb")
            .unwrap_err()
            .iter()
            .any(|e| e.kind == Kind::DuplicateInput)
    );
    assert_eq!(members("A;A").unwrap_err()[0].kind, Kind::DuplicateInput);
    assert_eq!(members("B;A").unwrap_err()[0].kind, Kind::DuplicateInput);
    for (name, bytes) in [
        ("inputs", INPUTS.len()),
        ("population", POPULATION.len()),
        ("aliases", ALIASES.len()),
        ("crosswalks", CROSSWALKS.len()),
        ("transfers", TRANSFERS.len()),
        ("owners", OWNERS.len()),
    ] {
        println!("INVENTORY {name}.tsv bytes={bytes}");
    }
    for path in &inputs.missing {
        println!("PortBlocked {path}");
    }
    println!("INPUT BASELINE PASS original=2186 owned=2027 policies=25 absent=4");
}

#[test]
fn obligations_alias_accounting_exact() {
    let inputs = AcceptedInputs::committed().unwrap();
    let aliases = &inputs.population.aliases;
    assert_eq!(aliases.len(), 2204);
    let count = |cohort: &str| aliases.values().filter(|a| a.cohort == cohort).count();
    assert_eq!(
        (
            count("c2-original"),
            count("c2-added"),
            count("g13"),
            count("prior")
        ),
        (1024, 18, 964, 198)
    );
    let c2: Vec<_> = aliases
        .values()
        .filter(|a| a.cohort.starts_with("c2-"))
        .collect();
    assert_eq!(c2.len(), 1042);
    assert_eq!(c2.iter().map(|a| a.keys).sum::<usize>(), 78330);
    assert_eq!(c2.iter().map(|a| a.encoding).sum::<usize>(), 18840);
    assert_eq!(c2.iter().filter(|a| !a.paired).count(), 24);
    assert_eq!(c2.iter().filter(|a| a.verdict == "Residual").count(), 118);
    assert!(c2.iter().all(|a| a.key_digest.len() == 64));
    for id in ["P0325", "P0326"] {
        assert_eq!((aliases[id].keys, aliases[id].encoding), (70, 20));
    }
    let mut prior = BTreeMap::new();
    for a in aliases.values().filter(|a| a.cohort == "prior") {
        *prior.entry(a.verdict.as_str()).or_insert(0) += 1;
    }
    assert_eq!(
        prior,
        BTreeMap::from([
            ("Boundary", 10),
            ("Equal", 43),
            ("Lost", 5),
            ("Unreachable", 140)
        ])
    );
    let g13: BTreeSet<_> = aliases
        .values()
        .filter(|a| a.cohort == "g13")
        .map(|a| &a.owner)
        .collect();
    assert_eq!(g13.len(), 44);
    for owner in g13 {
        let spec = &inputs.owners[owner];
        assert_eq!((&spec[0], &spec[1]), (&"g13".into(), &"Open".into()));
        for (position, disposition) in [(2, "OwnedObligation"), (3, "Equal"), (4, "Lost")] {
            assert_eq!(
                spec[position].parse::<usize>().unwrap(),
                aliases
                    .values()
                    .filter(|a| &a.owner == owner
                        && (if disposition == "OwnedObligation" {
                            &a.disposition
                        } else {
                            &a.verdict
                        }) == disposition)
                    .count(),
                "{owner}:{disposition}"
            );
        }
    }
    assert_eq!(&inputs.owners["S10"][2..5], &["28", "10", "1"]);
    assert_eq!(
        inputs
            .owners
            .values()
            .filter(|v| v[0] == "source" && v[1] == "Open")
            .count(),
        5
    );
    assert_eq!(
        inputs
            .population
            .transfers
            .iter()
            .filter(|t| !t.retained)
            .count(),
        513
    );
    assert_eq!(
        inputs
            .population
            .transfers
            .iter()
            .filter(|t| t.retained)
            .count(),
        283
    );
    assert_eq!(aliases.values().filter(|a| a.strict_root).count(), 283);
    for a in aliases.values() {
        assert!(a.policies.windows(2).all(|p| p[0] < p[1]), "{}", a.alias);
        assert!(!a.provenance.is_empty());
        if a.cohort.starts_with("c2-") {
            assert_eq!(a.owner, "G15-C2");
        }
        if let Some(key) = &a.target {
            assert!(inputs.population.rows[key].aliases.contains(&a.alias));
        }
    }
    println!(
        "ALIAS ACCOUNTING PASS accepted=2204 original_c2_replaced=1024 added=18 c2=1042 keys=78330 encoding=18840 g13_deferred=964 owners=44 prior=198 unresolved_pairings=24 S10=28+10+1 C2_Open_Residual=118"
    );
}

#[test]
fn obligations_discovery_shape() {
    let original = graph(FIXTURE);
    assert!(
        original.diagnostics.is_empty(),
        "{:?}",
        original.diagnostics
    );
    let expected = snapshot(&original);
    assert!(reconcile(&original, &expected).is_ok());
    for pass in ["lexical", "macro", "typed-macro", "typed", "supplemental"] {
        assert!(
            original.rows.values().any(|r| r.pass == pass),
            "missing pass {pass}"
        );
    }
    let leaf = "fn leaf(s: &mut Session) -> Result<i32, EvalError> { Ok(1) }";
    let driver =
        "fn driver(s: &mut Session) -> Result<i32, EvalError> { let x = leaf(s)?; Ok(x + 1) }";
    let mut moved = source(&FIXTURE.replace(leaf, "").replace(driver, ""));
    moved.files.push(SourceFile {
        path: "relocated.rs".into(),
        module: "fixture".into(),
        text: format!("{leaf}\n{driver}"),
    });
    let moved = discover(&moved, FeatureMode::current());
    assert_eq!(
        original.rows.keys().collect::<Vec<_>>(),
        moved.rows.keys().collect::<Vec<_>>()
    );
    assert!(reconcile(&moved, &expected).is_ok());
    let keys: BTreeSet<_> = original
        .rows
        .values()
        .map(|r| ObligationKey {
            edge: r.edge.clone(),
            policy: "ForwardResult".into(),
        })
        .collect();
    assert_eq!(keys.len(), original.rows.len());
    let pure_source = "fn pure(v: i32) -> i32 { v } fn historical_control() -> i32 { pure(1) }";
    let pure = graph(pure_source);
    assert!(pure.rows.is_empty());
    assert_eq!(pure.source_rows.len(), 1);
    let mut indexed = pure.clone();
    indexed.rows = pure.source_rows.clone();
    let accepted_control = snapshot(&indexed);
    assert!(reconcile(&pure, &accepted_control).is_ok());
    red(
        "changed-historical-pure-call",
        &reconcile(
            &graph(&pure_source.replace("pure(1)", "pure(2)")),
            &accepted_control,
        )
        .unwrap_err(),
        Kind::MissingAcceptedKey,
    );
    println!(
        "DISCOVERY SHAPE PASS moved-function=same-keys passes=lexical,macro,typed-macro,typed,supplemental"
    );
}

#[test]
fn obligations_discovery_negatives() {
    let original = graph(FIXTURE);
    let expected = snapshot(&original);
    for (label, old, new, kind) in [
        (
            "add-evaluating-call",
            "Ok(x + 1)",
            "let y = leaf(s)?; Ok(x + y)",
            Kind::NewEdge,
        ),
        (
            "delete-indexed-call",
            "let x = leaf(s)?;",
            "let x = 1;",
            Kind::MissingAcceptedKey,
        ),
        (
            "insert-identical-occurrence",
            "let b = leaf(s)?;",
            "let c = leaf(s)?; let b = leaf(s)?;",
            Kind::AmbiguousOccurrence,
        ),
        (
            "reorder-identical-occurrences",
            "let a = leaf(s)?; let b = leaf(s)?;",
            "let b = leaf(s)?; let a = leaf(s)?;",
            Kind::AmbiguousOccurrence,
        ),
        (
            "same-line-tail-token",
            "Ok(x + 1)",
            "Ok(x + 2)",
            Kind::StaleTail,
        ),
        (
            "omit-typed-edge",
            "f.resume(s)",
            "Ok(1)",
            Kind::MissingAcceptedKey,
        ),
        (
            "omit-macro-edge",
            "s, leaf(s), ok(value)",
            "s, Ok(1), ok(value)",
            Kind::MissingAcceptedKey,
        ),
        (
            "omit-supplemental-edge",
            "(leaf)(s)",
            "Ok(1)",
            Kind::MissingAcceptedKey,
        ),
        (
            "unresolved-dispatch",
            "f.resume(s)",
            "unknown.resume(s)",
            Kind::AmbiguousDispatch,
        ),
    ] {
        assert!(FIXTURE.contains(old));
        let changed = graph(&FIXTURE.replace(old, new));
        red(label, &reconcile(&changed, &expected).unwrap_err(), kind);
    }
    let changed =
        graph(&FIXTURE.replace("let x = leaf(s)?;", "opaque_macro!(leaf(s)); let x = 1;"));
    red(
        "unknown-macro",
        &reconcile(&changed, &expected).unwrap_err(),
        Kind::UnknownConstruct,
    );
    let changed = graph(&FIXTURE.replace("f.resume(s)", "(|f| f.resume(s))(unknown)"));
    red(
        "shadowed-dispatch",
        &reconcile(&changed, &expected).unwrap_err(),
        Kind::AmbiguousDispatch,
    );
    let changed = graph(&FIXTURE.replace(
        "let x = leaf(s)?;",
        "let callback = || leaf(s); let x = callback()?;",
    ));
    let diagnostics = reconcile(&changed, &expected).unwrap_err();
    red(
        "untyped-local-callback",
        &diagnostics,
        Kind::AmbiguousDispatch,
    );
    assert!(
        diagnostics
            .iter()
            .any(|d| d.kind == Kind::AmbiguousDispatch && d.detail.contains("callback ()"))
    );
}

fn transfer_fixture(graph: &DiscoveredGraph) -> Population {
    let source = graph
        .rows
        .iter()
        .find(|(_, r)| r.legacy_function == "fixture.rs:::hop")
        .unwrap();
    let root = graph
        .rows
        .iter()
        .find(|(_, r)| r.legacy_function == "fixture.rs:::root")
        .unwrap();
    let mut population = Population {
        rows: graph.rows.clone(),
        ..Population::default()
    };
    population.aliases.insert(
        "R".into(),
        AliasRecord {
            alias: "R".into(),
            kind: "row".into(),
            target: Some(root.0.clone()),
            cohort: "fixture".into(),
            owner: "fixture".into(),
            disposition: "Open".into(),
            policies: vec!["ForwardResult".into()],
            keys: 0,
            encoding: 0,
            function: root.1.legacy_function.clone(),
            tokens: root.1.tokens.clone(),
            provenance: "fixture".into(),
            paired: false,
            strict_root: true,
            verdict: String::new(),
            key_digest: "-".into(),
        },
    );
    population.transfers.push(Transfer {
        alias: "T".into(),
        source: Some(source.0.clone()),
        callers: BTreeSet::from(["R".into()]),
        roots: BTreeSet::from(["R".into()]),
        tail: source.1.tail.clone(),
        destructors: source.1.destructors.clone(),
        retained: false,
    });
    population
}

#[test]
fn obligations_transfer_fixed_point() {
    let g = graph(FIXTURE);
    let population = transfer_fixture(&g);
    let (closure, errors) = transfer_fixed_point(&population, &g);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(closure["T"], BTreeSet::from(["R".into()]));
    println!("TRANSFER FIXED POINT PASS indexed-caller=R concrete-root=R");
}

#[test]
fn obligations_transfer_negatives() {
    let g = graph(FIXTURE);
    let population = transfer_fixture(&g);
    let removed = graph(&FIXTURE.replace(
        "fn root(s: &mut Session) -> Result<i32, EvalError> { hop(s) }",
        "",
    ));
    red(
        "missing-transfer-target",
        &transfer_fixed_point(&population, &removed).1,
        Kind::MissingTransferTarget,
    );
    let changed = graph(&FIXTURE.replace("let _ = 1;", "let _ = 2;"));
    red(
        "post-producer-destructor",
        &transfer_fixed_point(&population, &changed).1,
        Kind::StaleDestructor,
    );
    let changed = graph(&FIXTURE.replace(
        "fn hop(s: &mut Session) -> Result<i32, EvalError> { leaf(s) }",
        "fn hop(s: &mut Session) -> Result<i32, EvalError> { let _guard = Guard; leaf(s) }",
    ));
    red(
        "post-producer-live-guard",
        &transfer_fixed_point(&population, &changed).1,
        Kind::StaleDestructor,
    );
    let cyclic = graph(
        "struct Session; struct EvalError; fn a(s: &mut Session) -> Result<i32,EvalError> { b(s) } fn b(s: &mut Session) -> Result<i32,EvalError> { a(s) }",
    );
    let mut cycle = Population::default();
    for row in cyclic.rows.values() {
        let a = if row.legacy_function == "fixture.rs:::a" {
            "A"
        } else {
            "B"
        };
        let b = if a == "A" { "B" } else { "A" };
        cycle.transfers.push(Transfer {
            alias: a.into(),
            source: Some(row.edge.key()),
            callers: BTreeSet::from([b.into()]),
            roots: BTreeSet::new(),
            tail: row.tail.clone(),
            destructors: row.destructors.clone(),
            retained: false,
        });
    }
    red(
        "A-B-A-without-root",
        &transfer_fixed_point(&cycle, &cyclic).1,
        Kind::RootlessCycle,
    );
}

#[test]
#[ignore = "explicit optional provenance audit; ordinary assertions never need historical files"]
fn obligations_audit_provenance() {
    let inputs = AcceptedInputs::committed().unwrap();
    let diagnostics = inputs.audit_pins(&super::super::coverage::source::workspace());
    for d in &diagnostics {
        println!("{}", d.line());
    }

    assert!(!diagnostics.iter().any(|d| d.kind == Kind::PinMismatch));
    println!(
        "PROVENANCE AUDIT PASS pins={} diagnostics={}",
        inputs.pins.len(),
        diagnostics.len()
    );
}

#[test]
#[ignore = "one explicit workspace discovery campaign per feature mode; no guests or settlements"]
fn obligations_discovery_campaign() {
    let start = std::time::Instant::now();
    let root = super::super::coverage::source::workspace();
    let g = discover(&SourceTree::workspace(&root), FeatureMode::current());
    println!("SOURCE_ROWS_BEGIN\n{ROW_HEADER}");
    for (key, row) in &g.source_rows {
        println!("{}", row_fields(key, row));
    }
    println!("SOURCE_ROWS_END");
    let inputs = AcceptedInputs::committed().unwrap();
    for alias in inputs.population.aliases.values() {
        println!(
            "ALIAS {}\t{}\t{}",
            alias.alias,
            alias.kind,
            alias.target.as_deref().unwrap_or("PortBlocked")
        );
    }
    let diagnostics = reconcile(&g, &inputs).err().unwrap_or_default();
    let mut kinds = BTreeMap::new();
    for d in &diagnostics {
        *kinds.entry(d.kind).or_insert(0) += 1;
        println!("{}", d.line());
    }
    println!(
        "IDENTITY DELTA new={} missing={} ambiguous_dispatch={} ambiguous_occurrence={} stale_tail={} stale_destructor={} unknown={} port_blocked={}",
        kinds.get(&Kind::NewEdge).unwrap_or(&0),
        kinds.get(&Kind::MissingAcceptedKey).unwrap_or(&0),
        kinds.get(&Kind::AmbiguousDispatch).unwrap_or(&0),
        kinds.get(&Kind::AmbiguousOccurrence).unwrap_or(&0),
        kinds.get(&Kind::StaleTail).unwrap_or(&0),
        kinds.get(&Kind::StaleDestructor).unwrap_or(&0),
        kinds.get(&Kind::UnknownConstruct).unwrap_or(&0),
        kinds.get(&Kind::PortBlocked).unwrap_or(&0)
    );
    println!(
        "DISCOVERY CAMPAIGN files={} functions={} calls={} edges={} resolved_source_edges={} accepted_aliases={} source_mapped={} diagnostics={kinds:?} elapsed_ms={}",
        g.files,
        g.functions,
        g.calls,
        g.rows.len(),
        g.source_rows.len(),
        inputs.population.aliases.len(),
        inputs
            .population
            .aliases
            .values()
            .filter(|a| a.target.is_some())
            .count(),
        start.elapsed().as_millis()
    );
    let (closure, _) = transfer_fixed_point(&inputs.population, &g);
    println!(
        "TRANSFER GRAPH excluded={} retained_roots={} concrete_root_sets={} settled=0 promoted=0 guests=0",
        inputs
            .population
            .transfers
            .iter()
            .filter(|t| !t.retained)
            .count(),
        inputs
            .population
            .transfers
            .iter()
            .filter(|t| t.retained)
            .count(),
        closure.values().filter(|r| !r.is_empty()).count()
    );
}
