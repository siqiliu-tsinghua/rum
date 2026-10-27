//! Profiled abandonment re-queues the remainder exactly as ordinary abandonment does.
use rum_core::{Expr, SharedArgs};
use rum_host::{HostEvalOutcome, HostSession};
use rum_runtime::kernel_api::{BuiltinSpec, EvalError, MemoryGrowthContract, Session};
use std::sync::Mutex;

static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn install(session: &mut Session) {
    session.register_builtin(BuiltinSpec::special(
        "LpSpan",
        "Native logical-profiler test span",
        span,
        MemoryGrowthContract::NonAmplifying,
    ));
    session.register_builtin(BuiltinSpec::special(
        "LpSeen",
        "Records its evaluated argument outside guest state",
        seen,
        MemoryGrowthContract::NonAmplifying,
    ));
}
fn span(session: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    session.logical_profile_test_span("native", args[0].clone())
}
fn seen(session: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    let value = session.evaluate(args[0].clone())?;
    SEEN.lock().unwrap().push(value.to_string());
    Ok(Expr::string("seen"))
}

// The inner cleanup runs while frames remain, so the remainder is re-queued
// with the stack and guest environment of that moment. The outer cleanup then
// runs on the re-queued remainder. The downvalues put evaluation-stack entries
// under the cells, so a remainder re-queued without its stack is visible.
const SETUP: &str = "k[] := WithCleanup[InputString[\"p\"], LpSeen[{\"inner\", Stack[]}]]; \
    m[] := WithCleanup[g[k[]], LpSeen[{\"outer\", Stack[]}]]";
const PROGRAM: &str = "LpSpan[m[]]";

fn run(trigger: &str) -> (String, Vec<[u64; 10]>) {
    Session::logical_profile_test_reset();
    SEEN.lock().unwrap().clear();
    let mut s = HostSession::builder()
        .with_builtin_installer(install)
        .build();
    let setup = s.eval(SETUP);
    assert!(
        !matches!(setup, HostEvalOutcome::InputRequired(_)),
        "{setup:?}"
    );
    let outcome = match trigger {
        "valueq" => s.eval(format!("ValueQ[{PROGRAM}]").as_str()),
        "cancel" => {
            let cancel = s.cancel_handle();
            let paused = s.eval(PROGRAM);
            assert!(
                matches!(paused, HostEvalOutcome::InputRequired(_)),
                "{paused:?}"
            );
            cancel.cancel();
            s.resume_with_input("ignored")
        }
        _ => panic!("unknown trigger {trigger}"),
    };
    let value = match &outcome {
        HostEvalOutcome::Success(success) => success.value.text.clone(),
        other => format!("{other:?}"),
    };
    let rows = Session::logical_profile_test_rows()
        .into_iter()
        .filter(|(key, _)| key.contains("native"))
        .map(|(_, counts)| counts)
        .collect();
    let seen = SEEN.lock().unwrap().join(" | ");
    (format!("value={value} seen=[{seen}]"), rows)
}

fn child(profiled: bool) -> Vec<String> {
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "profiled_abandonment_requeues_like_ordinary",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(
            "LP02_ABANDON_CHILD",
            if profiled { "profiled" } else { "ordinary" },
        )
        .env_remove("RUM_PROFILE_LEXICAL_SUBSTITUTION");
    if profiled {
        command.env("RUM_PROFILE_LEXICAL_SUBSTITUTION", "1");
    }
    let out = command.output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        out.status.success(),
        "profiled={profiled}\n{stdout}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    stdout
        .lines()
        .filter_map(|line| {
            line.split_once("LP02_ABANDON ")
                .map(|(_, rest)| rest.to_owned())
        })
        .collect()
}

#[test]
fn profiled_abandonment_requeues_like_ordinary() {
    let Some(arm) = std::env::var_os("LP02_ABANDON_CHILD") else {
        let ordinary = child(false);
        let profiled = child(true);
        for (o, p) in ordinary.iter().zip(&profiled) {
            println!("LP02_ABANDON_ORDINARY {o}");
            println!("LP02_ABANDON_PROFILED {p}");
        }
        assert_eq!(ordinary.len(), 2);
        assert_eq!(profiled, ordinary);
        return;
    };
    let profiled = arm == "profiled";
    for trigger in ["valueq", "cancel"] {
        let (observed, rows) = run(trigger);
        // Arm checks: the profiled child opened one record before the pause,
        // so its suspension was the profiled representation; the ordinary
        // child opened none.
        if profiled {
            assert_eq!(rows.len(), 1, "{trigger}: {rows:?}");
            assert_eq!(rows[0][0], 1, "{trigger}: {rows:?}");
        } else {
            assert!(rows.is_empty(), "{trigger}: {rows:?}");
        }
        // Control: the ordinary outer cleanup sees the saved stack under it,
        // so a remainder re-queued without that stack cannot match.
        if !profiled {
            assert!(
                observed
                    .split_once("List[\"outer\", List[")
                    .is_some_and(|(_, outer)| outer.contains("Hold[m[]]")),
                "{trigger}: the control did not reach the re-queued remainder: {observed}"
            );
        }
        println!("LP02_ABANDON {trigger} {observed}");
    }
}
