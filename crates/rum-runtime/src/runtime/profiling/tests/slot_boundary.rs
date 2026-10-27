use crate::runtime::Session;
use crate::runtime::profiling::logical::{AggregateKey, Call, Outcome, Policy, test_support};
use crate::runtime::suspension::{ContinuationState, FrameStorage, Slot};
use rum_core::Expr;

#[test]
fn logical_marker_preserves_slot_identity_and_rejects_wrong_channels() {
    let _enabled = test_support::enabled();
    let session = Session::new();
    let call = Call::start(
        &session,
        AggregateKey::Runtime {
            phase: "identity",
            source: None,
        },
        Policy::Expr,
    )
    .unwrap();
    let boundary = call.pause().unwrap();
    let value = session.lower_text("f[1,2,3]").unwrap();
    let Expr::Call { args: original, .. } = &value else {
        panic!()
    };
    let original = original.clone();
    let result = Ok(Slot::Value(value));
    let address = &result as *const _;
    boundary.observe(&result);
    boundary.observe(&result);
    assert_eq!(&result as *const _, address);
    let Ok(Slot::Value(Expr::Call { args, .. })) = &result else {
        panic!()
    };
    assert!(rum_core::SharedArgs::ptr_eq(args, &original));
    let counts = test_support::epoch().rows()[0].1;
    assert_eq!((counts.calls, counts.ok, counts.open), (1, 1, 0));
    assert_eq!(FrameStorage::entry_sizes(), (24, 16));
    Session::logical_profile_test_assert_quiescent();

    let call = Call::start(
        &session,
        AggregateKey::Runtime {
            phase: "wrong-channel",
            source: None,
        },
        Policy::Bool,
    )
    .unwrap();
    let boundary = call.pause().unwrap();
    boundary.observe(&result);
    assert!(!test_support::epoch().is_valid());
    assert!(rum_core::SharedArgs::ptr_eq(args, &original));
    assert_eq!(test_support::active(), 0);
}

#[test]
fn logical_marker_observes_closed_channels_without_codecs() {
    for (policy, slot, subset) in [
        (
            Policy::OptionalExpr,
            Slot::State(ContinuationState::OptionalExpr(Some(Expr::integer(
                3.into(),
            )))),
            1,
        ),
        (
            Policy::OptionalExpr,
            Slot::State(ContinuationState::OptionalExpr(None)),
            2,
        ),
        (Policy::Bool, Slot::State(ContinuationState::Bool(false)), 0),
        (
            Policy::ApplicableExpr,
            Slot::Value(Expr::integer(4.into())),
            1,
        ),
        (
            Policy::Incoming(Outcome::Throw),
            Slot::Value(Expr::integer(5.into())),
            3,
        ),
    ] {
        let _enabled = test_support::enabled();
        let session = Session::new();
        let call = Call::start(
            &session,
            AggregateKey::Runtime {
                phase: "channel",
                source: None,
            },
            policy,
        )
        .unwrap();
        let boundary = call.pause().unwrap();
        let result = Ok(slot);
        boundary.observe(&result);
        let c = test_support::epoch().rows()[0].1;
        assert_eq!(c.calls, 1);
        assert_eq!(c.open, 0);
        match subset {
            1 => assert_eq!(c.some, 1),
            2 => assert_eq!(c.none, 1),
            3 => assert_eq!((c.err, c.throw_err), (1, 1)),
            _ => assert_eq!(c.ok, 1),
        }
        Session::logical_profile_test_assert_quiescent();
    }
}
