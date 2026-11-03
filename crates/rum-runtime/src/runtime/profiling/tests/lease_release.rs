use super::{AggregateKey, Call, Outcome, Policy, budget, test_support};
use crate::runtime::Session;

#[test]
fn logical_lease_drop_balances_thread_registration() {
    for _ in 0..2 {
        let enabled = test_support::enabled();
        let baseline = budget::process_bytes();
        let session = Session::new();
        Call::start(
            &session,
            AggregateKey::Runtime {
                phase: "lease-release",
                source: None,
            },
            Policy::Expr,
        )
        .unwrap()
        .finish(Outcome::Ok);
        drop(session);
        assert_eq!(test_support::active(), 0);
        assert!(budget::process_bytes() > baseline);
        drop(enabled);

        let _lease = test_support::lease();
        assert_eq!(
            budget::process_bytes(),
            baseline,
            "lease drop must balance this thread's diagnostic registration"
        );
    }
}
