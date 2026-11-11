//! Whole native records for the completed-segment lifecycle in docs/dev/profiling.md.
use crate::oracle::{
    descriptors::Fixture,
    support::{drive, observe},
};
use rum_runtime::{RuntimeInputKind, Session};

#[test]
fn completed_segments_preserve_whole_native_rows() {
    // Profiling flags are process settings; isolate them without mutating the
    // environment shared by the other tests in this binary.
    if std::env::var_os("RUM_COMPLETED_SEGMENTS_CHILD").is_none() {
        let mut child = std::process::Command::new(std::env::current_exe().unwrap());
        child.args([
            "--exact",
            "completed_segments::completed_segments_preserve_whole_native_rows",
            "--nocapture",
        ]);
        for (name, _) in std::env::vars().filter(|(name, _)| name.starts_with("RUM_PROFILE_")) {
            child.env_remove(name);
        }
        child.env("RUM_COMPLETED_SEGMENTS_CHILD", "1");
        child.env("RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL", "0");
        for flag in [
            "LEXICAL_SUBSTITUTION",
            "BUILTIN_TIMING",
            "DEFINITION_RULE_TIMING",
            "RULE_RHS_TIMING",
            "EVALUATION_CACHE",
        ] {
            child.env(format!("RUM_PROFILE_{flag}"), "1");
        }
        assert!(child.status().unwrap().success());
        return;
    }

    let lease = Session::logical_profile_test_acquire();
    for (name, source, setup, prompts, answers) in [
        (
            "direct.prep",
            "c11f[Input[\"rule\"]]",
            "log={};c11f[83]:=89;",
            &["rule"][..],
            &["83"][..],
        ),
        (
            "direct.two",
            "c11f[Input[\"rule\"]]",
            "log={};c11f[83]:=Input[\"finish\"];",
            &["rule", "finish"][..],
            &["83", "89"][..],
        ),
        (
            "replace.two",
            "ReplaceAll[c11f[1],1:>Input[\"rule\"]]",
            "log={};c11f[83]:=Input[\"finish\"];",
            &["rule", "finish"][..],
            &["83", "89"][..],
        ),
        (
            "unheld.two",
            "ReplaceAllUnheld[c11f[1],1:>Input[\"rule\"]]",
            "log={};c11f[83]:=Input[\"finish\"];",
            &["rule", "finish"][..],
            &["83", "89"][..],
        ),
    ] {
        let mut fixture = Fixture::new(name, source);
        fixture.setup = setup.into();
        let requests = prompts
            .iter()
            .map(|prompt| (RuntimeInputKind::Input, (*prompt).into()))
            .collect::<Vec<_>>();
        let mut queued_rows = None;
        let mut queued_observation = None;
        for queued in [true, false] {
            let mut session = fixture.session();
            session.set_eval_timeout(Some(std::time::Duration::from_secs(2)));
            rum_runtime::test_logical_profile_reset(&lease);
            let run =
                drive::execute_planned(&mut session, source, answers, queued, Some(&requests))
                    .unwrap();
            let rows = Session::logical_profile_test_rows();
            assert!(run.output.is_ok(), "{name} queued={queued}");
            assert_eq!(run.requests.len(), if queued { 0 } else { prompts.len() });
            for family in ["Runtime {", "Builtin {", "Definition {"] {
                assert!(
                    rows.iter()
                        .any(|(key, row)| key.starts_with(family) && row[0] > 0),
                    "{name} queued={queued}: missing nonempty {family}"
                );
            }
            for phase in [
                "prepare_arguments.total",
                "definition.rhs.total",
                "evaluate.total",
            ] {
                assert!(
                    rows.iter()
                        .any(|(key, row)| key.contains(phase) && row[0] > 0),
                    "{name} queued={queued}: missing nonempty {phase}"
                );
            }
            let observation = observe::Observation::from_output(&run.output);
            if queued {
                assert!(rows.len() > 3 && rows.iter().map(|(_, row)| row[0]).sum::<u64>() > 3);
                queued_rows = Some(rows);
                queued_observation = Some(observation);
            } else {
                assert_eq!(
                    rows,
                    *queued_rows.as_ref().unwrap(),
                    "{name}: complete native ten-field rows"
                );
                observe::compare(&observation, queued_observation.as_ref().unwrap()).unwrap();
            }
            Session::logical_profile_test_assert_quiescent();
            assert_eq!(Session::logical_profile_test_leases(), 0);
        }
        eprintln!(
            "COMPLETED SEGMENT ROWS PASS case={name} whole_native_rows=SAME nonempty_families=Runtime,Builtin,Definition"
        );
    }
}
