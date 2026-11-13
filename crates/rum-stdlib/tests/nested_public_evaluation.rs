//! Same-session raw entry refusal and the request/batch boundaries around it.
use rum_core::{Atom, Expr, SharedArgs};
use rum_runtime::kernel_api::BoundedDestruction;
use rum_runtime::{
    BuiltinSpec, EvalError, FileIo, FileIoError, FileIoProvider, FileProbe, FileWriteMode,
    HoldAttribute, MemoryGrowthContract, Session, SessionEvalResponse, SessionRequestError,
    SessionRequestFailure,
};
use std::path::Path;

const BOUNDARY_MESSAGE: &str = "nested top-level evaluation cannot suspend for host input";
type Handler = fn(&mut Session, &Expr, &SharedArgs) -> Result<Expr, EvalError>;

fn source(args: &SharedArgs) -> &str {
    let [Expr::Atom(Atom::String(source))] = args.as_ref() else {
        panic!("literal source required");
    };
    source
}
fn native_text(s: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    s.eval_text(source(args)).map(|outcome| outcome.value)
}
fn native_script(s: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    s.eval_script(source(args)).map(|outcome| outcome.value)
}
fn native_file(s: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    s.eval_file("nested-boundary.rum")
        .map(|outcome| outcome.value)
}
fn native_evaluate(s: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    s.evaluate(s.lower_text(source(args))?)
}
fn native_map(s: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    let value = s.eval_text(source(args))?.value;
    // This Rust tail executes only after synchronous success; it is not a continuation.
    s.eval_text("nativeTailCount++")?;
    Ok(Expr::call(Expr::system_symbol("List"), vec![value]))
}
fn native_resume(s: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    s.resume_eval_request_with_input("answer-one")
        .map(|outcome| outcome.value)
        .map_err(|failure| match failure.error {
            SessionRequestError::InputRequired { request } => EvalError::InputRequired { request },
            SessionRequestError::NestedEvaluationInputBoundary => {
                EvalError::NestedEvaluationInputBoundary
            }
            other => panic!("unexpected native resume failure: {other:?}"),
        })
}
struct Files(String);
impl BoundedDestruction for Files {}
impl FileIo for Files {
    fn read_to_string(&self, _: &Path) -> Result<String, FileIoError> {
        Ok(self.0.clone())
    }
    fn probe(&self, path: &Path) -> FileProbe {
        FileProbe::file(path.to_owned())
    }
    fn write_string(&self, _: &Path, _: &str, _: FileWriteMode) -> Result<(), FileIoError> {
        Err(FileIoError::new("read-only boundary fixture"))
    }
}
fn fresh() -> Session {
    let mut s = Session::new();
    rum_stdlib::install_core(&mut s);
    for (name, handler) in [
        ("NestedText", native_text as Handler),
        ("NestedScript", native_script),
        ("NestedFile", native_file),
        ("NestedEvaluate", native_evaluate),
        ("NestedMap", native_map),
        ("ResumeExisting", native_resume),
    ] {
        s.register_builtin(BuiltinSpec::ordinary_with_hold(
            name,
            "Public evaluation boundary fixture.",
            HoldAttribute::HoldAll,
            handler,
            MemoryGrowthContract::Incremental,
        ));
    }
    s
}
fn retired_and_usable(s: &mut Session) {
    assert!(s.pending_input_request().is_none());
    let failure = s.resume_eval_request_with_input("unused").unwrap_err();
    assert!(
        matches!(failure.error, SessionRequestError::Other { ref message }
        if message.contains("no suspended evaluation"))
    );
    let success = s.eval_request("6*7").unwrap();
    assert_eq!(success.value.to_string(), "42");
    assert!(success.prints.is_empty());
}
fn typed_refusal(
    result: Result<SessionEvalResponse, SessionRequestFailure>,
) -> SessionRequestFailure {
    let failure = result.expect_err("actual nested input must refuse");
    assert!(matches!(
        failure.error,
        SessionRequestError::NestedEvaluationInputBoundary
    ));
    assert_eq!(failure.error.to_string(), BOUNDARY_MESSAGE);
    failure
}

