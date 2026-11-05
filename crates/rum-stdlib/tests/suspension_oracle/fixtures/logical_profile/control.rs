//! Additive LP06 A1 receipts, independent of the frozen LP01 loss ledger.
use super::{descriptors, lp};
use descriptors::{Checks, Fixture, Pause};
use rum_runtime::{RuntimeInputKind, Session};
use std::collections::{BTreeMap, BTreeSet};

const OK: [u64; 10] = [1, 1, 0, 0, 0, 0, 0, 0, 0, 0];
const ERROR: [u64; 10] = [1, 0, 1, 0, 0, 0, 0, 0, 0, 0];
const RETURN: [u64; 10] = [1, 0, 1, 0, 0, 1, 0, 0, 0, 0];
const OPEN: [u64; 10] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
const THROW: [u64; 10] = [1, 0, 1, 0, 0, 0, 1, 0, 0, 0];

/// LP07 observes the frozen subject again before any guest state projection.
pub(super) fn module_counts(
    fixture: &Fixture,
    queued: bool,
    mode: &str,
) -> BTreeMap<String, [u64; 10]> {
    const PHASES: [&str; 4] = [
        "module.eval.total",
        "module.resume_initializers",
        "module.initializer.evaluate",
        "module.body.evaluate",
    ];
    let lease = Session::logical_profile_test_acquire();
    let mut session = fixture.session();
    session.set_eval_timeout(Some(std::time::Duration::from_secs(2)));
    session.enable_tracing(fixture.profile_trace);
    rum_runtime::test_logical_profile_reset(&lease);
    let answers = fixture
        .pauses
        .iter()
        .map(|p| p.answer.as_str())
        .collect::<Vec<_>>();
    let requests = fixture
        .pauses
        .iter()
        .map(|p| (p.kind, p.prompt.clone()))
        .collect::<Vec<_>>();
    let run = super::super::super::support::drive::execute_planned(
        &mut session,
        &fixture.source,
        &answers,
        queued,
        Some(&requests),
    )
    .unwrap_or_else(|e| panic!("{} LP07 queued={queued}: {e:?}", fixture.id));
    assert_eq!(run.requests.len(), if queued { 0 } else { requests.len() });
    let rows = Session::logical_profile_test_rows()
        .into_iter()
        .filter(|(key, _)| {
            PHASES
                .iter()
                .any(|p| key.contains(&format!("phase: {p:?}")))
        })
        .collect::<BTreeMap<_, _>>();
    let legacy = rum_runtime::test_logical_profile_snapshot();
    assert_eq!(
        rows.keys().cloned().collect::<BTreeSet<_>>(),
        lp::control_legacy_keys(&legacy, &PHASES)
    );
    if mode == "disabled" {
        assert!(rows.is_empty());
    } else {
        for phase in [PHASES[0], PHASES[1], PHASES[3]] {
            assert!(
                rows.keys()
                    .any(|key| key.contains(&format!("phase: {phase:?}"))),
                "{} missing {phase}",
                fixture.id
            );
        }
    }
    for (key, row) in &rows {
        assert_eq!(row[0], row[1] + row[2], "{} {key}", fixture.id);
        assert_eq!(
            (row[3], row[4], row[7], row[8], row[9]),
            (0, 0, 0, 0, 0),
            "{} {key}",
            fixture.id
        );
    }
    Session::logical_profile_test_assert_quiescent();
    assert_eq!(Session::logical_profile_test_leases(), 0);
    rows
}

