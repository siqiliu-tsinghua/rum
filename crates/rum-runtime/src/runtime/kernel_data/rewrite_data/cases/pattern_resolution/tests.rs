use super::*;
use crate::runtime::suspension::{AbandonReason, SuspensionFrame};
use crate::runtime::test_builtins::session_with_test_control_builtins;
use std::sync::Arc;

const DEPTH: usize = 2_000;

fn on_small_stack(test: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name("held-pattern-2mib".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(test)
        .unwrap()
        .join()
        .unwrap();
}
fn shallow(session: &mut Session, source: &str) -> Expr {
    session
        .lower_syntax_expr(&parse_expr(source).unwrap())
        .unwrap()
}
fn spine(mut leaf: Expr, n: usize) -> Expr {
    let blank = Expr::call(system_expr("Blank"), vec![]);
    for _ in 0..n {
        leaf = Expr::call(Expr::symbol("g4Collapse"), vec![blank.clone(), leaf]);
    }
    leaf
}
fn drain(root: Expr) {
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        if let Expr::Call { head, args } = node {
            // Retain children while their shared parent containers retire.
            pending.push(head.as_ref().clone());
            pending.extend(args.iter().cloned());
        }
    }
}
#[test]
fn matcher_pattern_construction_and_teardown_control() {
    on_small_stack(|| {
        let root = spine(Expr::small_integer(0), DEPTH);
        let mut current = &root;
        for _ in 0..DEPTH {
            let Expr::Call { head, args } = current else {
                panic!("short spine");
            };
            assert!(head.as_ref() == &Expr::symbol("g4Collapse"));
            assert_eq!(args.len(), 2);
            current = &args[1];
        }
        assert!(current == &Expr::small_integer(0));
        drain(root);
        eprintln!("HELD_CONSTRUCTION_TEARDOWN n=2000 stack=2MiB passed");
    });
}
#[test]
fn matcher_pattern_deep_classifier_and_strip() {
    on_small_stack(|| {
        for (leaf, expected) in [
            (Expr::small_integer(0), false),
            (Expr::call(system_expr("Blank"), vec![]), true),
        ] {
            let mut root = leaf;
            for _ in 0..DEPTH {
                root = Expr::call(Expr::symbol("h"), vec![root]);
            }
            assert_eq!(contains_pattern_syntax(&root), expected);
            drain(root);
        }
        let blank = Expr::call(system_expr("Blank"), vec![]);
        let mut root = blank.clone();
        for _ in 0..DEPTH {
            root = Expr::call(system_expr("HoldPattern"), vec![root]);
        }
        assert!(strip_hold_pattern(&root) == &blank);
        assert!(contains_pattern_syntax(&root));
        drain(root);
        assert!(!contains_pattern_syntax(&Expr::call(
            system_expr("OptionsPattern"),
            vec![]
        )));
        eprintln!(
            "HELD_CLASSIFIER_AND_STRIP n=2000 true/false, unary strip, OptionsPattern excluded stack=2MiB passed"
        );
    });
}
fn prompt(out: &Result<Expr, EvalError>, expected: &str) {
    let Err(EvalError::InputRequired { request }) = out else {
        panic!("expected input");
    };
    assert_eq!(request.prompt, expected);
}
fn direct_walk(
    source: &'static str,
    inputs: &'static [&'static str],
    prompts: &'static [&'static str],
    error: bool,
    abandon: Option<usize>,
    prepared: bool,
) {
    on_small_stack(move || {
        for n in [16, DEPTH] {
            for queued in [true, false] {
                if queued && abandon.is_some() {
                    continue;
                }
                let mut session = session_with_test_control_builtins();
                session.eval_text(r#"effect=0;asks=0;g4Collapse[_,_]:=(effect=effect+1;0);g4Ask[_,_]:=(asks=asks+1;Input["prepared: "])"#).unwrap();
                let leaf = shallow(&mut session, source);
                let root = spine(leaf, n);
                if queued {
                    for input in inputs {
                        session.queue_input_response(*input);
                    }
                }
                // Keep the raw source until all frames retire. Generic last-owner
                // Expr drop is a separate boundary from the owned work stack.
                let mut out = session.resolve_matcher_pattern_expr_owned(root.clone());
                let mut abandoned = false;
                if !queued {
                    for (index, expected) in prompts.iter().enumerate() {
                        prompt(&out, expected);
                        if abandon == Some(index) {
                            assert_eq!(
                                session.abandon_pending_suspension(AbandonReason::CancelledRequest),
                                AbandonReason::CancelledRequest
                            );
                            abandoned = true;
                            break;
                        }
                        out = session.resume_suspended_evaluation(inputs[index]);
                    }
                }
                assert!(session.pending_suspension_generation().is_none());
                if abandoned || error {
                    if !abandoned {
                        assert!(matches!(out, Err(EvalError::Thrown { .. })));
                    }
                    assert_eq!(session.eval_text("effect").unwrap().value.to_string(), "0");
                } else {
                    assert!(
                        out.as_ref()
                            .is_ok_and(|value| value == &Expr::small_integer(0))
                    );
                    assert_eq!(
                        session.eval_text("effect").unwrap().value.to_string(),
                        n.to_string()
                    );
                }
                assert_eq!(
                    session.eval_text("asks").unwrap().value.to_string(),
                    if prepared { "1" } else { "0" }
                );
                assert_eq!(session.eval_text("1+1").unwrap().value.to_string(), "2");
                drop(out);
                drain(root);
                drop(session);
                eprintln!(
                    "HELD_DEEP n={n} queued={queued} prompts={} error={error} abandon={abandon:?} prepared={prepared} stack=2MiB passed",
                    prompts.len()
                );
            }
        }
    });
}
#[test]
fn matcher_pattern_deep_ordinary() {
    direct_walk("leaf[_,2,4]", &[], &[], false, None, false);
}
#[test]
fn matcher_pattern_deep_one_suspension() {
    direct_walk(
        r#"leaf[_,Input["first: "],4]"#,
        &["2"],
        &["first: "],
        false,
        None,
        false,
    );
}
#[test]
fn matcher_pattern_deep_two_suspensions() {
    direct_walk(
        r#"leaf[_,Input["first: "],Input["second: "]]"#,
        &["2", "4"],
        &["first: ", "second: "],
        false,
        None,
        false,
    );
}
#[test]
fn matcher_pattern_deep_error_after_resume() {
    direct_walk(
        r#"leaf[_,Input["first: "],Throw["boom"],(effect=99)]"#,
        &["2"],
        &["first: "],
        true,
        None,
        false,
    );
}
#[test]
fn matcher_pattern_deep_abandon_first_and_second() {
    for stop in [0, 1] {
        direct_walk(
            r#"leaf[_,Input["first: "],Input["second: "],(effect=99)]"#,
            &["2", "4"],
            &["first: ", "second: "],
            false,
            Some(stop),
            false,
        );
    }
}
#[test]
fn matcher_pattern_deep_prepared_call_suspension() {
    direct_walk("g4Ask[_,0]", &["7"], &["prepared: "], false, None, true);
}
#[test]
fn matcher_pattern_ordinary_root_resume_normalizes() {
    let mut session = session_with_test_control_builtins();
    session.eval_text("SetAttributes[f,Orderless]").unwrap();
    let source = shallow(
        &mut session,
        r#"(InputString["root: "];HoldComplete[f[2,1]])"#,
    );
    assert!(!contains_pattern_syntax(&source));
    let out = session.resolve_matcher_pattern_expr_owned(source);
    prompt(&out, "root: ");
    let result = session.resume_suspended_evaluation("done").unwrap();
    assert!(result == shallow(&mut session, "HoldComplete[f[1,2]]"));
    assert!(session.pending_suspension_generation().is_none());
}

struct Roots(Vec<Expr>);
impl FrameRootVisitor for Roots {
    fn visit_expr(&mut self, expr: &Expr) {
        self.0.push(expr.clone());
    }
    fn visit_symbol(&mut self, _: &Symbol) {
        panic!("no bare symbols");
    }
}
fn roots(value: &impl FrameRootTrace) -> Vec<Expr> {
    let mut visitor = Roots(Vec::new());
    value.trace_roots(&mut visitor);
    visitor.0
}
fn root_fixture() -> MatcherPatternState {
    MatcherPatternState {
        stack: vec![MatcherPatternWork {
            original_head: Expr::small_integer(1),
            original_args: (2..=4).map(Expr::small_integer).collect::<Vec<_>>().into(),
            resolved_head: Some(Expr::small_integer(5)),
            prefix: vec![Expr::small_integer(6)],
            next: 1,
        }],
    }
}
fn assert_roots(value: &impl FrameRootTrace, expected: &[Expr]) {
    assert_eq!(
        roots(value),
        expected,
        "all held pattern roots must be traced"
    );
}
#[test]
fn matcher_pattern_roots_cover_every_field_and_await_variant() {
    for destination in [
        MatcherPatternDestination::Root,
        MatcherPatternDestination::Head,
        MatcherPatternDestination::Argument(1),
    ] {
        for prepared in [false, true] {
            let pending = if prepared {
                MatcherPatternAwait::PreparedCall { destination }
            } else {
                MatcherPatternAwait::Evaluate {
                    destination,
                    pending_expr: Expr::small_integer(7),
                }
            };
            let frame = MatcherPatternSuspensionFrame {
                state: root_fixture(),
                pending,
            };
            assert_roots(
                &frame,
                &(1..=if prepared { 6 } else { 7 })
                    .map(Expr::small_integer)
                    .collect::<Vec<_>>(),
            );
        }
    }
    eprintln!(
        "HELD_ROOTS raw head, held/pending/unvisited args, resolved head, prefix, Evaluate/PreparedCall and all destinations passed"
    );
}
#[test]
#[should_panic(expected = "all held pattern roots must be traced")]
fn matcher_pattern_missing_root_falsifier() {
    let mut state = root_fixture();
    state.stack[0].resolved_head = None;
    assert_roots(
        &state,
        &(1..=6).map(Expr::small_integer).collect::<Vec<_>>(),
    );
}
#[test]
fn matcher_pattern_scalar_state_copy_trace_drop_abandon_20000() {
    on_small_stack(|| {
        for prepared in [false, true] {
            let fixture = root_fixture();
            let state = MatcherPatternState {
                stack: vec![fixture.stack[0].clone(); 20_000],
            };
            let expected = roots(&state);
            let copy = state.clone();
            assert_roots(&copy, &expected);
            drop(copy);
            let copy = continuation::matcher_pattern_working_copy(&state);
            assert_roots(&copy, &expected);
            drop(copy);
            let mut session = session_with_test_control_builtins();
            let input = shallow(&mut session, r#"InputString["abandon: "]"#);
            let out = session.evaluate(input);
            prompt(&out, "abandon: ");
            let destination = MatcherPatternDestination::Argument(1);
            let pending = if prepared {
                MatcherPatternAwait::PreparedCall { destination }
            } else {
                MatcherPatternAwait::Evaluate {
                    destination,
                    pending_expr: Expr::small_integer(7),
                }
            };
            crate::resume_or_suspend!(insert session, MatcherPatternSuspensionFrame { state, pending });
            assert_eq!(
                session.abandon_pending_suspension(AbandonReason::CancelledRequest),
                AbandonReason::CancelledRequest
            );
            assert!(session.pending_suspension_generation().is_none());
            drop(session);
        }
        eprintln!(
            "HELD_SCALAR_STATE records=20000 ordinary/cold copy, trace, drop, both await variants pure abandon stack=2MiB passed"
        );
    });
}
#[test]
fn matcher_pattern_error_resume_runs_no_work() {
    let frame = Arc::new(MatcherPatternSuspensionFrame {
        state: root_fixture(),
        pending: MatcherPatternAwait::PreparedCall {
            destination: MatcherPatternDestination::Argument(1),
        },
    });
    let mut session = session_with_test_control_builtins();
    assert!(matches!(
        frame.resume(&mut session, Err(EvalError::Abort)),
        crate::runtime::suspension::FrameAction::Complete(Err(EvalError::Abort))
    ));
}

mod base_cases;
