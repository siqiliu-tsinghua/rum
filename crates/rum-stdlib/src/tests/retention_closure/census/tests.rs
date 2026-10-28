use std::collections::{BTreeMap, BTreeSet};

use super::*;
use crate::tests::retention_closure::source;

#[test]
fn production_rejection_census_is_complete() {
    let source = source::production();
    assert!(source.errors.is_empty(), "{:?}", source.errors);
    let mut census = all(&source);
    let expected = definitions(&source);
    assert_eq!(
        expected.len(),
        source.functions.values().map(Vec::len).sum::<usize>()
    );
    let roots: Vec<_> = expected
        .iter()
        .map(|f| Root {
            name: format!("{}@{}", f.id, f.site.display()),
            site: f.site.clone(),
            target: Ok(f.id.clone()),
            spelling: f.id.clone(),
        })
        .collect();
    validate(&census, &expected, &roots).unwrap();
    assert!(census.visited.values().all(|rs| rs.len() == 1));
    let dir = report::write(&census, &expected, &roots);
    report::independent_audit(&dir);

    let removed = census.visited.pop_first().unwrap();
    assert_eq!(
        validate(&census, &expected, &roots).unwrap_err(),
        "census omitted or invented a production definition"
    );
    census.visited.insert(removed.0, removed.1);
    validate(&census, &expected, &roots).unwrap();
    baseline(&source);
}

#[test]
fn n_frontier_keeps_unresolved_roots() {
    let source = source::production();
    assert!(source.errors.is_empty(), "{:?}", source.errors);
    let roots = n_roots(&source);
    assert_eq!(roots.iter().filter(|r| r.target.is_err()).count(), 9);
    assert_eq!(
        roots
            .iter()
            .map(|r| r.name.split('@').next().unwrap())
            .collect::<BTreeSet<_>>()
            .len(),
        78
    );
    let mut census = frontier(&source, roots.clone());
    // Independently traverse the existing walker for each resolved attachment;
    // the new census's visited set is not its own expected population.
    let mut expected = BTreeSet::new();
    for root in &roots {
        if let Ok(id) = &root.target {
            for id in regions::frontier(&source, &[id.clone()]).bodies {
                let f = resolve::function(&source, &id).unwrap();
                expected.insert(Definition {
                    id,
                    site: f.site.clone(),
                });
            }
        }
    }
    validate(&census, &expected, &roots).unwrap();
    let dir = report::write(&census, &expected, &roots);
    report::independent_audit(&dir);

    let unresolved = roots.iter().find(|r| r.target.is_err()).unwrap();
    let row = census
        .rows
        .iter()
        .position(|r| r.root == unresolved.name)
        .unwrap();
    let removed = census.rows.remove(row);
    assert!(
        validate(&census, &expected, &roots)
            .unwrap_err()
            .contains("root vanished")
    );
    census.rows.insert(row, removed);
    let removed = census.roots.remove(&unresolved.name).unwrap();
    assert_eq!(
        validate(&census, &expected, &roots).unwrap_err(),
        "census omitted or invented a root attachment"
    );
    census.roots.insert(removed.name.clone(), removed);
    validate(&census, &expected, &roots).unwrap();
}

#[test]
fn conditional_ambiguous_and_missing_roots_are_located() {
    let source = source::fixture(&[(
        "probe",
        "probe.rs",
        "#[cfg(feature=\"one\")] fn root() { missing(); } #[cfg(not(feature=\"one\"))] fn root() { other(); } fn unique() {}",
    )]);
    let all = all(&source);
    assert_eq!(all.visited.len(), 3);
    assert_eq!(
        all.rows
            .iter()
            .filter(|r| r.diagnostic.reason.starts_with("ambiguous or conditional"))
            .count(),
        2
    );
    let f = &source.functions["fixture::probe::root"][0];
    let roots = vec![
        Root {
            name: "N:Conditional".into(),
            site: f.site.clone(),
            target: Ok(f.id.clone()),
            spelling: f.id.clone(),
        },
        Root {
            name: "N:Missing".into(),
            site: f.site.clone(),
            target: Ok("fixture::probe::missing".into()),
            spelling: "missing".into(),
        },
    ];
    let census = frontier(&source, roots.clone());
    validate(&census, &BTreeSet::new(), &roots).unwrap();
    assert_eq!(census.rows.len(), 2);
    assert!(
        census
            .rows
            .iter()
            .all(|r| r.diagnostic.site.file == "probe.rs")
    );
}

