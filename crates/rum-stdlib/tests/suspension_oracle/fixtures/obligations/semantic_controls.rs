//! Frozen structural controls and the held pair, executed only by copied workers.
use crate::oracle::mutations::source_auth::{self, Armed, Disabled};
use crate::{HostEvalOutcome, HostSession};

#[derive(Debug)]
struct Plan {
    id: String,
    row: String,
    trace: bool,
    source: String,
    setup: String,
    pauses: Vec<(String, String, String)>,
    projections: Vec<String>,
}

fn unhex(s: &str) -> String {
    assert!(s.len() % 2 == 0 && s.is_ascii());
    String::from_utf8(
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect(),
    )
    .unwrap()
}

fn hex(s: &str) -> String {
    s.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

fn plans() -> Vec<Plan> {
    let text = include_str!("../../mutations/semantic-programs.tsv");
    let mut lines = text.lines();
    assert_eq!(
        lines.next(),
        Some("id\trow\ttrace\tsetup\tsource\tpauses\tprojections")
    );
    let mut seen = std::collections::BTreeSet::new();
    let plans: Vec<_> = lines
        .map(|line| {
            let c: Vec<_> = line.split('\t').collect();
            assert_eq!(c.len(), 7);
            assert!(seen.insert(c[0]));
            let pauses = unhex(c[5]);
            let cells: Vec<_> = pauses.split('\0').collect();
            assert_eq!(cells.len() % 3, 0);
            Plan {
                id: c[0].into(),
                row: c[1].into(),
                trace: match c[2] {
                    "0" => false,
                    "1" => true,
                    _ => panic!("trace flag"),
                },
                setup: unhex(c[3]),
                source: unhex(c[4]),
                pauses: cells
                    .chunks(3)
                    .map(|v| (v[0].into(), v[1].into(), v[2].into()))
                    .collect(),
                projections: unhex(c[6]).split('\0').map(String::from).collect(),
            }
        })
        .collect();
    assert_eq!(plans.len(), 105);
    for row in ["P0325", "P0326"] {
        assert_eq!(plans.iter().filter(|p| p.row == row).count(), 50);
    }
    plans
}

fn structural(out: &HostEvalOutcome) -> String {
    match out {
        HostEvalOutcome::Success(s) => format!(
            "Success {:?} {:?} {:?} {:?}",
            s.messages, s.prints, s.trace, s.termination
        ),
        HostEvalOutcome::Failure(f) => format!("Failure {f:?}"),
        HostEvalOutcome::InputRequired(p) => format!("InputRequired {p:?}"),
    }
}

fn subject_begin(queued: bool) {
    rum_runtime::cm740_probe::begin(queued, "CM740-SEMANTIC");
}
fn subject_end() {
    rum_runtime::cm740_probe::end();
}

fn run(plan: &Plan, queued: bool) -> (String, String) {
    let lease = rum_runtime::Session::logical_profile_test_acquire();
    let mut session = HostSession::builder().with_tracing(plan.trace).build();
    assert!(matches!(
        session.eval(plan.setup.as_str()),
        HostEvalOutcome::Success(_)
    ));
    if queued {
        for (_, _, answer) in &plan.pauses {
            session.queue_input_response(answer);
        }
    }
    rum_runtime::test_logical_profile_reset(&lease);
    subject_begin(queued);
    let mut output = session.eval(plan.source.as_str());
    let mut pauses = Vec::new();
    if !queued {
        for (kind, prompt, answer) in &plan.pauses {
            let HostEvalOutcome::InputRequired(request) = &output else {
                break;
            };
            assert_eq!(request.request.prompt, *prompt);
            assert_eq!(format!("{:?}", request.request.kind), *kind);
            pauses.push(format!("{output:?}"));
            output = session.resume_with_input(answer);
        }
        assert_eq!(
            pauses.len(),
            plan.pauses.len(),
            "complete actual producer pauses"
        );
    }
    subject_end();
    let scalar = match &output {
        HostEvalOutcome::Success(s) => format!("{:?}", s.value),
        _ => String::new(),
    };
    let trace = format!("{:?}", super::guest::channels(&output).2);
    println!(
        "CM740_SEMANTIC_CHANNELS\t{}\t{}\t{queued}\t{}\t{}\t{}",
        plan.row,
        hex(&plan.id),
        hex(&scalar),
        hex(&trace),
        pauses.len()
    );
    let rows = rum_runtime::Session::logical_profile_test_rows();
    rum_runtime::Session::logical_profile_test_assert_quiescent();
    let projections: Vec<_> = plan
        .projections
        .iter()
        .map(|p| session.eval(p.as_str()))
        .collect();
    let pending = session.pending_input_request().is_some();
    let snapshot = session.snapshot_to_bytes();
    let fresh = session.eval("1+1");
    let structural = format!(
        "{} rows={rows:?} projections={projections:?} pending={pending} snapshot={snapshot:?} fresh={fresh:?}",
        structural(&output)
    );
    let full = format!("output={output:?} pauses={pauses:?} structural={structural}");
    (full, structural)
}

#[test]
fn cm_semantic_copy_worker() {
    let Ok(selected) = std::env::var("RUM_CM740_SEMANTIC") else {
        return;
    };
    let arm = std::env::var("RUM_CM740_ARM").unwrap();
    println!(
        "CM740_SEMANTIC_RUN_ID row={selected} arm={arm} binary={} schema={} selector={}",
        crate::oracle::obligations::content_digest(
            &std::fs::read(std::env::current_exe().unwrap()).unwrap()
        ),
        crate::oracle::obligations::content_digest(include_bytes!(
            "../../mutations/source_auth/overlays.tsv"
        )),
        std::env::var("RUM_CM740_OMIT").unwrap_or_default()
    );
    assert!(
        matches!(
            selected.as_str(),
            "Some" | "G15A" | "G15B" | "C1" | "P0325" | "P0326"
        ) || (selected == "all" && arm == "real")
    );
    if arm != "real" {
        let baseline = std::env::var_os("RUM_CM740_BASELINE").unwrap();
        let receipt = std::env::var_os("RUM_CM740_RECEIPT").unwrap();
        match arm.as_str() {
            "disabled" => {
                source_auth::authenticate::<Disabled>(baseline.as_ref(), receipt.as_ref()).unwrap();
            }
            "armed" => {
                source_auth::authenticate::<Armed>(baseline.as_ref(), receipt.as_ref()).unwrap();
            }
            _ => panic!("invalid source arm"),
        }
    }
    let selected_program = std::env::var("RUM_CM740_PROGRAM").ok();
    let mut count = 0;
    for plan in plans().into_iter().filter(|p| {
        (p.row == selected || selected == "all")
            && selected_program.as_ref().is_none_or(|id| id == &p.id)
    }) {
        assert!(!plan.id.contains("guest_heads."));
        println!("CM740_SEMANTIC_CASE\t{}\t{}", plan.row, hex(&plan.id));
        for queued in [false, true] {
            let (full, structural) = run(&plan, queued);
            println!(
                "CM740_SEMANTIC\t{}\t{}\t{queued}\t{}\t{}",
                plan.row,
                hex(&plan.id),
                hex(&full),
                hex(&structural)
            );
        }
        count += 1;
    }
    assert!(count > 0);
    if selected == "all" {
        assert_eq!(count, 105);
    }
    println!(
        "CM4 SEMANTIC WORKER COMPLETE arm={arm} selected={selected} programs={count} encoding=0"
    );
}
