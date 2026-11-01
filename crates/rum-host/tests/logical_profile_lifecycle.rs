//! Explicit native LP02 spans; production phases retain their LP01 counters.
use rum_core::{Expr, SharedArgs, Symbol};
use rum_host::{HostEvalOutcome, HostSession};
use rum_runtime::kernel_api::{BuiltinSpec, EvalError, MemoryGrowthContract, Session};

fn install(session: &mut Session) {
    session.register_builtin(BuiltinSpec::special(
        "LpSpan",
        "Native logical-profiler test span",
        span,
        MemoryGrowthContract::NonAmplifying,
    ));
    session.register_builtin(BuiltinSpec::special(
        "LpScope",
        "Native logical-profiler source scope probe",
        scope,
        MemoryGrowthContract::NonAmplifying,
    ));
}
fn span(session: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    session.logical_profile_test_span("native", args[0].clone())
}
fn scope(session: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    session.logical_profile_test_source_scope(Symbol::from_name("lp_scope"), args[0].clone())
}
fn session() -> HostSession {
    HostSession::builder()
        .with_builtin_installer(install)
        .build()
}
fn native_rows() -> Vec<(String, [u64; 10])> {
    Session::logical_profile_test_rows()
        .into_iter()
        .filter(|(key, _)| key.contains("native"))
        .collect()
}
fn counts() -> [u64; 10] {
    let rows = native_rows();
    assert!(rows.len() <= 1, "{rows:?}");
    rows.first().map(|(_, c)| *c).unwrap_or_default()
}
fn paused(outcome: HostEvalOutcome) {
    assert!(
        matches!(outcome, HostEvalOutcome::InputRequired(_)),
        "{outcome:?}"
    );
}
fn done(outcome: HostEvalOutcome) {
    assert!(
        !matches!(outcome, HostEvalOutcome::InputRequired(_)),
        "{outcome:?}"
    );
}
fn closed(expected: [u64; 10]) {
    assert_eq!(counts(), expected);
    assert_eq!(Session::logical_profile_test_leases(), 0);
    Session::logical_profile_test_assert_quiescent();
    println!("LP02_TEARDOWN pending_children=0 terminal_parents=0 open=0 leases=0 faults=0");
}

