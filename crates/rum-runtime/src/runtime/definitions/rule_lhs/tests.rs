use super::*;
use crate::runtime::suspension::AbandonReason;
use crate::runtime::test_builtins::session_with_test_control_builtins;

const DEPTH: usize = 2_000;

fn on_small_stack(test: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name("rule-lhs-2mib".into())
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
    for _ in 0..n {
        leaf = Expr::call(Expr::symbol("h"), vec![leaf]);
    }
    leaf
}

fn bottom(root: &Expr, n: usize) -> &Expr {
    let mut cursor = root;
    for _ in 0..n {
        let Expr::Call { head, args } = cursor else {
            panic!("short spine");
        };
        assert!(head.as_ref() == &Expr::symbol("h"));
        assert_eq!(args.len(), 1);
        cursor = &args[0];
    }
    cursor
}

fn drain(root: Expr) {
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        if let Expr::Call { head, args } = node {
            // Keep the children alive until the shared parent containers retire.
            pending.push(head.as_ref().clone());
            pending.extend(args.iter().cloned());
        }
    }
}

#[test]
fn rule_lhs_construction_and_teardown_control() {
    on_small_stack(|| {
        let root = spine(Expr::small_integer(0), DEPTH);
        assert!(bottom(&root, DEPTH) == &Expr::small_integer(0));
        drain(root);
        eprintln!("RULE_CONSTRUCTION_TEARDOWN n=2000 stack=2MiB passed");
    });
}

#[test]
fn rule_lhs_deep_classifier_true_and_false() {
    on_small_stack(|| {
        for (leaf, expected) in [
            (Expr::small_integer(0), false),
            (Expr::call(system_expr("Blank"), vec![]), true),
        ] {
            let root = spine(leaf, DEPTH);
            assert_eq!(definition_lhs_contains_pattern_syntax(&root), expected);
            drain(root);
        }
        eprintln!("RULE_CLASSIFIER n=2000 true/false stack=2MiB passed");
    });
}

fn prompt(out: &Result<Expr, EvalError>, expected: &str) {
    let Err(EvalError::InputRequired { request }) = out else {
        panic!("expected an input suspension");
    };
    assert_eq!(request.prompt, expected);
}

fn direct_walk(
    source: &'static str,
    inputs: &'static [&'static str],
    prompts: &'static [&'static str],
    error: bool,
    abandon: Option<usize>,
) {
    on_small_stack(move || {
        let mut session = session_with_test_control_builtins();
        session.eval_text("effect=0").unwrap();
        let leaf = shallow(&mut session, source);
        let root = spine(leaf, DEPTH);
        // Hold the source through retirement: generic last-owner Expr drop is
        // a separate boundary, not the work-stack depth contract.
        let mut out = session.evaluate_rule_lhs(root.clone());
        for (index, expected) in prompts.iter().enumerate() {
            prompt(&out, expected);
            if abandon == Some(index) {
                assert_eq!(
                    session.abandon_pending_suspension(AbandonReason::CancelledRequest),
                    AbandonReason::CancelledRequest
                );
                assert!(session.pending_suspension_generation().is_none());
                assert_eq!(session.eval_text("effect").unwrap().value.to_string(), "0");
                drop(out);
                drain(root);
                drop(session);
                eprintln!("RULE_DEEP n=2000 abandon_at={index} stack=2MiB passed");
                return;
            }
            out = session.resume_suspended_evaluation(inputs[index]);
        }
        assert!(session.pending_suspension_generation().is_none());
        if error {
            assert!(matches!(out, Err(EvalError::Thrown { .. })));
            assert_eq!(session.eval_text("effect").unwrap().value.to_string(), "0");
        } else {
            let result = out.expect("direct walk completes");
            let expected = shallow(&mut session, "h[x_,2,4]");
            assert!(bottom(&result, DEPTH) == &expected);
            drain(result);
        }
        assert_eq!(session.eval_text("1+1").unwrap().value.to_string(), "2");
        drain(root);
        drop(session);
        eprintln!(
            "RULE_DEEP n=2000 prompts={} error={error} stack=2MiB passed",
            prompts.len()
        );
    });
}

#[test]
fn rule_lhs_deep_ordinary() {
    direct_walk("h[x_,2,4]", &[], &[], false, None);
}
#[test]
fn rule_lhs_deep_one_suspension() {
    direct_walk(
        r#"h[x_,Input["first: "],4]"#,
        &["2"],
        &["first: "],
        false,
        None,
    );
}
#[test]
fn rule_lhs_deep_two_suspensions() {
    direct_walk(
        r#"h[x_,Input["first: "],Input["second: "]]"#,
        &["2", "4"],
        &["first: ", "second: "],
        false,
        None,
    );
}
#[test]
fn rule_lhs_deep_error_after_resume() {
    direct_walk(
        r#"h[x_,Input["first: "],Throw["boom"],(effect=99)]"#,
        &["2"],
        &["first: "],
        true,
        None,
    );
}
#[test]
fn rule_lhs_deep_abandon_at_first_and_second() {
    for stop in [0, 1] {
        direct_walk(
            r#"h[x_,Input["first: "],Input["second: "],(effect=99)]"#,
            &["2", "4"],
            &["first: ", "second: "],
            false,
            Some(stop),
        );
    }
}

