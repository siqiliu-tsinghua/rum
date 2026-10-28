//! Non-resumable batch ownership: embedding guide and brief-551 rulings.
use super::*;

mod restore;

const OUTER: &str = "Global`marker=5\nAppendTo[Global`log,1]\nGet[\"Inner`\"]\n99";
const INNER: &str = "AppendTo[Global`log,2]\nInputString[\"p: \"]\n7";
const STATE: &str = "{Global`log, Global`marker, $InputFileName}";

struct Files(BTreeMap<String, String>);
impl BoundedDestruction for Files {}
impl FileIo for Files {
    fn read_to_string(&self, path: &Path) -> Result<String, FileIoError> {
        self.0
            .get(path.file_name().unwrap().to_str().unwrap())
            .cloned()
            .ok_or_else(|| FileIoError::new("missing batch test file"))
    }
    fn probe(&self, path: &Path) -> FileProbe {
        if self
            .0
            .contains_key(path.file_name().unwrap().to_str().unwrap())
        {
            FileProbe::file(path.to_owned())
        } else {
            FileProbe::missing()
        }
    }
    fn write_string(&self, _: &Path, _: &str, _: FileWriteMode) -> Result<(), FileIoError> {
        Err(FileIoError::new("read-only"))
    }
}

fn session(files: &[(&str, &str)]) -> Session {
    let mut session = session_with_test_control_builtins();
    session.set_file_io(Some(FileIoProvider::new(Files(
        files
            .iter()
            .map(|(p, s)| ((*p).into(), (*s).into()))
            .collect(),
    ))));
    session
        .eval_request("Global`log={}; Global`marker=0")
        .unwrap();
    session
}

fn suspended(
    s: &Session,
    result: Result<SessionEvalResponse, SessionRequestFailure>,
    prompt: &str,
) {
    assert!(
        matches!(result, Err(SessionRequestFailure {
        error: SessionRequestError::InputRequired { request }, ..
    }) if request.prompt == prompt),
        "batch must actually suspend"
    );
    assert!(s.pending_suspension.is_some());
    assert_eq!(s.pending_input_request().unwrap().prompt, prompt);
    assert!(
        matches!(s.pending_request, Some(PendingRequest::Batch(_))),
        "batch has no resume checkpoint"
    );
}

fn idle(s: &mut Session) {
    assert!(s.pending_suspension.is_none());
    assert!(s.resuming_suspension.is_none());
    assert!(s.abandonment_worklist.is_empty());
    assert!(!s.abandonment_active);
    assert!(s.pending_request.is_none());
    assert!(s.pending_input_request().is_none());
    assert!(s.active_accepted_request.is_none());
    assert!(s.pending_accepted_request.borrow().is_none());
    assert!(s.load_stack.is_empty());
    assert!(s.guest_form.is_none());
    assert_eq!(s.eval_request("1+1").unwrap().value.to_string(), "2");
}

fn cancel_file(files: &[(&str, &str)], first_answer: bool, prompt: &str) {
    let mut s = session(files);
    if first_answer {
        // Batch input is queued; ordinary resume must remain unsupported.
        s.queue_input_response("answered once");
    }
    let cancel = s.request_cancel_handle();
    let result = s.eval_file_request("Outer.rum");
    suspended(&s, result, prompt);
    assert!(s.queued_input_responses.is_empty());
    assert!(matches!(
        s.resume_eval_request_with_input("not accepted")
            .unwrap_err()
            .error,
        SessionRequestError::Other { .. }
    ));
    assert!(matches!(
        s.eval_request("marker=100").unwrap_err().error,
        SessionRequestError::Other { .. }
    ));
    cancel.cancel();
    assert_eq!(
        s.resume_eval_request_with_input("unused")
            .unwrap()
            .termination,
        Some(EvalTerminationReason::Cancelled)
    );
    idle(&mut s);
    assert_eq!(
        s.eval_request(STATE).unwrap().value.to_string(),
        "List[List[], 0, \"\"]"
    );
    cancel.cancel();
    assert_eq!(s.eval_request("1+1").unwrap().value.to_string(), "2");
}