#[test]
fn logical_native_nested_pause_counts() {
    if std::env::var_os("LP02_NESTED_CHILD").is_none() {
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "logical_native_nested_pause_counts",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("LP02_NESTED_CHILD", "1")
            .env("RUM_PROFILE_LEXICAL_SUBSTITUTION", "1")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        println!("{}", String::from_utf8_lossy(&out.stdout));
        return;
    }

    let lease = Session::logical_profile_test_acquire();
    let mut s = session();
    for (source, second) in [
        (
            "LpSpan[LpSpan[InputString[\"one\"]; InputString[\"two\"]]]",
            4,
        ),
        (
            "LpSpan[LpSpan[InputString[\"one\"]]; InputString[\"two\"]]",
            3,
        ),
    ] {
        Session::logical_profile_test_reset(&lease);
        paused(s.eval(source));
        assert_eq!(counts()[9], 2);
        assert_eq!(Session::logical_profile_test_pauses_for("native"), 2);
        paused(s.resume_with_input("first"));
        assert_eq!(Session::logical_profile_test_pauses_for("native"), second);
        done(s.resume_with_input("second"));
        closed([2, 2, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(Session::logical_profile_test_pauses_for("native"), second);
        println!("LP02_NATIVE nested-pause source={source} pauses=2,{second},{second}");
    }
}

#[test]
fn logical_native_source_scope_survives_resume() {
    if std::env::var_os("LP02_SCOPE_CHILD").is_none() {
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "logical_native_source_scope_survives_resume",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("LP02_SCOPE_CHILD", "1")
            .env("RUM_PROFILE_LEXICAL_SUBSTITUTION", "1")
            .env("RUM_PROFILE_EVALUATION_CACHE", "1")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        println!("{}", String::from_utf8_lossy(&out.stdout));
        return;
    }

    let lease = Session::logical_profile_test_acquire();
    let mut s = session();
    for (arrangement, paused_source, control_source) in [
        (
            "outside",
            "LpScope[LpSpan[InputString[\"one\"]; LpSpan[2]]]",
            "LpScope[LpSpan[1; LpSpan[2]]]",
        ),
        (
            "inside",
            "LpSpan[LpScope[InputString[\"one\"]; LpSpan[2]]]",
            "LpSpan[LpScope[1; LpSpan[2]]]",
        ),
    ] {
        Session::logical_profile_test_reset(&lease);
        paused(s.eval(paused_source));
        let before = native_rows();
        done(s.resume_with_input("reply"));
        let resumed = native_rows();
        Session::logical_profile_test_reset(&lease);
        done(s.eval(control_source));
        let control = native_rows();
        println!(
            "LP02_SCOPE arrangement={arrangement} before={before:?} resumed={resumed:?} control={control:?}"
        );
        assert_eq!(resumed, control, "arrangement={arrangement}");
        assert_eq!(Session::logical_profile_test_leases(), 0);
        Session::logical_profile_test_assert_quiescent();
        println!("LP02_TEARDOWN pending_children=0 terminal_parents=0 open=0 leases=0 faults=0");
    }
}

#[test]
fn logical_native_lifecycle() {
    if std::env::var_os("LP02_NATIVE_CHILD").is_none() {
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "logical_native_lifecycle",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("LP02_NATIVE_CHILD", "1")
            .env("RUM_PROFILE_LEXICAL_SUBSTITUTION", "1")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        println!("{}", String::from_utf8_lossy(&out.stdout));
        return;
    }

    let lease = Session::logical_profile_test_acquire();
    let mut s = session();
    paused(s.eval("LpSpan[InputString[\"first\"]; InputString[\"second\"]]"));
    assert_eq!(counts()[0], 0);
    assert_eq!(counts()[9], 1);
    assert_eq!(Session::logical_profile_test_pauses_for("native"), 1);
    paused(s.resume_with_input("one"));
    assert_eq!(counts()[0], 0);
    assert_eq!(counts()[9], 1);
    assert_eq!(Session::logical_profile_test_pauses_for("native"), 2);
    done(s.resume_with_input("two"));
    closed([1, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(Session::logical_profile_test_pauses_for("native"), 2);
    println!("LP02_NATIVE second-pause success closed-once pauses=2");

    for (source, subset) in [
        ("LpSpan[InputString[]; Abort[]]", None),
        ("LpSpan[InputString[]; Return[7]]", Some(5)),
        ("Catch[LpSpan[InputString[]; Throw[8]]]", Some(6)),
    ] {
        Session::logical_profile_test_reset(&lease);
        paused(s.eval(source));
        done(s.resume_with_input("reply"));
        let mut expected = [1, 0, 1, 0, 0, 0, 0, 0, 0, 0];
        if let Some(index) = subset {
            expected[index] = 1;
        }
        closed(expected);
    }
    println!("LP02_NATIVE error return throw closed-once");

    Session::logical_profile_test_reset(&lease);
    let cancel = s.cancel_handle();
    paused(s.eval("LpSpan[InputString[]]"));
    cancel.cancel();
    done(s.resume_with_input("ignored"));
    closed([1, 0, 1, 0, 0, 0, 0, 1, 0, 0]);
    println!("LP02_NATIVE cancellation closed-once");

    Session::logical_profile_test_reset(&lease);
    paused(s.eval("LpSpan[InputString[]]"));
    s.reset();
    closed([1, 0, 0, 0, 0, 0, 0, 0, 1, 0]);
    println!("LP02_NATIVE session-reset abandoned-once");

    Session::logical_profile_test_reset(&lease);
    paused(s.eval("LpSpan[InputString[]]"));
    drop(s);
    closed([1, 0, 0, 0, 0, 0, 0, 0, 1, 0]);
    println!("LP02_NATIVE session-drop abandoned-once");

    Session::logical_profile_test_reset(&lease);
    let mut a = session();
    let mut b = session();
    paused(a.eval("LpSpan[InputString[]]"));
    paused(b.eval("LpSpan[InputString[]]"));
    assert_eq!(counts()[9], 2);
    let mut a = std::thread::spawn(move || {
        done(a.resume_with_input("moved"));
        a
    })
    .join()
    .unwrap();
    done(b.resume_with_input("interleaved"));
    closed([2, 2, 0, 0, 0, 0, 0, 0, 0, 0]);
    println!("LP02_NATIVE interleaved moved-session closed-once");

    Session::logical_profile_test_reset(&lease);
    paused(a.eval("LpSpan[InputString[]; RumProfileReset[]; LpSpan[InputString[]]]"));
    paused(a.resume_with_input("first"));
    assert_eq!(counts()[9], 1);
    assert_eq!(counts()[0], 0);
    done(a.resume_with_input("second"));
    closed([1, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
    println!("LP02_NATIVE profile-reset active-call closed-once");

    Session::logical_profile_test_reset(&lease);
    assert!(a.restore_from_bytes(b"LpSpan[InputString[]]").is_err());
    closed([1, 0, 0, 0, 0, 0, 0, 0, 1, 0]);
    assert_eq!(Session::logical_profile_test_reasons()[2], 1);
    println!("LP02_NATIVE failed-restore abandoned-once");

    for (source, expected) in [
        ("ValueQ[LpSpan[InputString[]]]", 1),
        ("Monitor[Null,LpSpan[InputString[]]]", 2),
    ] {
        Session::logical_profile_test_reset(&lease);
        done(a.eval(source));
        closed([expected, 0, 0, 0, 0, 0, 0, 0, expected, 0]);
        let reasons = Session::logical_profile_test_reasons();
        assert_eq!(reasons[3] + reasons[4], expected);
    }
    println!("LP02_NATIVE rollback helper-discard abandoned-once");

    Session::logical_profile_test_reset(&lease);
    done(a.eval(
        "ValueQ[LpSpan[WithCleanup[InputString[\"body\"], LpSpan[InputString[\"cleanup\"]]]]]",
    ));
    closed([2, 0, 0, 0, 0, 0, 0, 0, 2, 0]);
    // The cleanup child's pause reaches its open parent.
    assert_eq!(Session::logical_profile_test_pauses_for("native"), 3);
    println!("LP02_NATIVE nested-cleanup re-suspended abandoned-once");

    Session::logical_profile_test_reset(&lease);
    paused(a.eval("LpSpan[Map[Function[InputString[]],{1,2}]]"));
    paused(a.resume_with_input("first"));
    done(a.resume_with_input("second"));
    closed([1, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
    println!("LP02_NATIVE typed-collection resumed completion");
}
