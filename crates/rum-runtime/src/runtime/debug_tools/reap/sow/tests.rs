use super::*;
use crate::runtime::suspension::AbandonReason;
use crate::runtime::test_builtins::session_with_test_control_builtins;

struct Roots(Vec<Expr>);
impl FrameRootVisitor for Roots {
    fn visit_expr(&mut self, expr: &Expr) {
        self.0.push(expr.clone());
    }
    fn visit_symbol(&mut self, _: &Symbol) {
        panic!("Sow state has no bare symbol field");
    }
}
fn assert_roots(state: &impl FrameRootTrace, expected: &[Expr]) {
    let mut roots = Roots(Vec::new());
    state.trace_roots(&mut roots);
    assert_eq!(roots.0, expected, "all Sow roots must be traced");
}
fn frozen() -> SowMatchState {
    SowMatchState {
        value: Expr::small_integer(77),
        tag: Expr::small_integer(2),
        scope_limit: 2,
        scope_index: 1,
        pattern_index: 3,
        pattern_count: 5,
        matched: vec![(0, 1), (1, 0)],
    }
}
#[test]
fn state_and_frame_trace_value_and_tag_through_cold_copy() {
    let state = frozen();
    let expected = [state.value.clone(), state.tag.clone()];
    assert_roots(&state, &expected);
    let frame = SowMatchSuspensionFrame {
        state: state.clone(),
    };
    assert_roots(&frame, &expected);
    assert_eq!(frame.state.matched, [(0, 1), (1, 0)]);
    assert_eq!((frame.state.scope_limit, frame.state.scope_index), (2, 1));
    assert_eq!(
        (frame.state.pattern_index, frame.state.pattern_count),
        (3, 5)
    );
}
#[test]
#[should_panic(expected = "all Sow roots must be traced")]
fn missing_tag_root_falsifier() {
    let state = frozen();
    assert_roots(&state.value, &[state.value.clone(), state.tag.clone()]);
}

fn session_with_patterns(patterns: &[&str]) -> Session {
    let mut session = session_with_test_control_builtins();
    let patterns = patterns
        .iter()
        .map(|source| {
            session
                .lower_syntax_expr(&parse_expr(source).unwrap())
                .unwrap()
        })
        .collect::<Vec<_>>();
    session.push_accounted_reap_scope(ReapScope {
        buckets_by_pattern: vec![ReapBucketGroup::default(); patterns.len()],
        patterns,
        pattern_list_form: true,
        handler: None,
    });
    session
}
fn start(session: &mut Session) -> Result<Expr, EvalError> {
    session.eval_sow(
        &system_expr("Sow"),
        &vec![Expr::small_integer(77), Expr::small_integer(2)].into(),
    )
}
fn pending(outcome: &Result<Expr, EvalError>, prompt: &str) {
    let Err(EvalError::InputRequired { request }) = outcome else {
        panic!("expected suspended Sow match: {outcome:?}");
    };
    assert_eq!(request.prompt, prompt);
}
fn assert_uncommitted(session: &Session) {
    assert!(
        session.reap_scopes[0]
            .buckets_by_pattern
            .iter()
            .all(|g| g.buckets.is_empty()),
        "Sow committed before all matches finished"
    );
}
#[test]
fn successive_match_suspensions_commit_only_after_the_last_answer() {
    let mut session = session_with_patterns(&[
        "_Integer",
        r#"_?Function[t,(InputString["first: "];False)]"#,
        r#"_?Function[t,(InputString["second: "];True)]"#,
    ]);
    pending(&start(&mut session), "first: ");
    assert_uncommitted(&session);
    pending(&session.resume_suspended_evaluation("no"), "second: ");
    assert_uncommitted(&session);
    assert_eq!(
        session.resume_suspended_evaluation("yes").unwrap(),
        Expr::small_integer(77)
    );
    assert!(session.pending_suspension_generation().is_none());
    let groups = &session.reap_scopes[0].buckets_by_pattern;
    assert!(groups[1].buckets.is_empty());
    for i in [0, 2] {
        assert_eq!(groups[i].buckets.len(), 1);
        assert_eq!(groups[i].buckets[0].tag, Expr::small_integer(2));
        assert_eq!(groups[i].buckets[0].values, [Expr::small_integer(77)]);
    }
}
#[test]
fn later_error_discards_the_accepted_prefix() {
    let mut session = session_with_patterns(&[
        "_Integer",
        r#"_?Function[t,(InputString["first: "];True)]"#,
        r#"_?Function[t,Throw["later"]]"#,
    ]);
    pending(&start(&mut session), "first: ");
    assert_uncommitted(&session);
    assert!(matches!(
        session.resume_suspended_evaluation("yes"),
        Err(EvalError::Thrown { .. })
    ));
    assert_uncommitted(&session);
    assert!(session.pending_suspension_generation().is_none());
}
#[test]
fn pure_abandon_runs_no_remaining_match_or_commit() {
    for stop in [0, 1] {
        let mut session = session_with_patterns(&[
            "_Integer",
            r#"_?Function[t,(InputString["first: "];True)]"#,
            r#"_?Function[t,(InputString["second: "];True)]"#,
            r#"_?Function[t,(effect=99;True)]"#,
        ]);
        session.eval_text("effect=0").unwrap();
        pending(&start(&mut session), "first: ");
        if stop == 1 {
            pending(&session.resume_suspended_evaluation("yes"), "second: ");
        }
        session.abandon_pending_suspension(AbandonReason::CancelledRequest);
        assert_uncommitted(&session);
        assert!(session.pending_suspension_generation().is_none());
        assert_eq!(session.eval_text("effect").unwrap().value.to_string(), "0");
        // The direct driver owns no scope cleanup; enclosing Reap frames do.
        assert_eq!(session.reap_scopes.len(), 1);
        session.pop_accounted_reap_scope().unwrap();
        assert_eq!(session.eval_text("1+1").unwrap().value.to_string(), "2");
    }
}
#[test]
#[should_panic(expected = "Sow committed before all matches finished")]
fn early_commit_falsifier() {
    let mut session = session_with_patterns(&["_Integer"]);
    record_sown_value(
        &mut session.reap_scopes[0].buckets_by_pattern[0],
        Expr::small_integer(2),
        Expr::small_integer(77),
    );
    assert_uncommitted(&session);
}
