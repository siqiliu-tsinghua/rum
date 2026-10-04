//! Exercise every flat walk independently of descriptor Expr construction.
use super::{
    PullBudget,
    continuation::{LazyRealizeSuspensionFrame, working_copy},
    state::*,
};
use rum_core::{Expr, Rational, Symbol};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor, Session, system_expr};
use std::sync::Arc;

const DEPTH: usize = 2_000;
fn on_small_stack(body: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name("lazy-private-2mib".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(body)
        .unwrap()
        .join()
        .unwrap();
}

struct Roots(Vec<Expr>);
impl FrameRootVisitor for Roots {
    fn visit_expr(&mut self, expr: &Expr) {
        self.0.push(expr.clone());
    }
    fn visit_symbol(&mut self, _: &Symbol) {
        panic!("these fixtures have no bare Symbol fields");
    }
}
fn roots(value: &impl FrameRootTrace) -> Vec<Expr> {
    let mut visitor = Roots(Vec::new());
    value.trace_roots(&mut visitor);
    visitor.0
}

fn stages() -> (Vec<PullStage>, Vec<Expr>) {
    let mut stages = Vec::new();
    let mut expected = Vec::new();
    for index in 0..DEPTH {
        let root = Expr::integer((index as i64).into());
        stages.push(match index % 5 {
            0 => {
                expected.push(root.clone());
                PullStage::Map { function: root }
            }
            1 => {
                expected.push(root.clone());
                PullStage::Select { predicate: root }
            }
            2 => PullStage::Take { remaining: index },
            3 => PullStage::Drop { remaining: index },
            _ => {
                expected.push(root.clone());
                expected.push(Expr::integer((-(index as i64)).into()));
                PullStage::FoldList {
                    function: root,
                    accumulator: Expr::integer((-(index as i64)).into()),
                    emitted_init: true,
                }
            }
        });
    }
    (stages, expected)
}

fn flat_frame(source: PullSource, cursor: AwaitCursor) -> LazyRealizeSuspensionFrame {
    LazyRealizeSuspensionFrame {
        machine: PullMachine {
            source,
            stages: stages().0,
        },
        items: vec![Expr::string("prefix")].into(),
        budget: PullBudget::AtMost(7),
        limits: SavedPullLimits {
            iteration_limit: 11,
            source_pulls: 3,
        },
        cursor,
    }
}

fn range() -> PullSource {
    PullSource::Range {
        current: Rational::from_integer(0.into()),
        step: Rational::from_integer(1.into()),
    }
}

#[test]
fn lazy_flat_clone_and_working_copy_depth() {
    on_small_stack(|| {
        let frame = flat_frame(
            range(),
            AwaitCursor::Select {
                stage: 1,
                candidate: Expr::string("candidate"),
            },
        );
        let cloned = frame.machine.clone();
        assert_eq!(cloned.stages.len(), DEPTH);
        assert_eq!(roots(&cloned), roots(&frame.machine));
        let (working, prefix, cursor) = working_copy(&frame);
        assert_eq!(working.stages.len(), DEPTH);
        assert_eq!(roots(&working), roots(&frame.machine));
        assert_eq!(prefix.as_slice(), frame.items.as_slice());
        assert_eq!(roots(&cursor), roots(&frame.cursor));
        assert_eq!(frame.limits.source_pulls, 3);
        assert_eq!(frame.limits.iteration_limit, 11);
        println!("flat clone and working copy depth={DEPTH} complete");
    });
}

#[test]
fn lazy_flat_all_variant_roots_depth() {
    on_small_stack(|| {
        let sources = [
            (range(), vec![]),
            (
                PullSource::Permutations {
                    current: vec![Expr::string("perm")],
                    exhausted: false,
                },
                vec![Expr::string("perm")],
            ),
            (
                PullSource::Subsets {
                    elements: vec![Expr::string("subset")],
                    k: 0,
                    combo: vec![],
                },
                vec![Expr::string("subset")],
            ),
            (
                PullSource::Tuples {
                    inner_lists: vec![Expr::call(
                        system_expr("List"),
                        vec![Expr::string("factor")],
                    )],
                    digits: vec![0],
                    exhausted: false,
                },
                vec![Expr::call(
                    system_expr("List"),
                    vec![Expr::string("factor")],
                )],
            ),
            (
                PullSource::NestList {
                    function: Expr::string("nest_fn"),
                    current: Expr::string("current"),
                },
                vec![Expr::string("nest_fn"), Expr::string("current")],
            ),
        ];
        for (source, source_roots) in sources {
            for (cursor, cursor_roots) in [
                (AwaitCursor::Map { stage: 0 }, vec![]),
                (
                    AwaitCursor::Select {
                        stage: 1,
                        candidate: Expr::string("candidate"),
                    },
                    vec![Expr::string("candidate")],
                ),
                (AwaitCursor::Fold { stage: 4 }, vec![]),
                (
                    AwaitCursor::Nest {
                        yield_value: Expr::string("saved_yield"),
                    },
                    vec![Expr::string("saved_yield")],
                ),
            ] {
                let frame = flat_frame(source.clone(), cursor);
                let mut expected = source_roots.clone();
                expected.extend(stages().1);
                expected.push(Expr::string("prefix"));
                expected.extend(cursor_roots);
                assert_eq!(roots(&frame), expected);
            }
        }
        println!("all source/stage/cursor roots depth={DEPTH} complete");
    });
}

#[test]
fn lazy_flat_last_owner_drop_depth() {
    on_small_stack(|| {
        let frame = Arc::new(flat_frame(range(), AwaitCursor::Map { stage: 0 }));
        let other = frame.clone();
        drop(frame);
        assert_eq!(Arc::strong_count(&other), 1);
        drop(other);
        println!("last-owner flat drop depth={DEPTH} complete");
    });
}

#[test]
fn lazy_flat_abandonment_depth() {
    on_small_stack(|| {
        let mut session = Session::new();
        crate::install_core(&mut session);
        session.eval_request("log=0").unwrap();
        let failed = session.eval_request("InputString[\"p: \"]");
        assert!(failed.is_err());
        assert!(session.pending_input_request().is_some());
        // No descriptor exists in this Session: cancellation directly retires
        // the flat frame. Running its invalid function data would fail the test.
        rum_runtime::resume_or_suspend!(insert session, flat_frame(range(), AwaitCursor::Map { stage: 0 }));
        session.request_cancel_handle().cancel();
        let result = session.resume_eval_request_with_input("unused").unwrap();
        assert_eq!(format!("{:?}", result.termination), "Some(Cancelled)");
        assert!(session.pending_input_request().is_none());
        assert_eq!(session.eval_request("log").unwrap().value.to_string(), "0");
        assert_eq!(session.eval_request("1+1").unwrap().value.to_string(), "2");
        drop(session);
        println!("flat abandonment depth={DEPTH} complete");
    });
}
