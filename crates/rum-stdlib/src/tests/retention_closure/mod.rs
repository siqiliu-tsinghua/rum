//! Local instance certificates plus historical restricted fixture measurements.

mod catalog;
mod census;
mod compiler_oracle;
mod constructor_tests;
mod consumers;
mod effects;
mod fixtures;
mod leaves;
mod local;
mod method_tests;
mod methods;
mod model;
mod mutations;
mod obligations;
mod regions;
mod resolve;
mod source;
mod types;

use std::collections::BTreeMap;

use model::{Kind, Ty};

fn contracts() -> Vec<leaves::Leaf> {
    leaves::read(include_str!("inventory/leaves.tsv")).unwrap()
}

#[test]
fn production_obligations_are_exact() {
    let started = std::time::Instant::now();
    let source = source::production();
    assert!(
        source.errors.is_empty(),
        "module reconciliation: {:?}",
        source.errors
    );
    println!(
        "MODULES={} RECONCILED_FILES={} PRODUCTION_FILES={} FUNCTION_IDENTITIES={}",
        source.modules.len(),
        source.reached.len(),
        source.production_files.len(),
        source.functions.len()
    );
    let roots = [
        format!("{}::merged_option_exprs_for_symbol", regions::SESSION),
        format!("{}::evaluate_runtime_option_expr", regions::SESSION),
    ];
    let frontier = regions::frontier(&source, &roots);
    let files = frontier
        .bodies
        .iter()
        .map(|id| &resolve::function(&source, id).unwrap().site.file)
        .collect::<std::collections::BTreeSet<_>>();
    println!(
        "OPTION_FRONTIER resolved_bodies={} resolved_files={} rejected_edges={} name_spelling_candidates=2633 candidate_files=349",
        frontier.bodies.len(),
        files.len(),
        frontier.rejected.len()
    );
    for id in &frontier.bodies {
        println!(
            "RESOLVED\t{id}\t{}",
            resolve::function(&source, id).unwrap().site.display()
        );
    }
    for error in &frontier.rejected {
        println!("REJECTED\t{error}");
    }
    let public = regions::frontier(
        &source,
        &[format!(
            "{}::merged_option_exprs_for_system_symbol",
            regions::SESSION
        )],
    );
    let files = public
        .bodies
        .iter()
        .map(|id| &resolve::function(&source, id).unwrap().site.file)
        .collect::<std::collections::BTreeSet<_>>();
    println!(
        "PUBLIC_WRAPPER resolved_bodies={} resolved_files={} rejected_edges={} comparison_C01_C02_name_spelling_candidates=2633 candidate_files=349",
        public.bodies.len(),
        files.len(),
        public.rejected.len()
    );
    for id in &public.bodies {
        println!(
            "PUBLIC_RESOLVED\t{id}\t{}",
            resolve::function(&source, id).unwrap().site.display()
        );
    }
    for error in &public.rejected {
        println!("PUBLIC_REJECTED\t{error}");
    }
    regions::proposed_boundaries(&source);
    consumers::verify(&source);
    catalog::installed();
    let classes = obligations::census(&source);
    let routes = catalog::catalog_routes(&source);
    for (name, class) in &classes {
        let targets = routes.get(name).cloned().unwrap_or_default();
        if class == "N" {
            assert!(
                !targets.is_empty(),
                "N declaration has no resolved or explicitly rejected catalog attachment: {name}"
            );
        }
        println!(
            "OBLIGATION_ROUTE\t{name}\t{}",
            targets.into_iter().collect::<Vec<_>>().join(",")
        );
    }
    let mut recorded = BTreeMap::new();
    for line in include_str!("inventory/obligations.tsv")
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 8, "complete witness record: {line}");
        assert_eq!(
            fields[3],
            if ["C01", "C02"].contains(&fields[0]) {
                "LocalCovered"
            } else {
                "Pending"
            },
            "only the two exact local production bodies are promoted"
        );
        assert!(
            recorded.insert(fields[1], fields.clone()).is_none(),
            "duplicate obligation"
        );
    }
    assert_eq!(recorded.len(), classes.len() + 2);
    for (name, class) in classes {
        let row = &recorded[name.as_str()];
        assert_eq!(row[0], format!("{class}:{name}"));
        assert_eq!(row[2], class);
        let targets = routes
            .get(&name)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(row[7], targets, "canonical registered roots for {name}");
        assert!(!row[4].is_empty() && !row[5].is_empty() && !row[6].is_empty());
    }
    for (name, root) in ["C01", "C02"].into_iter().zip(roots) {
        assert_eq!(recorded[name][7], root);
    }
    local::verify_promotions(&source);
    println!(
        "PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315"
    );
    println!(
        "PRODUCTION_WALL_SECONDS={:.3}",
        started.elapsed().as_secs_f64()
    );
}

