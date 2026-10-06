use super::*;
use crate::runtime::suspension::{AbandonReason, FrameAction, SuspensionFrame};
use crate::runtime::test_builtins::session_with_test_control_builtins;
use std::sync::Arc;

struct Roots(Vec<Expr>);
impl FrameRootVisitor for Roots {
    fn visit_expr(&mut self, expr: &Expr) {
        self.0.push(expr.clone());
    }
    fn visit_symbol(&mut self, _: &Symbol) {
        panic!("no bare symbol roots");
    }
}
fn roots(value: &impl FrameRootTrace) -> Vec<Expr> {
    let mut roots = Roots(Vec::new());
    value.trace_roots(&mut roots);
    roots.0
}
fn n(value: i64) -> Expr {
    Expr::small_integer(value)
}
fn frozen(parts: bool) -> TraceFilterState {
    FilterWork {
        mode: TraceOutput::Print,
        original_value: n(1),
        pattern: Some(n(2)),
        levels: None,
        include_heads: false,
        current: LevelWork {
            remaining: vec![n(3)].into_iter(),
            emitted: vec![n(4)],
        },
        parents: vec![LevelWork {
            remaining: vec![n(5)].into_iter(),
            emitted: vec![n(6)],
        }],
    }
    .freeze_filter(LeafWork {
        original: n(7),
        underlying: vec![n(8), n(9)].into_iter(),
        field_index: 3,
        patterns: PatternWork::Parts(vec![n(10), n(11)]),
        pattern_index: 1,
        candidates: if parts {
            CandidateWork::Parts(vec![n(12), n(13)].into_iter())
        } else {
            CandidateWork::Single(Some(n(12)))
        },
        candidate_index: 5,
        selected: vec![n(14)],
    })
}
#[test]
fn every_frozen_root_survives_copy_thaw_and_frame_tracing() {
    for parts in [false, true] {
        let state = frozen(parts);
        let expected = (1..=14)
            .filter(|v| parts || *v != 13)
            .map(n)
            .collect::<Vec<_>>();
        assert_eq!(roots(&state), expected, "all filter roots");
        let (work, leaf) = state.thaw_filter();
        assert_eq!(
            (leaf.field_index, leaf.candidate_index, leaf.pattern_index),
            (3, 5, 1)
        );
        assert_eq!(roots(&work.freeze_filter(leaf)), expected);
        assert_eq!(
            roots(&TraceFilterSuspensionFrame {
                state: state.clone()
            }),
            expected
        );
    }
}
#[test]
#[should_panic(expected = "all filter roots")]
fn omitted_parent_prefix_root_falsifier() {
    let state = frozen(true);
    let (mut work, leaf) = state.thaw_filter();
    work.parents[0].emitted.clear();
    assert_eq!(
        roots(&work.freeze_filter(leaf)),
        roots(&state),
        "all filter roots"
    );
}
#[test]
fn false_advances_pattern_candidate_and_field_without_replaying() {
    let state = frozen(true);
    let (work, mut leaf) = state.thaw_filter();
    leaf.pattern_index = 0;
    assert!(!leaf.apply_match(false, false));
    assert_eq!(leaf.pattern_index, 1);
    assert_eq!(leaf.candidates.current(), Some(&n(12)));
    assert!(!leaf.apply_match(false, false));
    assert_eq!((leaf.pattern_index, leaf.candidate_index), (0, 6));
    assert_eq!(leaf.candidates.current(), Some(&n(13)));
    assert!(!leaf.apply_match(true, false));
    assert_eq!(leaf.selected, [n(14), n(13)]);
    let after = work.freeze_filter(leaf);
    assert!(
        !roots(&after).contains(&n(12)),
        "consumed candidate released"
    );
    let mut session = session_with_test_control_builtins();
    let (mut work, mut leaf) = state.thaw_filter();
    work.mode = TraceOutput::Detail;
    work.pattern = None;
    leaf.candidates = CandidateWork::Parts(Vec::new().into_iter());
    assert!(session.match_trace_leaf(&work, &mut leaf).unwrap());
    assert_eq!(leaf.field_index, 4);
    assert_eq!(leaf.candidates.current(), Some(&n(9)));
}
#[test]
fn detail_true_emits_original_once_and_skips_later_search() {
    let (mut work, mut leaf) = frozen(true).thaw_filter();
    work.mode = TraceOutput::Detail;
    assert!(leaf.apply_match(true, true));
    assert_eq!(
        (leaf.pattern_index, leaf.candidate_index, leaf.field_index),
        (1, 5, 3)
    );
    finish_leaf(&mut work, leaf, true);
    assert_eq!(work.current.emitted, [n(4), n(7)]);
}
#[test]
fn flat_ancestor_copy_and_drop_use_shallow_expressions() {
    let (mut work, leaf) = frozen(true).thaw_filter();
    for i in 0..512 {
        work.parents.push(LevelWork::from_items(vec![n(i)]));
    }
    let state = work.freeze_filter(leaf);
    let (work, leaf) = state.thaw_filter();
    assert_eq!(work.parents.len(), 513);
    assert_eq!(roots(&state), roots(&work.freeze_filter(leaf)));
}
#[test]
fn error_frame_propagates_without_thaw_match_or_print() {
    let mut session = session_with_test_control_builtins();
    let frame = Arc::new(TraceFilterSuspensionFrame {
        state: frozen(true),
    });
    let action = frame.resume(
        &mut session,
        Err(EvalError::Thrown {
            value: n(99),
            tag: system_expr("None"),
            explicit_tag: false,
        }),
    );
    assert!(matches!(
        action,
        FrameAction::Complete(Err(EvalError::Thrown { .. }))
    ));
}
fn start(session: &mut Session) -> Result<Expr, EvalError> {
    let pattern = session.lower_syntax_expr(&parse_expr(
        r#"{_?Function[t,(InputString["first: "];False)],_?Function[t,(InputString["second: "];True)],_?Function[t,(effect=99;True)]}"#
    ).unwrap()).unwrap();
    session.filter_trace_result(
        Expr::call(system_expr("List"), vec![trace_detail_hold(&n(2))]),
        TraceEvalState {
            original: n(0),
            pattern: Some(pattern),
            levels: None,
            include_heads: false,
            output: TraceOutput::Print,
        },
        n(77),
    )
}
fn pending(result: Result<Expr, EvalError>, prompt: &str) {
    let Err(EvalError::InputRequired { request }) = result else {
        panic!("expected input");
    };
    assert_eq!(request.prompt, prompt);
}
#[test]
fn successive_suspensions_print_only_after_root_completion() {
    let mut session = session_with_test_control_builtins();
    pending(start(&mut session), "first: ");
    assert!(session.prints().is_empty());
    pending(session.resume_suspended_evaluation("no"), "second: ");
    assert!(session.prints().is_empty());
    assert_eq!(session.resume_suspended_evaluation("yes").unwrap(), n(77));
    assert_eq!(session.prints(), ["2"]);
    assert!(session.pending_suspension_generation().is_none());
}
#[test]
fn pure_abandon_at_either_prompt_runs_no_missing_match() {
    for stop in [0, 1] {
        let mut session = session_with_test_control_builtins();
        session.eval_text("effect=0").unwrap();
        pending(start(&mut session), "first: ");
        if stop == 1 {
            pending(session.resume_suspended_evaluation("no"), "second: ");
        }
        session.abandon_pending_suspension(AbandonReason::CancelledRequest);
        assert!(session.prints().is_empty());
        assert!(session.pending_suspension_generation().is_none());
        assert_eq!(session.eval_text("effect").unwrap().value, n(0));
        assert_eq!(session.eval_text("1+1").unwrap().value, n(2));
    }
}