struct A2Cell {
    fixture: Fixture,
    builtin: Option<(&'static str, u16)>,
    expected: Vec<[u64; 10]>,
}

fn a2_cell(
    id: &str,
    source: &str,
    prompts: &[&str],
    value: Option<&str>,
    builtin: Option<(&'static str, u16)>,
    _pauses: &[&[[u64; 10]]],
    expected: &[[u64; 10]],
) -> A2Cell {
    let mut fixture = cell(
        id,
        "",
        source,
        prompts,
        value.unwrap_or(""),
        &lp::CATCH_PHASES,
        expected,
    )
    .fixture;
    fixture.expected_value = value.map(str::to_string);
    if value.is_none() {
        fixture.expected_error = Some(rum_runtime::SessionRequestError::Other {
            message: if id == "lp06.catch.bypass_miss" {
                "uncaught throw: 1 with tag t"
            } else {
                "uncaught Abort[]"
            }
            .into(),
        });
    }
    A2Cell {
        fixture,
        builtin,
        expected: expected.to_vec(),
    }
}

fn a2_counts(cell: &A2Cell, queued: bool, mode: &str) -> BTreeMap<String, [u64; 10]> {
    let fixture = &cell.fixture;
    let lease = Session::logical_profile_test_acquire();
    let mut session = fixture.session();
    session.set_eval_timeout(Some(std::time::Duration::from_secs(2)));
    session.enable_tracing(fixture.profile_trace);
    rum_runtime::test_logical_profile_reset(&lease);
    let answers = fixture
        .pauses
        .iter()
        .map(|p| p.answer.as_str())
        .collect::<Vec<_>>();
    let requests = fixture
        .pauses
        .iter()
        .map(|p| (p.kind, p.prompt.clone()))
        .collect::<Vec<_>>();
    let run = super::super::super::support::drive::execute_planned(
        &mut session,
        &fixture.source,
        &answers,
        queued,
        Some(&requests),
    )
    .unwrap_or_else(|e| panic!("{} A2 queued={queued}: {e:?}", fixture.id));
    assert_eq!(
        run.requests.len(),
        if queued { 0 } else { fixture.pauses.len() },
        "{} exact requests",
        fixture.id
    );
    let enabled = if cell.builtin.is_some() {
        matches!(mode, "timing" | "all")
    } else {
        mode != "disabled"
    };
    let keys: Vec<_> = match cell.builtin {
        Some((name, arity)) => vec![format!(
            "Builtin {{ symbol: {:?}, source: None, arity: {arity} }}",
            rum_core::Symbol::new("System`", name)
        )],
        None => lp::CATCH_PHASES
            .iter()
            .map(|phase| format!("Runtime {{ phase: {phase:?}, source: None }}"))
            .collect(),
    };
    let all_rows = Session::logical_profile_test_rows();
    let rows = all_rows
        .iter()
        .filter(|(key, _)| match cell.builtin {
            Some((name, _)) => key.starts_with(&format!(
                "Builtin {{ symbol: {:?},",
                rum_core::Symbol::new("System`", name)
            )),
            None => lp::CATCH_PHASES
                .iter()
                .any(|phase| key.contains(&format!("phase: {phase:?}"))),
        })
        .cloned()
        .collect::<BTreeMap<_, _>>();
    let expected = keys
        .into_iter()
        .zip(cell.expected.iter().copied())
        .filter(|(_, row)| enabled && *row != [0; 10])
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        rows, expected,
        "{} {mode}: FULL keys and calls/ok/err/some/none/return/throw/cancelled/abandoned/open",
        fixture.id
    );
    if !matches!(mode, "timing" | "all") {
        assert!(
            all_rows
                .iter()
                .all(|(key, _)| !key.starts_with("Builtin {")),
            "{}: builtin switch gating",
            fixture.id
        );
    }
    if mode == "disabled" {
        assert!(all_rows.iter().all(|(key, _)| {
            !lp::CATCH_PHASES
                .iter()
                .any(|p| key.contains(&format!("phase: {p:?}")))
        }));
    }
    if cell.builtin.is_none() {
        assert_eq!(
            rows.keys().cloned().collect::<BTreeSet<_>>(),
            lp::control_legacy_keys(
                &rum_runtime::test_logical_profile_snapshot(),
                &lp::CATCH_PHASES
            ),
            "{}: legacy source dimensions",
            fixture.id
        );
    }
    Session::logical_profile_test_assert_quiescent();
    assert_eq!(Session::logical_profile_test_leases(), 0);
    rows
}

pub(super) fn run_a2(mode: &str) {
    let cases = a2_cases();
    assert_eq!(
        cases
            .iter()
            .map(|c| c.fixture.id.as_str())
            .collect::<Vec<_>>(),
        lp::BUILTIN_CATCH_CELLS
            .into_iter()
            .filter(|id| *id != "lp06.builtin.no_handler")
            .collect::<Vec<_>>()
    );
    for mut cell in cases {
        for trace in [false, true] {
            cell.fixture.profile_trace = trace;
            descriptors::accept(&cell.fixture, Checks::ALL)
                .unwrap_or_else(|e| panic!("{} {mode} trace={trace}: {e:?}", cell.fixture.id));
            let queued = a2_counts(&cell, true, mode);
            let resumed = a2_counts(&cell, false, mode);
            assert_eq!(
                queued, resumed,
                "{} {mode} trace={trace}: complete selected full-key map",
                cell.fixture.id
            );
            println!(
                "LP06 A2 {} {mode} trace={trace}: SAME full_keys={} open=0 abandoned=0",
                cell.fixture.id,
                queued.len()
            );
        }
    }
    println!(
        "LP06 A2 {mode}: SAME guest_cells=18 trace_receipts=36; no_handler=native; additive_identities=19"
    );
}

struct Cell {
    fixture: Fixture,
    phases: &'static [&'static str],
    expected: Vec<[u64; 10]>,
}