#[test]
fn restricted_surface_rejects_six_shapes() {
    let source = fixtures::program();
    for name in fixtures::SIX {
        let result = effects::check(&source, &format!("fixture::{name}::root"), &contracts());
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("falsifier {name} was accepted"),
        };
        assert_eq!(
            error,
            fixtures::expected(name),
            "pinned rejection for {name}"
        );
        println!("PINNED\t{name}\t{error}");
    }
    // Independent compiler-checked spellings exercise the same semantic bans.
    for (root, kind) in [
        ("aliased_collect::adaptor", Kind::RET002),
        ("formatted_string::alias", Kind::RET003),
        ("formatted_string::writer", Kind::RET003),
        ("map_entry_callback::raw", Kind::RET001),
    ] {
        let error = match effects::check(&source, &format!("fixture::{root}"), &contracts()) {
            Err(error) => error,
            Ok(_) => panic!("variant {root} accepted"),
        };
        assert_eq!(error.kind, kind);
        assert!(error.site.start.0 > 0 && error.site.end > error.site.start);
    }
}

#[test]
fn extracted_helpers_enter_closure() {
    let source = fixtures::program();
    let proof =
        effects::check(&source, regions::DEMO, &contracts()).unwrap_or_else(|e| panic!("{e}"));
    assert!(proof.bodies.contains("fixture::accepted::next"));
    let error = match effects::check(&source, "fixture::extracted_allocator::root", &contracts()) {
        Err(e) => e,
        Ok(_) => panic!("extracted allocation escaped"),
    };
    assert_eq!(error, fixtures::expected("extracted_allocator"));
    let source = mutations::extracted();
    let error = regions::verify(
        &source,
        &regions::baseline(),
        &regions::baseline(),
        &contracts(),
    )
    .err()
    .expect("unmarked extracted helper must invalidate coverage");
    assert!(error.contains("fixture::extracted_allocator::allocate"));
    assert!(error.contains("RET001"));
}

fn rejection(text: &str) -> model::Diagnostic {
    let source = mutations::probe(text);
    match effects::check(&source, "fixture::probe::root", &contracts()) {
        Err(error) => error,
        Ok(_) => panic!("mutation accepted: {text}"),
    }
}

