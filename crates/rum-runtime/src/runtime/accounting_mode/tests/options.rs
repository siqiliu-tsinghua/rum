use super::*;
use crate::runtime::debug_tools::resources::occurrence::expression;
use crate::runtime::options::{OptionReentry, OptionReentryDeclaration};
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

fn session(armed: bool) -> Session {
    let mut session = Session::new();
    session.install_option_reentry_domain(DOMAIN);
    if armed {
        session.current_memory_limit = Some(1_000_000_000);
        session.note_accounting_arm();
    }
    session
}

fn defaults(session: &mut Session) -> Symbol {
    let symbol = Symbol::system("OptionOwner544");
    let rules = Expr::call(
        system_expr("List"),
        vec![Expr::call(
            system_expr("Rule"),
            vec![
                system_expr("Payload544"),
                Expr::string("default".repeat(10_000)),
            ],
        )],
    );
    let options = session
        .compile_assigned_options_from_resolved(&rules)
        .unwrap();
    session.set_options_for_symbol(symbol.clone(), options);
    symbol
}

fn released(session: &Session) {
    assert!(
        session
            .accounting_route
            .as_ref()
            .unwrap()
            .prefixes
            .is_empty()
    );
    if let Some(snapshot) = session.retention_snapshot() {
        assert_eq!(snapshot.bytes, Some(0));
        assert_eq!(snapshot.live, 0);
    }
}

#[test]
fn option_owner_snapshot_charges_defaults_and_copies() {
    let mut session = session(true);
    let symbol = defaults(&mut session);
    let state = snapshot::<AccountingShadow>(&mut session, &symbol).unwrap();
    assert_eq!(state.defaults, 1);
    let payload = expression(&state.values[0]).ceiling_bytes();
    let expected = payload
        + (state.values.capacity() - 1) * size_of::<Expr>()
        + size_of::<Vec<Expr>>()
        + 2 * size_of::<usize>();
    assert_eq!(
        session.retention_snapshot().unwrap().bytes,
        Some(expected),
        "snapshot charge must include the copied default occurrence"
    );
    state.guest(&mut session, |session| {
        let base = session.current_memory_proxy_bytes_for_test();
        session.current_memory_limit = Some(base + 1000);
        assert!(
            matches!(
                session.enforce_active_memory_limit_for_projected_bytes(0),
                Err(EvalError::MemoryConstraintExceeded { .. })
            ),
            "default snapshot must remain charged at the guest checkpoint"
        );
    });
    drop(state);
    released(&session);
}

#[test]
fn option_owner_capacity_and_callback_adoption() {
    let mut session = session(true);
    let mut state =
        snapshot::<AccountingShadow>(&mut session, &Symbol::system("NoDefaults544")).unwrap();
    let base = session.current_memory_proxy_bytes_for_test();
    session.current_memory_limit = Some(base + SHARED_BYTES + 1000);
    assert!(matches!(
        state.adopt(&mut session, Expr::string("incoming".repeat(10_000))),
        Err(EvalError::MemoryConstraintExceeded { .. })
    ));
    assert_eq!(
        state.values.capacity(),
        0,
        "callback must acquire its charge before reserve"
    );
    session.current_memory_limit = Some(1_000_000_000);
    for _ in 0..5 {
        state.adopt(&mut session, Expr::small_integer(4)).unwrap();
        let expected = state
            .values
            .iter()
            .map(|v| expression(v).ceiling_bytes())
            .sum::<usize>()
            + (state.values.capacity() - state.values.len()) * size_of::<Expr>()
            + SHARED_BYTES;
        assert_eq!(
            session.retention_snapshot().unwrap().bytes,
            Some(expected),
            "prefix and actual spare capacity must both be charged"
        );
    }
    assert!(state.values.capacity() > state.values.len());
    drop(state);
    released(&session);
}

#[derive(Clone, Copy, rum_derive::FrameRootTrace)]
struct CheckHandoff;
const DOMAIN: &[OptionReentryDeclaration] = &[OptionReentryDeclaration::of::<CheckHandoff>(
    "test::CheckHandoff544",
)];

impl OptionReentry for CheckHandoff {
    fn symbol_name(self) -> &'static str {
        "OptionOwner544"
    }
    fn resume(
        self,
        session: &mut Session,
        _: &Expr,
        _: &[Expr],
        options: RetainedOptions<'_>,
    ) -> Result<Expr, EvalError> {
        assert_eq!(options.len(), 1);
        let before = session.current_memory_proxy_bytes_for_test();
        session.current_memory_limit = Some(before + 1000);
        session.note_accounting_arm();
        assert!(
            matches!(
                session.enforce_active_memory_limit_for_projected_bytes(0),
                Err(EvalError::MemoryConstraintExceeded { .. })
            ),
            "handoff owner must be visible when the consumer arms a limit"
        );
        Ok(system_expr("True"))
    }
}

#[test]
fn option_owner_handoff_and_late_arm_keep_original_visible() {
    for armed in [false, true] {
        let mut session = session(armed);
        defaults(&mut session);
        let result = session
            .merged_option_exprs_for_system_symbol(CheckHandoff, &system_expr("List"), &[], &[])
            .unwrap();
        assert_eq!(result, system_expr("True"));
        released(&session);
    }
}

#[test]
fn option_owner_guest_error_and_unwind_release() {
    for armed in [false, true] {
        let mut session = session(armed);
        let symbol = defaults(&mut session);
        let run = |session: &mut Session| {
            if armed {
                let state = snapshot::<AccountingShadow>(session, &symbol).unwrap();
                state.guest(session, |_| panic!("option owner unwind544"));
            } else {
                let state = snapshot::<Unarmed>(session, &symbol).unwrap();
                state.guest(session, |_| panic!("option owner unwind544"));
            }
        };
        assert!(catch_unwind(AssertUnwindSafe(|| run(&mut session))).is_err());
        released(&session);
        let state = snapshot::<Unarmed>(&mut session, &symbol).unwrap();
        let error = state.guest(&mut session, |_| {
            Err::<(), _>(EvalError::Return {
                value: Expr::string("error544"),
            })
        });
        assert!(
            matches!(error, Err(EvalError::Return { value }) if value == Expr::string("error544"))
        );
        drop(state);
        released(&session);
    }
}