struct Roots(Vec<Expr>);
impl FrameRootVisitor for Roots {
    fn visit_expr(&mut self, expr: &Expr) {
        self.0.push(expr.clone());
    }
    fn visit_symbol(&mut self, _: &Symbol) {
        panic!("no bare symbols in rule state");
    }
}
fn roots(value: &impl FrameRootTrace) -> Vec<Expr> {
    let mut visitor = Roots(Vec::new());
    value.trace_roots(&mut visitor);
    visitor.0
}
fn root_fixture() -> RuleLhsState {
    RuleLhsState {
        stack: vec![RuleLhsWork {
            original_head: Expr::small_integer(1),
            original_args: vec![
                Expr::small_integer(2),
                Expr::small_integer(3),
                Expr::small_integer(4),
            ]
            .into(),
            resolved_head: Some(Expr::small_integer(5)),
            prefix: vec![Expr::small_integer(6)],
            next: 1,
            policy: RuleLhsPolicy::OrdinaryCall,
        }],
    }
}
fn assert_roots(value: &impl FrameRootTrace, expected: &[Expr]) {
    assert_eq!(roots(value), expected, "all rule LHS roots must be traced");
}
#[test]
fn rule_lhs_roots_cover_every_field_and_destination() {
    let expected = (1..=7).map(Expr::small_integer).collect::<Vec<_>>();
    for destination in [
        RuleLhsDestination::Root,
        RuleLhsDestination::Head,
        RuleLhsDestination::Argument(1),
    ] {
        let frame = RulePatternSuspensionFrame {
            state: root_fixture(),
            pending: RuleLhsAwait {
                destination,
                pending_expr: Expr::small_integer(7),
            },
        };
        assert_roots(&frame, &expected);
    }
    for delayed in [false, true] {
        let frame = continuation::RuleLhsCompletionFrame {
            rhs: Expr::small_integer(8),
            delayed,
        };
        assert_roots(&frame, &[Expr::small_integer(8)]);
    }
    eprintln!(
        "RULE_ROOTS raw head, held/pending/unvisited args, resolved head, prefix, pending expression, RHS passed"
    );
}
#[test]
#[should_panic(expected = "all rule LHS roots must be traced")]
fn rule_lhs_missing_root_falsifier() {
    let mut state = root_fixture();
    state.stack[0].resolved_head = None;
    assert_roots(
        &state,
        &(1..=6).map(Expr::small_integer).collect::<Vec<_>>(),
    );
}

#[test]
fn rule_lhs_scalar_state_copy_trace_drop_and_abandon_20000() {
    on_small_stack(|| {
        let fixture = root_fixture();
        let state = RuleLhsState {
            stack: vec![fixture.stack[0].clone(); 20_000],
        };
        let expected = roots(&state);
        let copy = state.clone();
        assert_roots(&copy, &expected);
        drop(copy);
        let copy = continuation::rule_lhs_working_copy(&state);
        assert_roots(&copy, &expected);
        drop(copy);
        let mut session = session_with_test_control_builtins();
        let input = shallow(&mut session, r#"InputString["abandon: "]"#);
        let out = session.evaluate(input);
        prompt(&out, "abandon: ");
        let _: Result<Expr, EvalError> = {
            let session = &mut session;
            (|| {
                crate::resume_typed_or_suspend!(session, out,
                    ok(_) => panic!("the input producer must pause"),
                    suspend => RulePatternSuspensionFrame { state,
                        pending: RuleLhsAwait { destination: RuleLhsDestination::Argument(1), pending_expr: Expr::small_integer(7) } },
                )
            })()
        };
        assert_eq!(
            session.abandon_pending_suspension(AbandonReason::CancelledRequest),
            AbandonReason::CancelledRequest
        );
        assert!(session.pending_suspension_generation().is_none());
        drop(session);
        eprintln!(
            "RULE_SCALAR_STATE records=20000 ordinary/cold copy, trace, drop, pure abandon stack=2MiB passed"
        );
    });
}

#[test]
fn rule_lhs_error_resume_propagates_without_running_work() {
    let frame = RulePatternSuspensionFrame {
        state: root_fixture(),
        pending: RuleLhsAwait {
            destination: RuleLhsDestination::Argument(1),
            pending_expr: Expr::small_integer(7),
        },
    };
    let mut session = session_with_test_control_builtins();
    let action = crate::runtime::suspension::resume_typed_completion_for_test(
        frame,
        &mut session,
        Err(EvalError::Abort),
    );
    assert!(matches!(action, Err(EvalError::Abort)));
}

mod base_cases;
