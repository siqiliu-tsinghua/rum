//! Real public snapshot restore, with a tail observer of the child's typed
//! InputRequired before restore turns it into its documented terminal failure.
use super::super::descriptors::{Fixture, Variant};
use super::primary::Descriptor;
use crate::{HostEvalOutcome, HostFailureCode, HostSession};
use rum_core::{Expr, SharedArgs};
use rum_runtime::kernel_api::{BuiltinSpec, HoldAttribute, MemoryGrowthContract};
use rum_runtime::{EvalError, RuntimeInputKind, RuntimeInputRequest, Session};
use std::cell::RefCell;
std::thread_local! {
    static OWNER: RefCell<Option<rum_runtime::RequestCancelHandle>> = const { RefCell::new(None) };
    static ATTEMPTS: RefCell<Vec<RuntimeInputRequest>> = const { RefCell::new(Vec::new()) };
}
fn attempt(s: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    let output = s.evaluate(args[0].clone());
    if let Err(EvalError::InputRequired { request }) = &output {
        ATTEMPTS.with(|rows| rows.borrow_mut().push(request.clone()));
        OWNER.with(|owner| *owner.borrow_mut() = Some(s.request_cancel_handle()));
    }
    output
}
fn install_observer(s: &mut Session) {
    ATTEMPTS.with(|rows| rows.borrow_mut().clear());
    OWNER.with(|owner| *owner.borrow_mut() = None);
    s.register_builtin(BuiltinSpec::ordinary_with_hold(
        "OracleRestoreAttempt",
        "Observe child input without owning continuation",
        HoldAttribute::HoldAll,
        attempt,
        MemoryGrowthContract::Delegated,
    ));
}
fn value(s: &mut HostSession, source: &str, expected: &str) {
    match s.eval(source) {
        HostEvalOutcome::Success(r) => {
            assert_eq!(r.value.text, expected);
            assert!(r.termination.is_none());
            assert!(r.messages.is_empty());
            assert!(r.prints.is_empty());
        }
        other => panic!("{source}: {other:?}"),
    }
}
pub fn image(twice: bool) -> String {
    let second = if twice {
        "OracleRestoreAttempt[Input[\"restore:4\"]];"
    } else {
        ""
    };
    format!(
        r#"DefinePackage["OracleRestore`",Public[{{get}}],secret=73;get[]:=secret]
Global`prefix=11;Global`log={{}};Global`suffix=0
RestoreSealedReferences["OracleRestore`",HoldComplete[WithCleanup[WithCleanup[(AppendTo[Global`log,1];Print[1];AppendTo[Global`log,2];Print[2];OracleRestoreAttempt[InputString["restore:2"]];AppendTo[Global`log,3];Print[3];{second}Global`suffix=1;AppendTo[Global`log,5];Print[5]),(AppendTo[Global`log,90])],(AppendTo[Global`log,91])]]]
Global`after=99"#
    )
}
fn restore(_: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    let twice = args[0].to_string() == "2";
    let mut s = HostSession::builder()
        .with_builtin_installer(install_observer)
        .build();
    value(&mut s, "Global`old=9", "9");
    let old = s.cancel_handle();
    let failure = s
        .restore_from_bytes(image(twice).as_bytes())
        .expect_err("restore must retire input");
    let expected = RuntimeInputRequest {
        kind: RuntimeInputKind::InputString,
        prompt: "restore:2".into(),
        password: false,
    };
    ATTEMPTS.with(|rows| {
        assert_eq!(
            *rows.borrow(),
            vec![expected.clone()],
            "actual child InputRequired; no second request"
        )
    });
    assert_eq!(failure.code, HostFailureCode::Other);
    assert_eq!(
        failure.message,
        "snapshot restore unexpectedly requested input"
    );
    assert_eq!(failure.prints, ["1", "2"]);
    assert!(failure.messages.is_empty(), "{failure:?}");
    assert!(failure.trace.is_empty());
    assert!(s.pending_input_request().is_none());
    match s.resume_with_input("unused") {
        HostEvalOutcome::Failure(f) => {
            assert_eq!(f.code, HostFailureCode::Other);
            assert_eq!(
                f.message,
                "no suspended evaluation is waiting for host input"
            );
        }
        other => panic!("retired restore resumed: {other:?}"),
    }
    value(
        &mut s,
        "{Global`prefix,Global`log,Global`suffix,ValueQ[Global`after],ValueQ[Global`old],OracleRestore`get[]}",
        "List[11, List[1, 2, 90, 91], 0, False, False, 73]",
    );
    value(&mut s, "1+1", "2");
    // Retired request identity cannot cancel the next accepted request.
    match s.eval("InputString[\"fresh restore\"]") {
        HostEvalOutcome::InputRequired(r) => assert_eq!(r.request.prompt, "fresh restore"),
        other => panic!("{other:?}"),
    }
    old.cancel();
    OWNER.with(|owner| {
        owner
            .borrow()
            .as_ref()
            .expect("restore owner observed at child input")
            .cancel()
    });
    match s.resume_with_input("fresh") {
        HostEvalOutcome::Success(r) => {
            assert_eq!(r.value.text, "\"fresh\"");
            assert!(r.termination.is_none());
        }
        other => panic!("{other:?}"),
    }
    value(&mut s, "Global`log", "List[1, 2, 90, 91]");
    ATTEMPTS.with(|rows| assert_eq!(*rows.borrow(), vec![expected]));
    println!(
        "RESTORE_BOUNDARY retired_twice={twice} child=InputString prompt=restore:2 cleanup=inner,outer pending=none resume=rejected fresh=PASS"
    );
    Ok(Expr::small_integer(73))
}
fn configure(s: &mut Session) {
    s.register_builtin(BuiltinSpec::ordinary(
        "OracleRestoreBoundary",
        "Drive public restore boundary",
        restore,
        MemoryGrowthContract::Delegated,
    ));
}
pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = Fixture::new(
        entry,
        if v == Variant::Twice {
            "OracleRestoreBoundary[2]"
        } else {
            "OracleRestoreBoundary[1]"
        },
    );
    f.configure = configure;
    f.expected_value = Some("73".into());
    f.expected_log = Some("List[]".into());
    f.expected_prints = Some(vec![]);
    f.expected_messages = Some(vec![]);
    f.pause_prints = Some(vec![]);
    f.pause_messages = Some(vec![]);
    f.pause_trace = Some(vec![]);
    f
}
pub fn descriptor() -> Descriptor {
    Descriptor {
        id: "RestoreSealedReferences.body",
        entry: "RestoreSealedReferences.body",
        factory: fixture,
        variants: &[Variant::Middle, Variant::Twice],
        boundary: true,
        lifecycle: |_| 0,
    }
}