fn cell(
    id: &str,
    setup: &str,
    source: &str,
    prompts: &[&str],
    value: &str,
    phases: &'static [&'static str],
    expected: &[[u64; 10]],
) -> Cell {
    let mut fixture = Fixture::new(id, source);
    fixture.setup = setup.into();
    fixture.logical_profile = true;
    fixture.expected_value = Some(value.into());
    fixture.pauses = prompts
        .iter()
        .zip(["a", "b"])
        .map(|(prompt, answer)| Pause {
            prompt: (*prompt).into(),
            kind: RuntimeInputKind::InputString,
            answer: answer.into(),
        })
        .collect();
    if id == "lp06.while.body.continue" {
        fixture.projections.push("n".into());
        fixture.expected_projections.push("2".into());
    }
    Cell {
        fixture,
        phases,
        expected: expected.to_vec(),
    }
}

fn counts(cell: &Cell, queued: bool, enabled: bool) -> BTreeMap<String, [u64; 10]> {
    let fixture = &cell.fixture;
    let lease = Session::logical_profile_test_acquire();
    let mut session = fixture.session();
    session.set_eval_timeout(Some(std::time::Duration::from_secs(2)));
    session.enable_tracing(fixture.profile_trace);
    rum_runtime::test_logical_profile_reset(&lease);
    let answers = fixture
        .pauses
        .iter()
        .map(|p| p.answer.as_str())
        .collect::<Vec<_>>();
    let requests = fixture
        .pauses
        .iter()
        .map(|p| (p.kind, p.prompt.clone()))
        .collect::<Vec<_>>();
    let run = super::super::super::support::drive::execute_planned(
        &mut session,
        &fixture.source,
        &answers,
        queued,
        Some(&requests),
    )
    .unwrap_or_else(|e| panic!("{} adopted queued={queued}: {e:?}", fixture.id));
    assert_eq!(
        run.requests.len(),
        if queued { 0 } else { fixture.pauses.len() },
        "{} exact requests",
        fixture.id
    );
    let legacy = rum_runtime::test_logical_profile_snapshot();
    let all_rows = Session::logical_profile_test_rows();
    let rows = all_rows
        .iter()
        .filter(|(key, _)| {
            cell.phases
                .iter()
                .any(|p| key.contains(&format!("phase: \"{p}\"")))
        })
        .cloned()
        .collect::<BTreeMap<_, _>>();
    if !enabled {
        assert!(
            all_rows.iter().all(|(key, _)| !lp::CONTROL_PHASES
                .iter()
                .any(|p| key.contains(&format!("phase: \"{p}\"")))),
            "{} disabled control rows: {all_rows:?}",
            fixture.id
        );
    }
    assert_eq!(
        rows.keys().cloned().collect::<BTreeSet<_>>(),
        lp::control_legacy_keys(&legacy, cell.phases),
        "{}: full selected phase/source keys differ from legacy",
        fixture.id
    );
    for (phase, expected) in cell.phases.iter().zip(&cell.expected) {
        let selected = rows
            .iter()
            .filter(|(key, _)| key.contains(&format!("phase: \"{phase}\"")))
            .collect::<Vec<_>>();
        if !enabled || *expected == [0; 10] {
            assert!(
                selected.is_empty(),
                "{} unexpected {phase}: {selected:?}",
                fixture.id
            );
        } else {
            assert_eq!(
                selected.len(),
                1,
                "{} required {phase}: {selected:?}",
                fixture.id
            );
            assert_eq!(
                selected[0].1, expected,
                "{} {phase}: calls/ok/err/some/none/return/throw/cancelled/abandoned/open",
                fixture.id
            );
        }
    }
    for (key, row) in &rows {
        assert_eq!(row[0], row[1] + row[2], "{} {key}", fixture.id);
        assert_eq!(
            (row[3], row[4], row[7], row[8], row[9]),
            (0, 0, 0, 0, 0),
            "{} {key}",
            fixture.id
        );
    }
    // These synchronous observations retain their incoming outcome.
    let return_value = matches!(
        fixture.id.as_str(),
        "lp06.function.return.twice" | "lp06.status.value_return"
    );
    let projection_return = fixture.id == "lp06.status.projection_return";
    if enabled && (return_value || projection_return || fixture.id == "lp06.function.ok") {
        let finish = legacy
            .logical
            .iter()
            .filter(|(key, _)| key.starts_with("runtime|function.apply.finish|"))
            .collect::<Vec<_>>();
        if projection_return {
            assert!(finish.is_empty(), "{} unexpected value finish", fixture.id);
        } else {
            assert_eq!(finish.len(), 1, "{} legacy finish key", fixture.id);
            assert_eq!(
                finish[0].1.as_slice(),
                if return_value { &RETURN[..7] } else { &OK[..7] },
                "{} legacy finish",
                fixture.id
            );
        }
        if fixture.id == "lp06.function.return.twice" {
            let returns = legacy
                .logical
                .iter()
                .filter(|(key, _)| key.starts_with("runtime|return.eval|"))
                .collect::<Vec<_>>();
            assert_eq!(returns.len(), 1);
            assert_eq!(returns[0].1.as_slice(), &RETURN[..7]);
        }
    }
    Session::logical_profile_test_assert_quiescent();
    assert_eq!(Session::logical_profile_test_leases(), 0);
    rows
}

