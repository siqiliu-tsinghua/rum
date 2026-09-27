use super::*;
use crate::runtime::options::{
    MergedOptionsSuspensionFrame, OptionReentry, OptionReentryDeclaration,
};

#[derive(Clone, Copy, rum_derive::FrameRootTrace)]
struct ListConsumer;

const LIST_DOMAIN: &[OptionReentryDeclaration] = &[OptionReentryDeclaration::of::<ListConsumer>(
    "test::ListConsumer",
)];
const HOST_DOMAIN: &[OptionReentryDeclaration] = &[OptionReentryDeclaration::of::<ListConsumer>(
    "external::claimed",
)];

impl OptionReentry for ListConsumer {
    fn symbol_name(self) -> &'static str {
        "TestOptions"
    }
    fn resume(
        self,
        _session: &mut Session,
        head: &Expr,
        _args: &[Expr],
        options: RetainedOptions<'_>,
    ) -> Result<Expr, EvalError> {
        Ok(Expr::call(head.clone(), options.to_vec()))
    }
}

#[derive(Default)]
struct Roots {
    exprs: Vec<Expr>,
    symbols: Vec<Symbol>,
}

impl FrameRootVisitor for Roots {
    fn visit_expr(&mut self, expr: &Expr) {
        self.exprs.push(expr.clone());
    }

    fn visit_symbol(&mut self, symbol: &Symbol) {
        self.symbols.push(symbol.clone());
    }
}

#[test]
fn option_loop_frame_traces_inputs_and_accumulated_prefix() {
    let frame = RetentionFrame {
        prefix: FrozenValues {
            values: vec![Expr::symbol("default"), Expr::symbol("prefix")],
            complement: FrozenComplement {
                spare_slots: 0,
                receipt: None,
            },
        },
        continuation: MergedOptionsSuspensionFrame {
            domain_generation: 1,
            reentry: ListConsumer,
            head: system_expr("List"),
            args: vec![Expr::symbol("argument")].into(),
            target: Symbol::system("StringReplace"),
            defaults: 1,
            explicit: vec![Expr::symbol("explicit")].into(),
            next_index: 1,
        },
    };
    let mut roots = Roots::default();
    frame.trace_roots(&mut roots);
    assert_eq!(roots.symbols, vec![frame.continuation.target]);
    assert_eq!(
        roots.exprs,
        vec![
            Expr::symbol("default"),
            Expr::symbol("prefix"),
            system_expr("List"),
            Expr::symbol("argument"),
            Expr::symbol("explicit"),
        ]
    );
}

#[test]
fn option_loop_frame_resumes_with_another_arc_owner() {
    let frame = Arc::new(RetentionFrame {
        prefix: FrozenValues {
            values: vec![Expr::symbol("already_evaluated"), Expr::symbol("prefix")],
            complement: FrozenComplement {
                spare_slots: 0,
                receipt: None,
            },
        },
        continuation: MergedOptionsSuspensionFrame {
            domain_generation: 1,
            reentry: ListConsumer,
            head: system_expr("List"),
            args: vec![Expr::symbol("argument")].into(),
            target: Symbol::system("StringReplace"),
            defaults: 1,
            explicit: vec![Expr::symbol("pending"), Expr::symbol("remaining")].into(),
            next_index: 2,
        },
    });
    let retained = frame.clone();
    let mut session = Session::new();
    let FrameAction::Complete(Ok(refused)) = frame
        .clone()
        .resume(&mut session, Ok(Expr::symbol("resolved")))
    else {
        panic!("unregistered resume must refuse");
    };
    assert_eq!(refused, system_expr("$Failed"));
    assert_eq!(session.messages().last().unwrap().tag, "optreentry");
    session.install_option_reentry_domain(LIST_DOMAIN);
    let FrameAction::Complete(Ok(value)) = frame.resume(&mut session, Ok(Expr::symbol("resolved")))
    else {
        panic!("option loop should complete without requiring unique ownership");
    };
    assert_eq!(
        value,
        Expr::call(
            system_expr("List"),
            vec![
                Expr::symbol("prefix"),
                Expr::symbol("resolved"),
                Expr::symbol("remaining")
            ]
        )
    );
    assert_eq!(&retained.prefix.values[1..], &[Expr::symbol("prefix")]);
    session.install_option_reentry_domain(LIST_DOMAIN);
    let FrameAction::Complete(Ok(refused)) =
        retained.resume(&mut session, Ok(Expr::symbol("late")))
    else {
        panic!("changed domain must refuse the restored frame");
    };
    assert_eq!(refused, system_expr("$Failed"));
}