#[test]
fn unresolved_edges_are_errors() {
    let imports =
        source::fixture(&[("imports", "imports.rs", include_str!("fixtures/imports.rs"))]);
    let proof = effects::check(&imports, "fixture::imports::root", &contracts()).unwrap();
    assert_eq!(
        proof.bodies.len(),
        2,
        "cyclic globs converge on canonical bodies"
    );
    let scope = &imports.modules["fixture::imports"];
    assert!(
        scope
            .bindings
            .values()
            .flatten()
            .all(|b| !b.target.contains("std::format::"))
    );
    let write: syn::Path = syn::parse_str("Write").unwrap();
    assert!(
        resolve::path(&imports, "fixture::imports", &write)
            .unwrap_err()
            .contains("opaque import namespace")
    );
    let cycle = source::fixture(&[("cycle", "cycle.rs", "#[path = \"./cycle.rs\"] mod again;")]);
    assert_eq!(
        cycle.errors,
        ["recursive source path: cycle.rs in fixture::cycle::again"]
    );
    // A workspace crate is an extern-prelude root too. Passing through its
    // reexports must not let an opaque alias grow a self-referential suffix.
    let mut alias = source::fixture(&[("bridge", "bridge.rs", "use std::fmt as opaque;")]);
    alias
        .dependencies
        .get_mut("fixture")
        .unwrap()
        .insert("fixture".into());
    let import: syn::ItemUse =
        syn::parse_str("use fixture::bridge::opaque::Error as opaque;").unwrap();
    resolve::collect_imports(
        &mut alias,
        "fixture::bridge",
        &import.tree,
        Vec::new(),
        None,
        false,
    );
    resolve::imports(&mut alias);
    let bindings = &alias.modules["fixture::bridge"].bindings["opaque"];
    assert_eq!(
        bindings.len(),
        2,
        "canonical original plus explicit unresolved import"
    );
    assert!(
        bindings
            .iter()
            .all(|b| !b.target.contains("std::fmt::Error"))
    );
    for (text, reason) in [
        ("pub fn root() { missing(); }", "exact bindings"),
        ("pub fn root(f: fn()) { f(); }", "type is outside"),
        (
            "fn target() {} pub fn root() { let target = 1usize; target(); }",
            "shadows",
        ),
        ("pub fn root() { fn helper() {} helper(); }", "block-local"),
        (
            "#[cfg(feature = \"unknown\")] pub fn root() {}",
            "conditional",
        ),
        (
            "pub fn root() { <String as Default>::default(); }",
            "trait projection",
        ),
        (
            "type A = B; type B = A; pub fn root(x: A) {}",
            "recursive type/alias",
        ),
        (
            "pub fn root() { let a = super::accepted::substrate::start(); std::mem::forget(a); }",
            "not a resolved function",
        ),
    ] {
        let error = rejection(text);
        assert_eq!(error.kind, Kind::RET002, "{error}");
        assert!(error.reason.contains(reason), "expected {reason}: {error}");
    }
    let alias = rejection("type Hidden = Vec<usize>; pub fn root(values: Hidden) {}");
    assert_eq!(alias.kind, Kind::RET001);
    let source = source::fixture(&[(
        "probe",
        "probe.rs",
        "#[cfg(feature=\"one\")] mod branch { pub fn root() {} } #[cfg(not(feature=\"one\"))] mod branch { pub fn root() { unknown!(); } }",
    )]);
    assert_eq!(
        source.functions["fixture::probe::branch::root"].len(),
        2,
        "both non-test alternatives must enter discovery"
    );
    assert!(
        resolve::function(&source, "fixture::probe::branch::root")
            .err()
            .unwrap()
            .contains("conditional")
    );
}

#[test]
fn coverage_cannot_shrink() {
    let injected = mutations::inject("unleased_callback_result");
    let mut expected = fixtures::expected("unleased_callback_result");
    expected.witness[0] = regions::DEMO.into();
    let actual = effects::check(&injected, regions::DEMO, &contracts())
        .err()
        .expect("injected callback must invalidate promotion");
    assert_eq!(actual, expected);
    let source = fixtures::program();
    let baseline = regions::baseline();
    let error = regions::verify(&source, &[], &baseline, &contracts())
        .err()
        .unwrap();
    assert_eq!(error, "coverage record removed: demo");
    let mut records = baseline.clone();
    records[0].covered = false;
    assert_eq!(
        regions::verify(&source, &records, &baseline, &contracts())
            .err()
            .unwrap(),
        "promotion changed or downgraded: demo"
    );
    records[0] = baseline[0].clone();
    records[0].root = "fixture::accepted::next".into();
    assert_eq!(
        regions::verify(&source, &records, &baseline, &contracts())
            .err()
            .unwrap(),
        "promotion changed or downgraded: demo"
    );
    records[0] = baseline[0].clone();
    records[0].exit = Ty::Unit;
    assert_eq!(
        regions::verify(&source, &records, &baseline, &contracts())
            .err()
            .unwrap(),
        "promotion changed or downgraded: demo"
    );
    let mut unmarked = source.clone();
    unmarked.functions.get_mut(regions::DEMO).unwrap()[0]
        .attrs
        .clear();
    assert_eq!(
        regions::verify(&unmarked, &baseline, &baseline, &contracts())
            .err()
            .unwrap(),
        "root marker missing or redirected: demo"
    );
    let mut deleted = source;
    deleted.functions.remove(regions::DEMO);
    assert!(
        regions::verify(&deleted, &baseline, &baseline, &contracts())
            .err()
            .unwrap()
            .contains("not a resolved function definition")
    );
    assert!(
        regions::verify(&mutations::extracted(), &baseline, &baseline, &contracts())
            .err()
            .unwrap()
            .contains("RET001")
    );
}