#[test]
fn public_file_cancel_nested() {
    cancel_file(&[("Outer.rum", OUTER), ("Inner.rum", INNER)], false, "p: ");
}

#[test]
fn public_file_cancel_two_levels() {
    cancel_file(
        &[
            (
                "Outer.rum",
                "Global`marker=5\nAppendTo[Global`log,1]\nGet[\"Middle`\"]\n99",
            ),
            ("Middle.rum", "AppendTo[Global`log,2]\nGet[\"Inner`\"]\n88"),
            (
                "Inner.rum",
                "AppendTo[Global`log,3]\nInputString[\"p: \"]\n7",
            ),
        ],
        false,
        "p: ",
    );
}

#[test]
fn public_file_first_input_answered_then_cancel_deeper_second() {
    cancel_file(
        &[
            (
                "Outer.rum",
                "Global`marker=5\nAppendTo[Global`log,1]\nInputString[\"first: \"]\nGet[\"Middle`\"]\n99",
            ),
            ("Middle.rum", "AppendTo[Global`log,2]\nGet[\"Inner`\"]\n88"),
            (
                "Inner.rum",
                "AppendTo[Global`log,3]\nInputString[\"second: \"]\n7",
            ),
        ],
        true,
        "second: ",
    );
}

#[test]
fn public_file_rollback_is_deferred_while_blocked() {
    let mut s = session(&[("Outer.rum", OUTER), ("Inner.rum", INNER)]);
    let cancel = s.request_cancel_handle();
    let result = s.eval_file_request("Outer.rum");
    suspended(&s, result, "p: ");
    assert_eq!(
        s.load_stack,
        vec![PathBuf::from("Outer.rum"), PathBuf::from("Inner.rum")]
    );
    let marker = s
        .definitions
        .iter()
        .find(|(symbol, _)| symbol.context() == "Global`" && symbol.name() == "marker")
        .unwrap()
        .1
        .own_value
        .as_ref()
        .unwrap();
    assert_eq!(
        marker.to_string(),
        "5",
        "read-only inspection sees unrolled-back state"
    );
    cancel.cancel();
    assert_eq!(
        s.resume_eval_request_with_input("unused")
            .unwrap()
            .termination,
        Some(EvalTerminationReason::Cancelled)
    );
    idle(&mut s);
}

#[test]
fn flat_public_file_cancellation_control() {
    cancel_file(
        &[(
            "Outer.rum",
            "Global`marker=5\nAppendTo[Global`log,1]\nInputString[\"p: \"]\n99",
        )],
        false,
        "p: ",
    );
}

#[test]
fn script_snapshot_and_guest_cancellation_controls() {
    for entry in 0..3 {
        let mut s = session(&[("Outer.rum", OUTER), ("Inner.rum", INNER)]);
        let cancel = s.request_cancel_handle();
        let result = match entry {
            0 => s.eval_script_request(OUTER),
            1 => s.eval_snapshot_restore_request(OUTER),
            _ => s.eval_request("Get[\"Outer`\"]"),
        };
        assert!(matches!(
            result.unwrap_err().error,
            SessionRequestError::InputRequired { .. }
        ));
        assert!(s.pending_suspension.is_some());
        cancel.cancel();
        assert_eq!(
            s.resume_eval_request_with_input("unused")
                .unwrap()
                .termination,
            Some(EvalTerminationReason::Cancelled)
        );
        idle(&mut s);
        assert_eq!(
            s.eval_request(STATE).unwrap().value.to_string(),
            if entry == 2 {
                "List[List[], 0, \"\"]"
            } else {
                "List[List[1], 5, \"\"]"
            }
        );
    }
}