#[test]
fn option_working_copy_checkpoint_and_shared_frame_release() {
    let mut session = Session::new();
    session.install_option_reentry_domain(LIST_DOMAIN);
    session.current_memory_limit = Some(1_000_000_000);
    session.note_accounting_arm();
    let mut values = Vec::with_capacity(8);
    values.push(Expr::string("frozen544".repeat(10_000)));
    values.push(Expr::rational(
        Rational::new(BigInt::from(7), BigInt::from(13)).unwrap(),
    ));
    values.push(Expr::call(system_expr("held544"), vec![values[0].clone()]));
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
        continuation: MergedOptionsSuspensionFrame {
            domain_generation: session.option_domain_generation(),
            reentry: ListConsumer,
            head: system_expr("List"),
            args: vec![].into(),
            target: Symbol::system("TestOptions"),
            defaults: 0,
            explicit: vec![].into(),
            next_index: 0,
        },
    });
    let shared = frame.clone();
    let empty = RetainedValues {
        values: vec![],
        receipt: AccountingShadow::receipt(&mut session),
        brand: std::marker::PhantomData,
    };
    let mut cx = RetentionContext::<AccountingShadow> {
        session: &mut session,
        brand: std::marker::PhantomData,
        mode: std::marker::PhantomData,
    };
    let working = cx.resume_values(empty, &frame.prefix).unwrap();
    let expected = values_bytes(&working.values).unwrap() + complement;
    assert_eq!(
        cx.session.retention_snapshot().unwrap().bytes,
        Some(expected),
        "working copy and still-shared frozen complement must coexist"
    );
    let base = cx.session.current_memory_proxy_bytes_for_test();
    cx.session.current_memory_limit = Some(base + complement + 1000);
    assert!(
        cx.session
            .enforce_active_memory_limit_for_projected_bytes(0)
            .is_err(),
        "working copy must be visible at the resume checkpoint"
    );
    drop(working);
    session.current_memory_limit = Some(1_000_000_000);
    assert!(matches!(
        frame
            .clone()
            .resume(&mut session, Ok(Expr::small_integer(1))),
        FrameAction::Complete(Ok(_))
    ));
    assert_eq!(
        session.retention_snapshot().unwrap().bytes,
        Some(complement)
    );
    assert!(
        matches!(frame.resume(&mut session, Err(EvalError::Return { value: Expr::string("error544") })),
        FrameAction::Complete(Err(EvalError::Return { value })) if value == Expr::string("error544"))
    );
    assert_eq!(session.retention_snapshot().unwrap().live, 1);
    drop(shared);
    assert_eq!(session.retention_snapshot().unwrap().bytes, Some(0));
    assert_eq!(session.retention_snapshot().unwrap().live, 0);
}

fn loop_guest(session: &mut Session, head: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    session.merged_option_exprs_for_system_symbol(ListConsumer, head, args, args)
}

#[test]
fn option_frozen_complement_repeated_resume_and_abandon() {
    for armed in [false, true] {
        let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
        session.install_option_reentry_domain(LIST_DOMAIN);
        session.register_builtin(BuiltinSpec::ordinary_with_hold(
            "Loop544",
            "retained option frame control",
            HoldAttribute::HoldAll,
            loop_guest,
            MemoryGrowthContract::Incremental,
        ));
        if armed {
            session.resource_limits.eval_memory_limit = Some(1_000_000_000);
        }
        let source = "Loop544[1, 2, 3, 4, 5, name :> (InputString[\"first544\"]; InputString[\"second544\"])]";
        session.eval_request(source).unwrap_err();
        let control = size_of::<Mutex<RetentionLease>>() + 2 * size_of::<usize>();
        if armed {
            let snapshot = session.retention_snapshot().unwrap();
            assert_eq!(snapshot.transfers, 2);
            assert_eq!(
                snapshot.bytes,
                Some(2 * control + 3 * size_of::<Expr>()),
                "option freeze must preserve spare capacity and receipt controls"
            );
        } else {
            session.current_memory_limit = Some(1_000_000_000);
            session.note_accounting_arm();
            assert_eq!(
                session.retained_admission_bytes(),
                Some(3 * size_of::<Expr>()),
                "late arm must see frozen spare capacity"
            );
            session.current_memory_limit = None;
        }
        session.resume_eval_request_with_input("one").unwrap_err();
        if armed {
            assert_eq!(session.retention_snapshot().unwrap().transfers, 2);
        }
        let outcome = session.resume_eval_request_with_input("two").unwrap();
        assert_eq!(
            outcome.value.to_string(),
            "Loop544[1, 2, 3, 4, 5, Rule[name, \"two\"]]"
        );
        assert!(
            session
                .retention_snapshot()
                .is_none_or(|s| s.live == 0 && s.bytes == Some(0))
        );
        session.eval_request(source).unwrap_err();
        session.abandon_pending_suspension(
            crate::runtime::suspension::AbandonReason::CancelledRequest,
        );
        assert!(
            session
                .retention_snapshot()
                .is_none_or(|s| s.live == 0 && s.bytes == Some(0))
        );
    }
}

#[test]
fn option_reentry_registration_does_not_mint_certification() {
    let mut session = Session::new();
    let head = system_expr("List");
    let run = |session: &mut Session| {
        session
            .merged_option_exprs_for_system_symbol(
                ListConsumer,
                &head,
                &[],
                &[Expr::small_integer(17)],
            )
            .unwrap()
    };
    assert_eq!(run(&mut session), system_expr("$Failed"));
    assert_eq!(session.messages().last().unwrap().tag, "optreentry");
    session.register_option_reentry::<ListConsumer>();
    session.register_option_reentry::<ListConsumer>();
    assert_eq!(
        session.option_reentry_registration::<ListConsumer>(),
        (true, None)
    );
    assert_eq!(run(&mut session), system_expr("$Failed"));
    // The public installer seam can deliberately assert trust. This is the
    // executable boundary of the repository-build certification guarantee.
    session.install_option_reentry_domain(HOST_DOMAIN);
    assert_eq!(
        run(&mut session),
        Expr::call(head, vec![Expr::small_integer(17)])
    );
    println!(
        "plain public registration => $Failed; public installer can assert trust, outside repository certificates => List[17]"
    );
}

#[test]
fn option_reentry_domain_replacement_expires_resume() {
    let mut session = Session::new();
    const DECLARATION: &[OptionReentryDeclaration] =
        &[OptionReentryDeclaration::of::<ListConsumer>(
            "test::ListConsumer",
        )];
    session.install_option_reentry_domain(DECLARATION);
    let old = session.option_domain_generation();
    assert!(session.check_option_resume::<ListConsumer>(old));
    session.install_option_reentry_domain(DECLARATION);
    assert!(!session.check_option_resume::<ListConsumer>(old));
    session.install_option_reentry_domain(&[]);
    assert!(!session.check_option_resume::<ListConsumer>(session.option_domain_generation()));
}
