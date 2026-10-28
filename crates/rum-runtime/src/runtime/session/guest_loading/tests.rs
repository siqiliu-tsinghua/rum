use super::*;
use crate::runtime::test_builtins::session_with_test_control_builtins;
use std::cell::{Cell, RefCell};
use std::sync::Mutex;
use std::time::{Duration, Instant};

thread_local! {
    static CLOSES: RefCell<Vec<(String, usize, bool, String)>> = const { RefCell::new(Vec::new()) };
    static RESTORES: Cell<usize> = const { Cell::new(0) };
}
pub(in crate::runtime) fn record_restore() {
    RESTORES.with(|count| count.set(count.get() + 1));
}
pub(super) fn record_close(session: &Session, path: &Path, depth: usize, rollback: bool) {
    let marker = session
        .definitions
        .iter()
        .find(|(symbol, _)| symbol.context() == "Global`" && symbol.name() == "marker")
        .and_then(|(_, definition)| definition.own_value.as_ref())
        .map(ToString::to_string)
        .unwrap_or_default();
    CLOSES.with(|events| {
        events.borrow_mut().push((
            path.file_name().unwrap().to_string_lossy().into_owned(),
            depth,
            rollback,
            marker,
        ))
    });
}

struct Files(BTreeMap<String, String>);
impl crate::runtime::host_payload::BoundedDestruction for Files {}
impl FileIo for Files {
    fn read_to_string(&self, path: &Path) -> Result<String, FileIoError> {
        self.0
            .get(path.file_name().unwrap().to_str().unwrap())
            .cloned()
            .ok_or_else(|| FileIoError::new("missing test file"))
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
    CLOSES.with(|events| events.borrow_mut().clear());
    RESTORES.with(|count| count.set(0));
    let mut s = session_with_test_control_builtins();
    s.set_file_io(Some(FileIoProvider::new(Files(
        files
            .iter()
            .map(|(name, source)| ((*name).into(), (*source).into()))
            .collect(),
    ))));
    s.eval_request("Global`marker=0").unwrap();
    s
}
fn suspended(result: Result<SessionEvalResponse, SessionRequestFailure>, prompt: &str) {
    assert!(matches!(result, Err(SessionRequestFailure {
        error: SessionRequestError::InputRequired { request }, ..
    }) if request.prompt == prompt));
}
fn idle(s: &mut Session) {
    assert!(s.pending_suspension.is_none());
    assert!(s.resuming_suspension.is_none());
    assert!(s.abandonment_worklist.is_empty());
    assert!(!s.abandonment_active);
    assert!(s.pending_eval_request.is_none());
    assert!(s.pending_input_request.is_none());
    assert!(s.guest_form.is_none());
    assert!(s.load_stack.is_empty());
    assert!(s.evaluation_stack.is_empty());
    assert_eq!(s.evaluation_depth.get(), 0);
    assert_eq!(s.eval_request("1+1").unwrap().value.to_string(), "2");
}
const NESTED: &[(&str, &str)] = &[
    (
        "Outer.rum",
        r#"Global`marker=1
Get["Inner`"]
InputString["outer: "]
Global`marker=4
77"#,
    ),
    (
        "Inner.rum",
        r#"Global`marker=2
InputString["first: "]
InputString["second: "]
44"#,
    ),
];

#[test]
fn nested_files_close_once_and_all_owner_queues_empty() {
    let mut s = session(NESTED);
    suspended(s.eval_request(r#"Get["Outer`"]"#), "first: ");
    assert_eq!(s.load_stack.len(), 2);
    assert!(CLOSES.with(|events| events.borrow().is_empty()));
    suspended(s.resume_eval_request_with_input("one"), "second: ");
    assert_eq!(s.load_stack.len(), 2);
    suspended(s.resume_eval_request_with_input("two"), "outer: ");
    assert_eq!(s.load_stack.len(), 1);
    assert_eq!(
        s.resume_eval_request_with_input("three")
            .unwrap()
            .value
            .to_string(),
        "77"
    );
    assert_eq!(
        CLOSES.with(|events| events.borrow().clone()),
        vec![
            ("Inner.rum".into(), 1, false, "2".into()),
            ("Outer.rum".into(), 0, false, "4".into()),
        ]
    );
    assert_eq!(RESTORES.with(Cell::get), 0);
    idle(&mut s);
}

#[test]
fn cancellation_restores_nested_snapshots_inner_to_outer_at_each_input() {
    for stop in 0..3 {
        let mut s = session(NESTED);
        let cancel = s.request_cancel_handle();
        suspended(s.eval_request(r#"Get["Outer`"]"#), "first: ");
        if stop >= 1 {
            suspended(s.resume_eval_request_with_input("one"), "second: ");
        }
        if stop == 2 {
            suspended(s.resume_eval_request_with_input("two"), "outer: ");
        }
        cancel.cancel();
        assert_eq!(
            s.resume_eval_request_with_input("unused")
                .unwrap()
                .termination,
            Some(EvalTerminationReason::Cancelled)
        );
        let events = CLOSES.with(|events| events.borrow().clone());
        assert_eq!(
            events[0],
            (
                "Inner.rum".into(),
                1,
                stop < 2,
                if stop < 2 { "1" } else { "2" }.into()
            )
        );
        assert_eq!(events[1], ("Outer.rum".into(), 0, true, "0".into()));
        assert_eq!(events.len(), 2);
        assert_eq!(RESTORES.with(Cell::get), if stop < 2 { 2 } else { 1 });
        idle(&mut s);
    }
}

#[test]
fn dependency_exit_restores_package_frame_and_preserves_earlier_definitions() {
    for queued in [false, true] {
        let mut s = session(&[
            ("Earlier.rum", "Global`earlier=8\n8"),
            (
                "ExitDep.rum",
                r#"Global`marker=1
InputString["exit: "]
Exit[7]"#,
            ),
        ]);
        if queued {
            s.queue_input_response("one");
        }
        let result = s.eval_request(r#"BeginPackage["Main`",{"Earlier`","ExitDep`"}]"#);
        let result = if queued {
            result
        } else {
            suspended(result, "exit: ");
            assert_eq!(s.current_context, "Main`");
            assert_eq!(s.context_stack.len(), 1);
            s.resume_eval_request_with_input("one")
        };
        assert!(matches!(
            result,
            Err(SessionRequestFailure {
                error: SessionRequestError::Exit { code: 7 },
                ..
            })
        ));
        assert_eq!(s.current_context, "Global`");
        assert_eq!(s.context_path, vec!["System`", "Global`"]);
        assert!(s.context_stack.is_empty());
        assert!(s.loaded_packages.is_empty());
        assert_eq!(
            s.eval_request("Global`earlier").unwrap().value.to_string(),
            "8"
        );
        assert_eq!(RESTORES.with(Cell::get), 1);
        idle(&mut s);
    }
}

#[derive(Default)]
struct Roots {
    exprs: Vec<String>,
    symbols: Vec<String>,
}
impl FrameRootVisitor for Roots {
    fn visit_expr(&mut self, expr: &Expr) {
        self.exprs.push(expr.to_string());
    }
    fn visit_symbol(&mut self, symbol: &Symbol) {
        self.symbols.push(symbol.to_string());
    }
}
#[test]
fn pending_frames_trace_snapshot_definitions_and_previous_form_values() {
    let mut s = session(&[(
        "Roots.rum",
        r#"Global`kept=0
Hold[Global`bufferedRoot]
InputString["roots: "]
77"#,
    )]);
    s.eval_request("Global`kept=Hold[Global`snapshotRoot]")
        .unwrap();
    suspended(s.eval_request(r#"Get["Roots`"]"#), "roots: ");
    let mut roots = Roots::default();
    s.pending_suspension
        .as_ref()
        .unwrap()
        .trace_frame_roots(&mut roots);
    assert!(
        roots.exprs.iter().any(|expr| expr.contains("snapshotRoot")),
        "snapshot definition root missing: {:?}",
        roots.exprs
    );
    assert!(
        roots.exprs.iter().any(|expr| expr.contains("bufferedRoot")),
        "cursor value root missing: {:?}",
        roots.exprs
    );
    assert!(roots.symbols.iter().any(|symbol| symbol == "kept"));
    s.resume_eval_request_with_input("one").unwrap();
    idle(&mut s);
}

#[test]
fn paused_guest_form_reinstalls_original_deadline_before_child_frames() {
    thread_local! {
        static DEADLINE_CHECKS: Cell<(Option<Instant>, usize)> = const { Cell::new((None, 0)) };
    }
    fn require_deadline(s: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
        DEADLINE_CHECKS.with(|state| {
            let (deadline, checks) = state.get();
            assert_eq!(s.eval_deadline, deadline);
            state.set((deadline, checks + 1));
        });
        Ok(system_expr("Null"))
    }
    let mut s = session(&[(
        "Deadline.rum",
        r#"InputString["first: "];RequireOriginalDeadline[];InputString["second: "];RequireOriginalDeadline[]
77"#,
    )]);
    s.register_builtin(BuiltinSpec::ordinary(
        "RequireOriginalDeadline",
        "Test the guest deadline inside each resumed segment.",
        require_deadline,
        MemoryGrowthContract::NonAmplifying,
    ));
    s.set_eval_timeout(Some(Duration::from_secs(60)));
    suspended(s.eval_request(r#"Get["Deadline`"]"#), "first: ");
    let deadline = s.eval_deadline;
    assert!(deadline.is_some());
    DEADLINE_CHECKS.with(|state| state.set((deadline, 0)));
    // Guest work uses the captured deadline; each pause restores the enclosing state.
    let enclosing_deadline = Some(Instant::now() + Duration::from_secs(120));
    s.eval_deadline = enclosing_deadline;
    suspended(s.resume_eval_request_with_input("one"), "second: ");
    assert_eq!(s.eval_deadline, enclosing_deadline);
    DEADLINE_CHECKS.with(|state| assert_eq!(state.get().1, 1));
    s.resume_eval_request_with_input("two").unwrap();
    assert_eq!(s.eval_deadline, deadline);
    DEADLINE_CHECKS.with(|state| assert_eq!(state.get().1, 2));
    idle(&mut s);
}

#[test]
fn message_scope_and_print_sink_are_not_replayed_across_inputs() {
    let mut s = session(&[(
        "Streams.rum",
        r#"Message[Global`probe::tag]
Print["prefix"]
InputString["first: "];Global`before=$MessageList;Message[Global`probe::tag];Global`after=$MessageList;InputString["second: "];Print["suffix"]
77"#,
    )]);
    s.eval_request(r#"Global`probe::tag="message""#).unwrap();
    let prints = Arc::new(Mutex::new(Vec::new()));
    let sink = prints.clone();
    s.set_print_sink(Box::new(ClosurePrintSink::assert_bounded_destruction(
        move |text| {
            sink.lock().unwrap().push(text.to_string());
        },
    )));
    suspended(s.eval_request(r#"Get["Streams`"]"#), "first: ");
    assert_eq!(*prints.lock().unwrap(), vec!["prefix"]);
    suspended(s.resume_eval_request_with_input("one"), "second: ");
    assert_eq!(*prints.lock().unwrap(), vec!["prefix"]);
    let result = s.resume_eval_request_with_input("two").unwrap();
    assert_eq!(*prints.lock().unwrap(), vec!["prefix", "suffix"]);
    assert_eq!(result.messages.len(), 2);
    assert_eq!(
        s.eval_request("{Global`before,Global`after}")
            .unwrap()
            .value
            .to_string(),
        "List[List[], List[HoldForm[probe::tag]]]"
    );
    assert!(s.current_message_list_checkpoint.is_none());
    assert!(!s.current_message_list_dirty);
    assert!(s.handled_non_loader_eval_errors.is_empty());
    idle(&mut s);
}

fn seed_handled_error(s: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    s.handled_non_loader_eval_errors.push(RuntimeMessage {
        symbol: "GuestProbe".into(),
        tag: "handled".into(),
        text: "saved tail".into(),
    });
    s.refresh_memory_owner(
        crate::runtime::debug_tools::resources::MemoryOwner::HandledNonLoaderEvalErrors,
    );
    Ok(system_expr("Null"))
}

fn require_guest_environment(s: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    assert_eq!(s.handled_non_loader_eval_errors.len(), 1);
    assert_eq!(s.handled_non_loader_eval_errors[0].text, "saved tail");
    assert_eq!(s.current_message_list_checkpoint, Some(0));
    assert!(s.guest_form.is_some());
    assert_eq!(s.current_memory_limit, Some(100_000_000));
    Ok(system_expr("Null"))
}

#[test]
fn guest_environment_restores_handled_error_tail_and_memory_limit_before_child() {
    let mut s = session(&[(
        "Environment.rum",
        r#"SeedHandledError[];InputString["first: "];RequireGuestEnvironment[];InputString["second: "];RequireGuestEnvironment[]"#,
    )]);
    s.register_builtin(BuiltinSpec::ordinary(
        "SeedHandledError",
        "Test guest scope.",
        seed_handled_error,
        MemoryGrowthContract::NonAmplifying,
    ));
    s.register_builtin(BuiltinSpec::ordinary(
        "RequireGuestEnvironment",
        "Test guest scope.",
        require_guest_environment,
        MemoryGrowthContract::NonAmplifying,
    ));
    suspended(
        s.eval_request(r#"MemoryConstrained[Get["Environment`"],100000000]"#),
        "first: ",
    );
    let live_state = |s: &Session| {
        (
            s.handled_non_loader_eval_errors.clone(),
            s.current_message_list_checkpoint,
            s.current_message_list_dirty,
            s.last_message_list.clone(),
            s.guest_form,
            s.current_memory_limit,
            s.eval_deadline,
            s.current_time_deadline,
        )
    };
    assert!(s.handled_non_loader_eval_errors.is_empty());
    let first_pause = live_state(&s);
    suspended(s.resume_eval_request_with_input("one"), "second: ");
    assert_eq!(
        live_state(&s),
        first_pause,
        "pauses restore the enclosing state"
    );
    assert!(s.handled_non_loader_eval_errors.is_empty());
    s.resume_eval_request_with_input("two").unwrap();
    assert!(s.current_memory_limit.is_none());
    assert!(s.handled_non_loader_eval_errors.is_empty());
    idle(&mut s);
}

#[test]
fn abandonment_cleanup_loading_file_drains_nested_worklist() {
    let mut s = session(&[(
        "Cleanup.rum",
        r#"Global`marker=9
InputString["cleanup: "]
Global`marker=10"#,
    )]);
    let cancel = s.request_cancel_handle();
    suspended(
        s.eval_request(r#"WithCleanup[InputString["primary: "],Get["Cleanup`"]]"#),
        "primary: ",
    );
    cancel.cancel();
    assert_eq!(
        s.resume_eval_request_with_input("unused")
            .unwrap()
            .termination,
        Some(EvalTerminationReason::Cancelled)
    );
    assert_eq!(
        CLOSES.with(|events| events.borrow().clone()),
        vec![("Cleanup.rum".into(), 0, true, "0".into())]
    );
    assert_eq!(RESTORES.with(Cell::get), 1);
    idle(&mut s);
}