#[test]
fn public_file_success_and_ordinary_error_control() {
    let mut s = session(&[("Outer.rum", OUTER), ("Inner.rum", INNER)]);
    s.queue_input_response("queued");
    assert_eq!(
        s.eval_file_request("Outer.rum").unwrap().value.to_string(),
        "99"
    );
    idle(&mut s);
    assert_eq!(
        s.eval_request(STATE).unwrap().value.to_string(),
        "List[List[1, 2], 5, \"\"]"
    );
    let mut s = session(&[(
        "Outer.rum",
        "Global`marker=5\nAppendTo[Global`log,1]\nThrow[3]",
    )]);
    assert!(s.eval_file_request("Outer.rum").is_err());
    idle(&mut s);
    assert_eq!(
        s.eval_request(STATE).unwrap().value.to_string(),
        "List[List[], 0, \"\"]"
    );
}

#[test]
fn public_file_snapshot_roots_survive_suspension() {
    use crate::runtime::suspension::trace::FrameRootVisitor;
    struct Roots(Vec<Expr>);
    impl FrameRootVisitor for Roots {
        fn visit_expr(&mut self, expr: &Expr) {
            self.0.push(expr.clone());
        }
        fn visit_symbol(&mut self, _: &Symbol) {}
    }
    let mut s = session(&[("Outer.rum", "snapshotOnly=0\nInputString[\"p: \"]")]);
    s.eval_request("snapshotOnly=\"only in the public file snapshot\"")
        .unwrap();
    let cancel = s.request_cancel_handle();
    let result = s.eval_file_request("Outer.rum");
    suspended(&s, result, "p: ");
    let mut roots = Roots(Vec::new());
    s.pending_suspension
        .as_ref()
        .unwrap()
        .trace_frame_roots(&mut roots);
    assert!(
        roots
            .0
            .contains(&Expr::string("only in the public file snapshot"))
    );
    cancel.cancel();
    assert_eq!(
        s.resume_eval_request_with_input("unused")
            .unwrap()
            .termination,
        Some(EvalTerminationReason::Cancelled)
    );
    idle(&mut s);
    assert_eq!(
        s.eval_request("snapshotOnly").unwrap().value,
        Expr::string("only in the public file snapshot")
    );
}

#[test]
fn public_file_cleanup_observes_inner_then_outer_state_once() {
    use std::sync::{Arc, Mutex};
    let mut s = session(&[
        (
            "Outer.rum",
            "marker=5\nWithCleanup[Get[\"Inner`\"], Print[marker]]",
        ),
        (
            "Inner.rum",
            "marker=6\nWithCleanup[InputString[\"p: \"], Print[marker]]",
        ),
    ]);
    let prints = Arc::new(Mutex::new(Vec::new()));
    let sink = prints.clone();
    s.set_print_sink(Box::new(ClosurePrintSink::assert_bounded_destruction(
        move |line| {
            sink.lock().unwrap().push(line.to_string());
        },
    )));
    let cancel = s.request_cancel_handle();
    let result = s.eval_file_request("Outer.rum");
    suspended(&s, result, "p: ");
    assert!(prints.lock().unwrap().is_empty());
    cancel.cancel();
    assert_eq!(
        s.resume_eval_request_with_input("unused")
            .unwrap()
            .termination,
        Some(EvalTerminationReason::Cancelled)
    );
    assert_eq!(*prints.lock().unwrap(), vec!["6", "5"]);
    idle(&mut s);
    assert_eq!(s.eval_request("marker").unwrap().value.to_string(), "0");
    assert_eq!(*prints.lock().unwrap(), vec!["6", "5"]);
}

// Deliberately broken, test-only callback: swallow InputRequired while keeping
// its owner installed. No production hook or fixture exists for this behavior.
fn leaked_completion(s: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    let expr = s.lower_text("InputString[\"leaked: \"]").unwrap();
    assert!(matches!(
        s.evaluate(expr),
        Err(EvalError::InputRequired { .. })
    ));
    assert!(s.pending_suspension.is_some());
    if args.is_empty() {
        Ok(Expr::integer(42.into()))
    } else {
        Err(EvalError::Exit { code: 3 })
    }
}