pub(super) fn run(mode: &str) {
    let cases = cases();
    assert_eq!(
        cases
            .iter()
            .map(|c| c.fixture.id.as_str())
            .collect::<Vec<_>>(),
        lp::CONTROL_CELLS
    );
    for mut cell in cases {
        for trace in [false, true] {
            cell.fixture.profile_trace = trace;
            descriptors::accept(&cell.fixture, Checks::ALL)
                .unwrap_or_else(|e| panic!("{} {mode} trace={trace}: {e:?}", cell.fixture.id));
            let queued = counts(&cell, true, mode != "disabled");
            let resumed = counts(&cell, false, mode != "disabled");
            assert_eq!(
                queued, resumed,
                "{} {mode} trace={trace}: complete selected phase/source map",
                cell.fixture.id
            );
            println!(
                "LP06 A1 {} {mode} trace={trace}: SAME keys={} open=0 abandoned=0",
                cell.fixture.id,
                queued.len()
            );
        }
    }
    println!("LP06 A1 {mode}: SAME cells=13 trace_receipts=26; separate from 875 legacy receipts");
}

fn cases() -> Vec<Cell> {
    vec![
        cell(
            "lp06.while.test.once",
            r#""#,
            r#"While[(InputString["test"];False),Null]"#,
            &["test"],
            "Null",
            &lp::CONTROL_PHASES[..4],
            &[OK, OK, [0; 10], [0; 10]],
        ),
        cell(
            "lp06.while.test.twice",
            r#""#,
            r#"While[(InputString["test1"];InputString["test2"];False),Null]"#,
            &["test1", "test2"],
            "Null",
            &lp::CONTROL_PHASES[..4],
            &[OK, OK, [0; 10], [0; 10]],
        ),
        cell(
            "lp06.while.body.break",
            r#""#,
            r#"While[True,(InputString["body"];Break[])]"#,
            &["body"],
            "Null",
            &lp::CONTROL_PHASES[..4],
            &[OK, OK, ERROR, ERROR],
        ),
        cell(
            "lp06.while.body.continue",
            r#"n=0"#,
            r#"While[n<2,(n++;InputString["body"];Continue[])]"#,
            &["body", "body"],
            "Null",
            &lp::CONTROL_PHASES[..4],
            &[
                [3, 3, 0, 0, 0, 0, 0, 0, 0, 0],
                [3, 3, 0, 0, 0, 0, 0, 0, 0, 0],
                [2, 0, 2, 0, 0, 0, 0, 0, 0, 0],
                [2, 0, 2, 0, 0, 0, 0, 0, 0, 0],
            ],
        ),
        cell(
            "lp06.compound.items",
            r#""#,
            r#"InputString["first"];InputString["second"]"#,
            &["first", "second"],
            "\"b\"",
            &lp::CONTROL_PHASES[4..6],
            &[
                [2, 2, 0, 0, 0, 0, 0, 0, 0, 0],
                [2, 2, 0, 0, 0, 0, 0, 0, 0, 0],
            ],
        ),
        cell(
            "lp06.compound.next_throw",
            r#""#,
            r#"Catch[(InputString["first"];Throw[7])]"#,
            &["first"],
            "7",
            &lp::CONTROL_PHASES[4..6],
            &[
                [2, 1, 1, 0, 0, 0, 1, 0, 0, 0],
                [2, 1, 1, 0, 0, 0, 1, 0, 0, 0],
            ],
        ),
        cell(
            "lp06.function.return.twice",
            r#""#,
            r#"Function[{},(InputString["one"];InputString["two"];Return[7])][]"#,
            &["one", "two"],
            "7",
            &lp::CONTROL_PHASES[6..],
            &[RETURN],
        ),
        cell(
            "lp06.function.ok",
            r#""#,
            r#"Function[{},(InputString["value"];19)][]"#,
            &["value"],
            "19",
            &lp::CONTROL_PHASES[6..],
            &[OK],
        ),
        cell(
            "lp06.status.projection_value",
            r#""#,
            r#"Failure["t",<|"MessageTemplate":>(InputString["value"];19),"MessageParameters":>(InputString["projection"];{})|>]["Message"]"#,
            &["projection", "value"],
            "19",
            &lp::CONTROL_PHASES[6..],
            &[OK],
        ),
        cell(
            "lp06.status.projection_return",
            r#"f[]:=(Failure["t",<|"MessageTemplate"->"x","MessageParameters":>(InputString["w"];Return[3])|>]["Message"];7)"#,
            r#"f[]"#,
            &["w"],
            "3",
            &lp::CONTROL_PHASES[6..],
            &[[0; 10]],
        ),
        cell(
            "lp06.status.value_return",
            r#""#,
            r#"f[]:=(Failure["t",<|"MessageTemplate":>(InputString["value"];Return[3]),"MessageParameters":>(InputString["projection"];{})|>]["Message"];7);f[]"#,
            &["projection", "value"],
            "7",
            &lp::CONTROL_PHASES[6..],
            &[RETURN],
        ),
        cell(
            "lp06.status.raw_failure",
            r#""#,
            r#"Failure["t",<|"x":>(InputString["value"];17)|>]["x"]"#,
            &["value"],
            "17",
            &lp::CONTROL_PHASES[6..],
            &[OK],
        ),
        cell(
            "lp06.status.raw_success",
            r#""#,
            r#"Success["t",<|"x":>(InputString["value"];17)|>]["x"]"#,
            &["value"],
            "17",
            &lp::CONTROL_PHASES[6..],
            &[OK],
        ),
    ]
}