#[test]
fn raw_text_script_and_file_refuse_only_when_the_queue_is_empty() {
    let source = "InputString[\"first\"];Input[\"second\"]+2";
    for route in ["NestedText", "NestedScript", "NestedFile"] {
        for answers in [vec![], vec!["first"], vec!["first", "40"]] {
            let mut s = fresh();
            s.set_file_io(Some(FileIoProvider::new(Files(source.into()))));
            for answer in &answers {
                s.queue_input_response(*answer);
            }
            let result = s.eval_request(&format!("{route}[{}]", Expr::string(source)));
            if answers.len() == 2 {
                assert_eq!(result.unwrap().value.to_string(), "42", "{route}");
            } else {
                typed_refusal(result);
            }
            retired_and_usable(&mut s);
        }
    }
}

#[test]
fn raw_lowering_and_evaluation_keep_their_lower_level_contract() {
    for queued in [false, true] {
        let mut s = fresh();
        if queued {
            s.queue_input_response("40");
        }
        let expr = s.lower_text("Input[\"raw\"]+2").unwrap();
        let result = s.evaluate(expr);
        if queued {
            assert_eq!(result.unwrap().to_string(), "42");
            assert!(s.pending_input_request().is_none());
        } else {
            assert!(
                matches!(result, Err(EvalError::InputRequired { ref request })
                if request.prompt == "raw")
            );
            assert_eq!(s.pending_input_request().unwrap().prompt, "raw");
        }
    }
    // A lower-level native evaluate forwards the actual suspension to the
    // outer request driver; it does not enter the raw text finalization boundary.
    let mut s = fresh();
    let result = s.eval_request(r#"NestedEvaluate["Input[\"raw\"]+2"]"#);
    assert!(
        matches!(result, Err(ref f) if matches!(f.error, SessionRequestError::InputRequired { .. }))
    );
    assert_eq!(
        s.resume_eval_request_with_input("40")
            .unwrap()
            .value
            .to_string(),
        "42"
    );
    retired_and_usable(&mut s);
}

fn request(
    s: &mut Session,
    route: &str,
    source: &str,
) -> Result<SessionEvalResponse, SessionRequestFailure> {
    match route {
        "source" => s.eval_request(source),
        "expr" => {
            let expr = s.lower_text(source).unwrap();
            s.eval_expr_request(expr)
        }
        "accepted-source" => {
            let accepted = s.accept_cancellable_request().unwrap();
            s.eval_accepted_request(accepted, source)
        }
        "accepted-expr" => {
            let accepted = s.accept_cancellable_request().unwrap();
            let expr = s.lower_text(source).unwrap();
            s.eval_accepted_expr_request(accepted, expr)
        }
        _ => panic!("unknown request route"),
    }
}

#[test]
fn source_and_expr_request_drivers_preserve_repeated_pause_and_queued_results() {
    let source = "Print[1];InputString[\"first\"];Print[2];Input[\"second\"]+2";
    for route in ["source", "expr", "accepted-source", "accepted-expr"] {
        let mut queued = fresh();
        queued.queue_input_response("first");
        queued.queue_input_response("40");
        let expected = request(&mut queued, route, source).unwrap();
        let mut resumed = fresh();
        let first = request(&mut resumed, route, source).unwrap_err();
        assert!(
            matches!(&first.error, SessionRequestError::InputRequired { request }
            if request.prompt == "first")
        );
        assert_eq!(first.prints, ["1"]);
        let second = resumed.resume_eval_request_with_input("first").unwrap_err();
        assert!(
            matches!(&second.error, SessionRequestError::InputRequired { request }
            if request.prompt == "second")
        );
        assert_eq!(second.prints, ["1", "2"]);
        assert_eq!(
            resumed.resume_eval_request_with_input("40").unwrap(),
            expected,
            "{route}"
        );
        retired_and_usable(&mut resumed);
    }
}

#[test]
fn request_drivers_forward_the_typed_nested_failure() {
    for route in ["source", "expr", "accepted-source", "accepted-expr"] {
        let mut s = fresh();
        typed_refusal(request(
            &mut s,
            route,
            r#"NestedText["InputString[\"inner\"]"]"#,
        ));
        retired_and_usable(&mut s);
    }
}

#[test]
fn script_file_and_snapshot_batches_keep_the_nonresumable_boundary() {
    let source = "x=1\nInputString[\"batch\"]\nx=2";
    for route in [
        "script",
        "file",
        "accepted-script",
        "accepted-file",
        "snapshot-batch",
    ] {
        for queued in [false, true] {
            let mut s = fresh();
            s.set_file_io(Some(FileIoProvider::new(Files(source.into()))));
            if queued {
                s.queue_input_response("ready");
            }
            let result = match route {
                "script" => s.eval_script_request(source),
                "file" => s.eval_file_request("batch-boundary.rum"),
                "accepted-script" => {
                    let accepted = s.accept_cancellable_request().unwrap();
                    s.eval_accepted_script_request(accepted, source)
                }
                "accepted-file" => {
                    let accepted = s.accept_cancellable_request().unwrap();
                    s.eval_accepted_file_request(accepted, "batch-boundary.rum")
                }
                "snapshot-batch" => s.eval_snapshot_restore_request(source),
                _ => unreachable!(),
            };
            if queued {
                assert_eq!(result.unwrap().value.to_string(), "2", "{route}");
                retired_and_usable(&mut s);
            } else {
                assert!(
                    matches!(result, Err(ref f)
                    if matches!(&f.error, SessionRequestError::InputRequired { request }
                        if request.prompt == "batch")),
                    "{route}: {result:?}"
                );
                let before = s.pending_input_request().cloned();
                let failure = s.resume_eval_request_with_input("unused").unwrap_err();
                assert!(
                    matches!(failure.error, SessionRequestError::Other { ref message }
                    if message.contains("no suspended evaluation"))
                );
                assert_eq!(s.pending_input_request(), before.as_ref(), "{route}");
                assert!(s.eval_request("42").is_err(), "batch still owns admission");
                assert_eq!(s.pending_input_request(), before.as_ref());
            }
        }
    }
}

#[test]
fn snapshot_terminal_input_retires_its_owner_and_keeps_partial_state() {
    let mut s = fresh();
    let failure = s
        .restore_snapshot_request("x=1\nInputString[\"restore\"]\nx=2")
        .unwrap_err();
    assert!(
        matches!(failure.error, SessionRequestError::Other { ref message }
        if message == "snapshot restore unexpectedly requested input")
    );
    retired_and_usable(&mut s);
    assert_eq!(s.eval_request("x").unwrap().value.to_string(), "1");
}

#[test]
fn native_success_mapping_runs_once_and_its_rust_tail_is_never_resumed() {
    for queued in [false, true] {
        let mut s = fresh();
        s.eval_request("nativeTailCount=0").unwrap();
        if queued {
            s.queue_input_response("40");
        }
        let result = s.eval_request(r#"NestedMap["Input[\"inner\"]+2"]"#);
        if queued {
            assert_eq!(result.unwrap().value.to_string(), "List[42]");
        } else {
            typed_refusal(result);
        }
        retired_and_usable(&mut s);
        assert_eq!(
            s.eval_request("nativeTailCount").unwrap().value.to_string(),
            if queued { "1" } else { "0" }
        );
    }
}

#[test]
fn refusal_abandons_inner_then_outer_cleanup_exactly_once() {
    let mut s = fresh();
    let failure = typed_refusal(s.eval_request(
        r#"x=0;NestedText["WithCleanup[WithCleanup[InputString[\"inner\"],(x=10*x+1;Print[\"inner\"])],(x=10*x+2;Print[\"outer\"])]"]"#,
    ));
    assert_eq!(failure.prints, ["inner", "outer"]);
    assert_eq!(s.eval_request("x").unwrap().value.to_string(), "12");
    retired_and_usable(&mut s);
    assert_eq!(s.eval_request("x").unwrap().value.to_string(), "12");
}

#[test]
fn no_input_return_throw_abort_and_syntax_behavior_is_unchanged() {
    for source in ["Return[1]", "Throw[1]", "Abort[]", "Break[]", "1+1", "("] {
        let expected = fresh()
            .eval_text(source)
            .map(|outcome| outcome.value)
            .map_err(|error| error.to_string());
        let mut s = fresh();
        let actual = s
            .eval_text(&format!("NestedText[{}]", Expr::string(source)))
            .map(|outcome| outcome.value)
            .map_err(|error| error.to_string());
        assert_eq!(actual, expected, "{source}");
        retired_and_usable(&mut s);
    }
}

const OWNER: &str = "WithCleanup[(Print[\"owner-start\"];InputString[\"owner-first\"];Print[\"owner-between\"];InputString[\"owner-second\"];Print[\"owner-finish\"];42),Print[\"owner-cleanup\"]]";

#[test]
fn nested_reentrant_resume_preserves_the_real_preexisting_owner() {
    let mut s = fresh();
    let first = s.eval_request(OWNER).unwrap_err();
    assert!(
        matches!(&first.error, SessionRequestError::InputRequired { request }
        if request.prompt == "owner-first")
    );
    assert_eq!(first.prints, ["owner-start"]);
    assert_eq!(s.pending_input_request().unwrap().prompt, "owner-first");
    // The native callback resumes the stored owner and reaches its second
    // real pause. The raw entry must not abandon that request on its way out.
    assert!(matches!(
        s.eval_text("ResumeExisting[]"),
        Err(EvalError::NestedEvaluationInputBoundary)
    ));
    assert_eq!(s.pending_input_request().unwrap().prompt, "owner-second");
    let complete = s.resume_eval_request_with_input("answer-two").unwrap();
    assert_eq!(complete.value.to_string(), "42");
    assert_eq!(
        complete.prints,
        [
            "owner-start",
            "owner-between",
            "owner-finish",
            "owner-cleanup"
        ]
    );
    retired_and_usable(&mut s);
}

#[test]
fn normal_raw_entry_and_overlap_rejection_leave_the_pending_owner_untouched() {
    let mut s = fresh();
    s.eval_request(OWNER).unwrap_err();
    let before = s.pending_input_request().cloned();
    assert_eq!(s.eval_text("6*7").unwrap().value.to_string(), "42");
    assert_eq!(s.pending_input_request(), before.as_ref());
    let rejected = s
        .eval_request(r#"NestedText["InputString[\"nested\"]"]"#)
        .unwrap_err();
    assert!(
        matches!(rejected.error, SessionRequestError::Other { ref message }
        if message.contains("cannot start a new evaluation"))
    );
    assert_eq!(s.pending_input_request(), before.as_ref());
    let second = s.resume_eval_request_with_input("one").unwrap_err();
    assert_eq!(second.prints, ["owner-start", "owner-between"]);
    let complete = s.resume_eval_request_with_input("two").unwrap();
    assert_eq!(
        complete.prints,
        [
            "owner-start",
            "owner-between",
            "owner-finish",
            "owner-cleanup"
        ]
    );
    retired_and_usable(&mut s);
}

#[test]
fn native_resume_can_complete_the_existing_owner_synchronously() {
    let mut s = fresh();
    s.eval_request("WithCleanup[(InputString[\"first\"];42),Print[\"cleanup\"]]")
        .unwrap_err();
    assert_eq!(
        s.eval_text("ResumeExisting[]").unwrap().value.to_string(),
        "42"
    );
    retired_and_usable(&mut s);
}

#[test]
fn resumed_block_and_other_legacy_error_routes_forward_the_typed_refusal() {
    let nested = r#"NestedText["InputString[\"nested\"]"]"#;
    for (source, resumed) in [
        (nested.to_owned(), false),
        (format!("Block[{{x=1}},{nested}]"), false),
        (format!("Block[{{x=1,y=2}},{nested}]"), false),
        // This supported public route hit the old typed-only wire guard.
        (
            format!("Block[{{x=1}},InputString[\"outer\"];{nested}]"),
            true,
        ),
        (
            format!("Block[{{x=InputString[\"outer\"]}},{nested}]"),
            true,
        ),
        (format!("Table[{nested},{{i,1}}]"), false),
        (
            format!("Table[(InputString[\"outer\"];{nested}),{{i,1}}]"),
            true,
        ),
        (format!("Cases[{{1}},_,{nested}]"), false),
        (
            format!("Cases[{{1}},_,(InputString[\"outer\"];{nested})]"),
            true,
        ),
        (format!("Position[{{1}},_,{nested}]"), false),
    ] {
        let mut s = fresh();
        let mut result = s.eval_request(&source);
        if resumed {
            assert!(
                matches!(&result, Err(f)
                if matches!(&f.error, SessionRequestError::InputRequired { request }
                    if request.prompt == "outer")),
                "{source}: {result:?}"
            );
            result = s.resume_eval_request_with_input("outer-answer");
        }
        typed_refusal(result);
        retired_and_usable(&mut s);
    }
}

#[test]
fn guest_test_report_and_get_keep_their_capture_and_loading_boundaries() {
    let nested = r#"NestedText["InputString[\"nested\"]"]"#;
    for resumed in [false, true] {
        for guest in ["TestReport", "Get"] {
            let mut s = fresh();
            let body = if resumed {
                format!("InputString[\"outer\"];{nested}")
            } else {
                nested.to_owned()
            };
            s.set_file_io(Some(FileIoProvider::new(Files(body.clone()))));
            let source = if guest == "TestReport" {
                format!("Lookup[TestReport[{{TestCreate[({body}),42]}}],\"TestsFailedCount\"]")
            } else {
                "Get[\"nested-boundary.rum\"]".into()
            };
            let mut result = s.eval_request(&source);
            if resumed {
                assert!(matches!(&result, Err(f)
                    if matches!(&f.error, SessionRequestError::InputRequired { request }
                        if request.prompt == "outer")));
                assert_eq!(s.pending_input_request().unwrap().prompt, "outer");
                result = s.resume_eval_request_with_input("outer-answer");
            }
            let outcome = result.unwrap();
            if guest == "TestReport" {
                assert_eq!(outcome.value.to_string(), "1");
            } else {
                assert_eq!(outcome.value.to_string(), "$Failed");
                assert_eq!(outcome.messages.len(), 1);
                assert_eq!(outcome.messages[0].symbol, "Get");
                assert_eq!(outcome.messages[0].tag, "load");
                assert_eq!(outcome.messages[0].text, BOUNDARY_MESSAGE);
            }
            retired_and_usable(&mut s);
        }
    }
}

#[test]
fn ordinary_guest_test_report_and_get_still_resume() {
    for source in [
        r#"Lookup[TestReport[{TestCreate[InputString["guest"],"ok"]}],"TestsSucceededCount"]"#,
        r#"Get["ordinary.rum"]"#,
    ] {
        let mut s = fresh();
        s.set_file_io(Some(FileIoProvider::new(Files(
            "InputString[\"guest\"];1".into(),
        ))));
        let result = s.eval_request(source);
        assert!(matches!(result, Err(ref f)
            if matches!(&f.error, SessionRequestError::InputRequired { request }
                if request.prompt == "guest")));
        assert_eq!(
            s.resume_eval_request_with_input("ok")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        retired_and_usable(&mut s);
    }
}