fn leaky_session() -> Session {
    let mut s = session(&[("Leak.rum", "Leak551[]")]);
    s.register_builtin(BuiltinSpec::ordinary(
        "Leak551",
        "Test-only terminal owner leak",
        leaked_completion,
        MemoryGrowthContract::Delegated,
    ));
    s
}

#[test]
#[should_panic(expected = "terminal request outcome retained a suspension owner")]
fn invariant_falsifier_value() {
    let _ = leaky_session().eval_request("Leak551[]");
}

#[test]
#[should_panic(expected = "terminal request outcome retained a suspension owner")]
fn invariant_falsifier_error() {
    let _ = leaky_session().eval_request("Leak551[1]");
}

#[test]
#[should_panic(expected = "terminal request outcome retained a suspension owner")]
fn invariant_falsifier_script() {
    let _ = leaky_session().eval_script_request("Leak551[]");
}

#[test]
#[should_panic(expected = "terminal request outcome retained a suspension owner")]
fn invariant_falsifier_file() {
    let _ = leaky_session().eval_file_request("Leak.rum");
}

#[test]
#[should_panic(expected = "terminal request outcome retained a suspension owner")]
fn invariant_falsifier_snapshot() {
    let _ = leaky_session().eval_snapshot_restore_request("Leak551[]");
}

/// Outside any request, a successful batch takes its own output off the
/// session lists, as `eval_text` does. A request's batch leaves it there for
/// the request to publish, then cuts at its admission mark.
#[test]
fn non_request_batches_take_their_output_off_the_session_lists() {
    const BATCH: &str = "Print[\"a\"]\nMessage[probe::m]\nPrint[\"b\"]";
    let mut s = session(&[("Batch.rum", BATCH)]);
    s.eval_request("probe::m = \"probe text\"").unwrap();
    for _ in 0..2 {
        for route in ["text", "script", "file"] {
            let outcome = match route {
                "text" => s.eval_text("Print[\"a\"]; Message[probe::m]; Print[\"b\"]"),
                "script" => s.eval_script(BATCH),
                _ => s.eval_file("Batch.rum"),
            }
            .unwrap();
            assert_eq!(outcome.prints, ["a", "b"], "{route}");
            assert_eq!(outcome.messages.len(), 1, "{route}");
            assert_eq!(outcome.messages[0].tag, "m", "{route}");
            assert_eq!(s.prints.len(), 0, "{route}");
            assert_eq!(s.messages.len(), 0, "{route}");
            assert_eq!(s.last_message_list, outcome.messages, "{route}");
        }
    }
    idle(&mut s);
}

/// A suspension the non-request `eval_text` left behind has no stored request.
/// Nothing reported anything for it, so the cancelled request that retires it
/// admits fresh at the current clock and reports only its cancel notice.
#[test]
fn cancelled_request_retires_a_non_request_suspension() {
    let mut s = session(&[]);
    s.eval_request("probe::m = \"probe text\"").unwrap();
    let first = s.eval_text("Message[probe::m]; InputString[\"p: \"]");
    assert!(matches!(first, Err(EvalError::InputRequired { .. })));
    assert!(s.pending_suspension.is_some());
    assert!(s.pending_request.is_none());
    let accepted = s.accept_cancellable_request().unwrap();
    accepted.cancel_handle().cancel();
    let retired = s.eval_accepted_request(accepted, "").unwrap();
    assert_eq!(retired.termination, Some(EvalTerminationReason::Cancelled));
    assert_eq!(
        retired
            .messages
            .iter()
            .map(|message| format!("{}::{}", message.symbol, message.tag))
            .collect::<Vec<_>>(),
        ["General::cancel"]
    );
    idle(&mut s);
}