#[test]
fn trusted_items_have_exact_bounds() {
    let source = fixtures::program();
    let leaves = contracts();
    let proof = effects::check(&source, regions::DEMO, &leaves).unwrap_or_else(|e| panic!("{e}"));
    leaves::validate(&source, &leaves, &proof.leaves).unwrap();
    let mut stale = leaves.clone();
    stale[0].record[11].push('0');
    assert!(
        leaves::validate(&source, &stale, &proof.leaves)
            .err()
            .unwrap()
            .contains("stale dependency")
    );
    let wildcard = include_str!("inventory/leaves.tsv").replacen(
        "fixture::accepted::substrate::start",
        "fixture::accepted::substrate::*",
        1,
    );
    assert_eq!(
        leaves::read(&wildcard).err().unwrap(),
        "wildcard trusted entry"
    );
    let mut unused = proof.leaves;
    unused.remove(&leaves[0].key());
    assert!(
        leaves::validate(&source, &leaves, &unused)
            .err()
            .unwrap()
            .contains("unexercised trusted entry")
    );
    let mut dependency = source;
    let file = dependency
        .files
        .iter_mut()
        .find(|(p, _)| p.ends_with("fixtures/accepted.rs"))
        .unwrap()
        .1;
    *file = file.replace("bytes: Vec<u8>", "bytes: Vec<u16>");
    let used = leaves.iter().map(|l| l.key()).collect();
    assert!(
        leaves::validate(&dependency, &leaves, &used)
            .err()
            .unwrap()
            .contains("stale dependency")
    );
}

#[test]
fn implicit_operations_are_classified() {
    for text in [
        "pub fn root() { for _ in 0usize..1usize {} }",
        "pub fn root() -> Result<(), ()> { Ok::<(), ()>(())?; Ok(()) }",
        "pub fn root() -> usize { [1usize][0] }",
        "struct Owner; impl Drop for Owner { fn drop(&mut self) { let _ = vec![0]; } } pub fn root(x: Owner) {}",
        "pub fn root() { (|| 1usize)(); }",
        "#[derive(PartialEq)] struct Key; pub fn root(a: Key, b: Key) -> bool { a == b }",
        "pub fn root() { let x = super::accepted::substrate::start(); let y = if true { x } else { super::accepted::substrate::start() }; }",
    ] {
        assert_eq!(rejection(text).kind, Kind::RET002);
    }
    let bare_error = rejection(
        "pub fn root(n: usize) -> super::accepted::substrate::Raw { super::accepted::substrate::callback(n) }",
    );
    assert_eq!(bare_error.kind, Kind::RET004);
    assert_eq!(
        bare_error.target.as_deref(),
        Some("fixture::accepted::substrate::callback")
    );
    let source = mutations::probe(
        "pub fn root(n: usize) -> usize { if n < 1usize { n } else { root(n - 1usize) } }",
    );
    let proof = effects::check(&source, "fixture::probe::root", &contracts())
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(proof.bodies.len(), 1, "recursive scalar SCC is closed");
}

#[test]
fn accepted_fixture_exercises_d2_shapes() {
    let source = mutations::promoted_map();
    assert!(source.errors.is_empty(), "{:?}", source.errors);
    let leaves = leaves::read(include_str!("inventory/leaves.tsv")).unwrap();
    let proof = regions::verify(&source, &regions::baseline(), &regions::baseline(), &leaves)
        .unwrap_or_else(|error| panic!("{error}"));
    leaves::validate(&source, &leaves, &proof.leaves).unwrap();
    assert_eq!(proof.bodies.len(), 2);
    assert_eq!(proof.leaves.len(), 5);
    assert_eq!(
        (proof.loop_invariants, proof.adoptions, proof.transfers),
        (1, 1, 1)
    );
    assert!(proof.facts.len() > 20);
    for fact in &proof.facts {
        assert!(fact.site.start.0 > 0 && fact.site.end > fact.site.start);
        assert!(!fact.effect.is_empty());
        assert!(
            !matches!(fact.ty, Ty::Raw(_)),
            "unleased fact in accepted fixture"
        );
    }
    println!(
        "ACCEPTED bodies={} leaves={} expression_facts={} loops={} adoptions={} transfers={}",
        proof.bodies.len(),
        proof.leaves.len(),
        proof.facts.len(),
        proof.loop_invariants,
        proof.adoptions,
        proof.transfers
    );
}