fn a2_cases() -> Vec<A2Cell> {
    vec![
        a2_cell(
            "lp06.builtin.ok",
            r###"Length[{1,2}]"###,
            &[],
            Some("2"),
            Some(("Length", 1)),
            &[],
            &[OK],
        ),
        a2_cell(
            "lp06.builtin.cleanup.twice",
            r###"WithCleanup[(InputString["primary"];7),(InputString["cleanup"];Null)]"###,
            &["primary", "cleanup"],
            Some("7"),
            Some(("WithCleanup", 2)),
            &[&[OPEN], &[OPEN]],
            &[OK],
        ),
        a2_cell(
            "lp06.builtin.cleanup.throw",
            r###"Catch[WithCleanup[Throw[7,t],(InputString["cleanup"];Null)],t]"###,
            &["cleanup"],
            Some("7"),
            Some(("WithCleanup", 2)),
            &[&[OPEN]],
            &[THROW],
        ),
        a2_cell(
            "lp06.builtin.return",
            r###"Function[{},Return[7]][]"###,
            &[],
            Some("7"),
            Some(("Return", 1)),
            &[],
            &[RETURN],
        ),
        a2_cell(
            "lp06.catch.body",
            r###"Catch[(InputString["p"];Throw[1])]"###,
            &["p"],
            Some("1"),
            None,
            &[&[OPEN, OPEN, [0; 10]]],
            &[OK, THROW, THROW],
        ),
        a2_cell(
            "lp06.catch.handler.twice",
            r###"Catch[(InputString["body"];Throw[7,t]),t,Function[{v,k},(InputString["handler"];v+1)]]"###,
            &["body", "handler"],
            Some("8"),
            None,
            &[&[OPEN, OPEN, [0; 10]], &[OPEN, THROW, OPEN]],
            &[OK, THROW, THROW],
        ),
        a2_cell(
            "lp06.catch.handler_error",
            r###"Catch[(InputString["body"];Throw[7,t]),t,Function[{v,k},(InputString["handler"];Abort[])]]"###,
            &["body", "handler"],
            Some("$Aborted"),
            None,
            &[&[OPEN, OPEN, [0; 10]], &[OPEN, THROW, OPEN]],
            &[ERROR, THROW, THROW],
        ),
        a2_cell(
            "lp06.catch.form",
            r###"Catch[Throw[1,t],(InputString["f"];t)]"###,
            &["f"],
            Some("1"),
            None,
            &[&[OPEN, THROW, OPEN]],
            &[OK, THROW, THROW],
        ),
        a2_cell(
            "lp06.catch.match",
            r###"Catch[Throw[1,t[2]],t[x_]/;(InputString["g"];x>1)]"###,
            &["g"],
            Some("1"),
            None,
            &[&[OPEN, THROW, OPEN]],
            &[OK, THROW, THROW],
        ),
        a2_cell(
            "lp06.catch.match_miss",
            r###"Catch[Catch[Throw[1,t[2]],t[x_]/;(InputString["g"];x>5)],_]"###,
            &["g"],
            Some("1"),
            None,
            &[&[
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
                [1, 0, 1, 0, 0, 0, 1, 0, 0, 1],
                OPEN,
            ]],
            &[
                [2, 1, 1, 0, 0, 0, 1, 0, 0, 0],
                [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
                [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
            ],
        ),
        a2_cell(
            "lp06.catch.bypass_form",
            r###"Catch[Catch[WithCleanup[Throw[1,t],Throw[2,t]],(InputString["f"];t)],t]"###,
            &["f"],
            Some("1"),
            None,
            &[&[
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
                [1, 0, 1, 0, 0, 0, 1, 0, 0, 1],
                OPEN,
            ]],
            &[
                [2, 1, 1, 0, 0, 0, 1, 0, 0, 0],
                [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
                [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
            ],
        ),
        a2_cell(
            "lp06.catch.bypass_match",
            r###"Catch[Catch[WithCleanup[Throw[1,t[1]],Throw[2,t[1]]],t[x_]/;(InputString["g"];True)],_]"###,
            &["g"],
            Some("1"),
            None,
            &[&[
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
                [1, 0, 1, 0, 0, 0, 1, 0, 0, 1],
                OPEN,
            ]],
            &[
                [2, 1, 1, 0, 0, 0, 1, 0, 0, 0],
                [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
                [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
            ],
        ),
        a2_cell(
            "lp06.catch.ok",
            r###"Catch[(InputString["body"];19)]"###,
            &["body"],
            Some("19"),
            None,
            &[&[OPEN, OPEN, [0; 10]]],
            &[OK, OK, OK],
        ),
        a2_cell(
            "lp06.catch.invalid_arity",
            r###"Catch[]"###,
            &[],
            Some("Catch[]"),
            None,
            &[],
            &[OK, [0; 10], [0; 10]],
        ),
        a2_cell(
            "lp06.catch.return",
            r###"Function[{},Catch[(InputString["body"];Return[7])]][]"###,
            &["body"],
            Some("7"),
            None,
            &[&[OPEN, OPEN, [0; 10]]],
            &[RETURN, RETURN, RETURN],
        ),
        a2_cell(
            "lp06.catch.bypass_no_form",
            r###"Catch[Catch[WithCleanup[Throw[1,t],(InputString["cleanup"];Throw[2,t])]],t]"###,
            &["cleanup"],
            Some("1"),
            None,
            &[&[
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
                [0; 10],
            ]],
            &[
                [2, 1, 1, 0, 0, 0, 1, 0, 0, 0],
                [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
                [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
            ],
        ),
        a2_cell(
            "lp06.catch.bypass_miss",
            r###"Catch[Catch[WithCleanup[Throw[1,t],Throw[2,t]],(InputString["f"];u)],t]"###,
            &["f"],
            None,
            None,
            &[&[
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
                [1, 0, 1, 0, 0, 0, 1, 0, 0, 1],
                OPEN,
            ]],
            &[
                [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
                [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
                [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
            ],
        ),
        a2_cell(
            "lp06.catch.match_error",
            r###"Catch[Throw[1,t[2]],t[x_]/;(InputString["g"];Abort[])]"###,
            &["g"],
            Some("$Aborted"),
            None,
            &[&[OPEN, THROW, OPEN]],
            &[ERROR, THROW, THROW],
        ),
    ]
}
