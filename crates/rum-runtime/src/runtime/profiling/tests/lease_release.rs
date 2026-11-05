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

// The falsifier releases its retired charge at the drain, or after the next
// baseline if the drain is removed. A process-wide hook lets any intervening
// test acquire the lease without deadlocking on this deliberately held charge.
static DEFERRED_RELEASE: std::sync::Mutex<Option<std::sync::mpsc::Sender<()>>> =
    std::sync::Mutex::new(None);

pub(super) fn release_deferred_charge() {
    if let Some(release) = DEFERRED_RELEASE.lock().unwrap().take() {
        release.send(()).unwrap();
    }
}

#[test]
fn lease_acquisition_drains_a_prior_epoch_charge() {
    let previous = test_support::enabled();
    let storage = {
        let epoch = super::Epoch::for_test(previous.lease(), 1024 * 1024).unwrap();
        epoch.storage(32 * 1024).unwrap()
    };
    let (release, released) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        released.recv().unwrap();
        drop(storage);
    });
    *DEFERRED_RELEASE.lock().unwrap() = Some(release);
    drop(previous);

    let enabled = test_support::enabled();
    let baseline = budget::process_bytes();
    // Without the drain, the worker still owns at least 32 KiB at baseline.
    // Joining makes its release land before the unchanged comparison below.
    release_deferred_charge();
    worker.join().unwrap();
    let session = Session::new();
    Call::start(
        &session,
        AggregateKey::Runtime {
            phase: "lease-drain",
            source: None,
        },
        Policy::Expr,
    )
    .unwrap()
    .finish(Outcome::Ok);
    drop(session);
    assert!(budget::process_bytes() > baseline);
    drop(enabled);
}
