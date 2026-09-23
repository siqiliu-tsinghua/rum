use super::*;

#[derive(rum_derive::FrameRootTrace)]
struct ResumePrefix;

impl RetentionContinuation for ResumePrefix {
    fn resume(
        &self,
        session: &mut Session,
        prefix: &FrozenValues,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        FrameAction::complete(super::super::retained::scoped::<AccountingShadow, _>(
            session,
            |mut cx, empty| {
                let mut values = cx.resume_values(empty, prefix)?;
                cx.push(&mut values, outcome?)?;
                cx.finish_list(values)
            },
        ))
    }
}

#[derive(Default)]
struct Roots(usize);
impl FrameRootVisitor for Roots {
    fn visit_expr(&mut self, _: &Expr) {
        self.0 += 1;
    }
    fn visit_symbol(&mut self, _: &Symbol) {}
}

#[test]
fn retention_shared_frame_keeps_roots_and_lease_until_last_owner_drops() {
    let mut session = Session::new();
    session.current_memory_limit = Some(1_000_000_000);
    session.note_accounting_arm();
    let mut values = vec![Expr::string("shared frame payload".repeat(1000))];
    values.reserve_exact(7);
    let spare_slots = values.capacity() - values.len();
    let complement = spare_slots * size_of::<Expr>()
        + size_of::<Mutex<RetentionLease>>()
        + 2 * size_of::<usize>();
    let mut lease = RetentionLease::new(&mut session);
    lease.resize(Some(complement));
    let frame = Arc::new(RetentionFrame {
        prefix: FrozenValues {
            values,
            complement: FrozenComplement {
                spare_slots,
                receipt: Some(Arc::new(Mutex::new(lease))),
            },
        },
        continuation: ResumePrefix,
    });
    let mut roots = Roots::default();
    frame.trace_roots(&mut roots);
    assert_eq!(roots.0, 1, "frozen payload must be a traced frame root");
    let shared = frame.clone();
    assert!(matches!(
        frame.resume(&mut session, Ok(Expr::small_integer(7))),
        FrameAction::Complete(Ok(_))
    ));
    assert_eq!(
        session.retention_snapshot().unwrap().bytes,
        Some(complement)
    );
    assert_eq!(session.retention_snapshot().unwrap().live, 1);
    let extra_owner = shared.clone();
    assert!(matches!(
        shared.resume(&mut session, Err(EvalError::Abort)),
        FrameAction::Complete(Err(EvalError::Abort))
    ));
    assert_eq!(
        session.retention_snapshot().unwrap().bytes,
        Some(complement)
    );
    let mut roots = Roots::default();
    extra_owner.trace_roots(&mut roots);
    assert_eq!(roots.0, 1);
    drop(extra_owner);
    assert_eq!(session.retention_snapshot().unwrap().bytes, Some(0));
    assert_eq!(session.retention_snapshot().unwrap().live, 0);
    println!(
        "shared frame: root preserved on success/error, complement {complement} released at last drop"
    );
}
