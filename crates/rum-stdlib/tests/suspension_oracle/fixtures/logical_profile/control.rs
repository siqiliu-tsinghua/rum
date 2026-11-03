//! Additive LP06 A1 receipts, independent of the frozen LP01 loss ledger.
use super::{descriptors, lp};
use descriptors::{Checks, Fixture, Pause};
use rum_runtime::{RuntimeInputKind, Session};
use std::collections::{BTreeMap, BTreeSet};

const OK: [u64; 10] = [1, 1, 0, 0, 0, 0, 0, 0, 0, 0];
const ERROR: [u64; 10] = [1, 0, 1, 0, 0, 0, 0, 0, 0, 0];
const RETURN: [u64; 10] = [1, 0, 1, 0, 0, 1, 0, 0, 0, 0];

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