fn baseline(source: &Source) {
    let options = regions::frontier(
        source,
        &[
            format!("{}::merged_option_exprs_for_symbol", regions::SESSION),
            format!("{}::evaluate_runtime_option_expr", regions::SESSION),
        ],
    );
    // The symbol constructors remain absent. The removed raw C01 vector is
    // protected by local_option_contract_controls; C02 clone-return rejection
    // remains in compiler_oracle::exact. This historical frontier still rejects
    // unmodeled adapter internals independently of the concrete local marks.
    for (suffix, line, start, end) in [("expr_helpers.rs", 22, 45, 57)] {
        assert!(
            !options
                .rejected
                .iter()
                .any(|r| r.site.file.ends_with(suffix)
                    && r.site.start == (line, start)
                    && r.site.end == (line, end)),
            "constructor row remains at {suffix}:{line}:{start}-{end}"
        );
    }
    for (suffix, line, reason) in [
        ("options/core.rs", 233, "method syntax"),
        ("accounting_mode/rule.rs", 20, "closure capture"),
        ("accounting_mode/rule.rs", 22, "closure capture"),
        ("accounting_mode/rule.rs", 111, "closure capture"),
        ("expr_runtime_helpers.rs", 23, "implicit Index/Deref/bounds"),
    ] {
        assert!(
            options
                .rejected
                .iter()
                .any(|r| r.site.file.ends_with(suffix)
                    && r.site.start.0 == line
                    && r.reason.starts_with(reason))
        );
    }
    for (suffix, line) in [("accounting_mode/retained.rs", 84)] {
        assert!(options.rejected.iter().any(|r| r.kind == Kind::RET001
            && r.site.file.ends_with(suffix)
            && r.site.start.0 == line));
    }
    let mut kinds = BTreeMap::new();
    for row in &options.rejected {
        *kinds.entry(row.kind).or_insert(0) += 1;
    }
    // The direct unarmed guest entry reaches the evaluator in this historical
    // frontier. Guest-form abandonment adds exactly three rejected method sites:
    // taking/installing the paused environment and capturing it for cleanup
    // in suspension/core/abandon.rs. The cold profiled trampoline moves the
    // capture from line 85 to line 92 without changing the operation or kind,
    // and brief-629 shifts the three sites from lines 41, 42 and 92 to 43, 44
    // and 94. These remain unresolved boundaries; the loader earns no retention
    // mark.
    for line in [43, 44, 94] {
        assert!(options.rejected.iter().any(|row| row.kind == Kind::RET002
            && row.site.file.ends_with("suspension/core/abandon.rs")
            && row.site.start.0 == line));
    }
    // brief-595: abandonment settles a frame's provisional capture output
    // before abandoning it (`frame.output_boundary()` in suspension/core.rs and
    // four receiver calls in suspension/output.rs). All five are unresolved
    // method-receiver boundaries; no retention mark is earned.
    // brief-608: discovery pairs all 20 changed sites: 19 identical operations
    // moved into core/abandon.rs and one receiver shifted within core.rs. No
    // unpaired additions/removals; totals stay 932 (10/895/27). The cleanup
    // capture's new lexical scope reports an opaque Arc import instead of an
    // unexpanded item; it remains RET002. All boundaries remain unresolved.
    // Redispatch 4: the one changed site is the same take_for_abandon receiver
    // shifted from core.rs:473 to :477; all other 931 diagnostics are identical.
    // Redispatch 5: the same receiver moves from :477 to :482; the other
    // 931 rows and every unresolved-boundary assertion remain unchanged.
    // brief-609 S3: 916 S2 records are unchanged and 15 identical operations
    // move (one witness gains the cold frame factory). M0561's legacy RET003
    // becomes five accounted typed-expansion RET002 rows, including the cold
    // helper's captures/moves, FnOnce call and drops. The interim Try boundary
    // disappears when success exits the labeled block directly. No unpaired
    // removal or newly revealed legacy-arm row; freeze remains visible.
    // Redispatch 6: all 936 rows pair with the same operations, including
    // the caller-place witness and the outlined cycle formatter. Restoring
    // M0561's trailing `?` adds one unresolved Try/FromResidual boundary.
    // brief-613 G14: retire the old abandon.rs:41 stack.len() rejection with
    // the deleted depth-from-length assignment; pair witness.suspend with
    // witness.suspend_evaluation_return at the ruled fixed-point macro arm.
    // Fourteen new cold rejections cover twelve maintained-stack operations,
    // the return event and the context assertion. All remain rejected; the
    // frontier is 950 (10/913/27), with no new retention mark or coverage.
    // LP02 (brief-626): finite modes traverse one abandonment body; 950 accepted
    // rows pair, ten duplicate-loop rows merge, and two mode-dispatch sites
    // remain rejected: 952 (10/915/27), no mark or coverage. The C13 cold
    // profiled trampoline adds eleven witnessed occurrences (ten RET002 and one
    // RET003) through the same shared body, with no removed operation: 963
    // (10/925/28), still no mark or coverage.
    // brief-629: the native stack base moves to the thread's entry record and
    // the depth gains its threshold type. Keyed by kind, file, callee and
    // reason, three RET002 rows retire (two primitive operands of the old
    // depth-0 rebase in resolution.rs, abandon.rs's mem::replace) and eleven
    // RET002 rows appear (method receivers of EvaluationDepth in resolution.rs,
    // abandon.rs and evaluation_context.rs; NativeStackEntry::enter's marker
    // cast and TLS closure), all in files the brief changed. The added `use`
    // shifts AccumulatorChain::default from line 35 to 36. 971 (10/933/28),
    // still no mark or coverage. Continuation A3 keeps Session::evaluate's
    // body as at the base and gives the cold root entry its own copy, so the
    // record_profile_evaluate_entry receiver is rejected in both: one key's
    // count goes from 1 to 2, nothing added or removed. 972 (10/934/28).
    assert!(options.rejected.iter().any(|row| row.kind == Kind::RET002
        && row.site.file.ends_with("resolution.rs")
        && row.site.start == (36, 19)
        && row.reason == "path AccumulatorChain::default has 0 exact bindings"));
    assert_eq!(
        options.rejected.len(),
        972,
        "STOP: brief-545 exact legacy frontier inventory moved"
    );
    assert_eq!(
        kinds,
        BTreeMap::from([(Kind::RET001, 10), (Kind::RET002, 934), (Kind::RET003, 28)])
    );
    let wrapper = regions::frontier(
        source,
        &[format!(
            "{}::merged_option_exprs_for_system_symbol",
            regions::SESSION
        )],
    );
    // The registration guard and mode selection add public wrapper boundaries.
    // All 16 remain rejected here; this census does not grant local coverage.
    assert_eq!(wrapper.rejected.len(), 16, "STOP: PUBLIC_WRAPPER moved");
    assert!(
        wrapper.rejected.iter().any(|row| row.target.as_deref()
            == Some(&format!("{}::check_option_reentry", regions::SESSION)))
    );
    super::super::local::verify_promotions(source);
    println!(
        "ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2"
    );
}
