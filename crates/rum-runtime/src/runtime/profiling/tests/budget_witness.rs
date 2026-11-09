//! A real foreign refund inside an owning Budget's observation window.
use super::Epoch;
use crate::runtime::profiling::lease::{self, TestLease};
use std::sync::{Arc, mpsc};
use std::time::Duration;

pub(in crate::runtime) struct ForeignRefund {
    bytes: usize,
    baseline: usize,
    foreign: Arc<Epoch>,
    release: Option<mpsc::Sender<()>>,
    completed: mpsc::Receiver<(usize, bool)>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl ForeignRefund {
    pub(in crate::runtime) fn charged(lease: &TestLease, bytes: usize) -> Self {
        let foreign = Epoch::for_test(lease, 1024 * 1024).unwrap();
        // The foreign epoch's own retained charge, captured from the object
        // rather than written as a measured literal: the window may move
        // nothing but the injected charge and its refund.
        let baseline = foreign.test_budget_bytes();
        let worker_epoch = Arc::clone(&foreign);
        let (ready, charged) = mpsc::channel();
        let (release, released) = mpsc::channel();
        let (done, completed) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            assert!(lease::blocks_unregistered_thread());
            let (access, registration) =
                lease::admit_generation(Some(worker_epoch.lease_generation())).unwrap();
            let before = worker_epoch.test_budget_bytes();
            let storage = worker_epoch.storage_admitted(&access, bytes).unwrap();
            assert_eq!(worker_epoch.test_budget_bytes() - before, bytes);
            drop(access);
            drop(registration);
            assert!(lease::blocks_unregistered_thread());
            ready.send(()).unwrap();
            released.recv_timeout(Duration::from_secs(10)).unwrap();
            let before = worker_epoch.test_budget_bytes();
            drop(storage);
            let refunded = before
                .checked_sub(worker_epoch.test_budget_bytes())
                .expect("foreign charges unexpectedly grew in the refund window");
            done.send((refunded, lease::blocks_unregistered_thread()))
                .unwrap();
        });
        let refund = Self {
            bytes,
            baseline,
            foreign,
            release: Some(release),
            completed,
            worker: Some(worker),
        };
        charged.recv_timeout(Duration::from_secs(10)).unwrap();
        refund
    }

    pub(in crate::runtime) fn refund_and_join(&mut self) {
        self.release.take().unwrap().send(()).unwrap();
        let (refunded, unregistered) = self
            .completed
            .recv_timeout(Duration::from_secs(10))
            .unwrap();
        self.worker.take().unwrap().join().unwrap();
        let remaining = self.foreign.test_budget_bytes();
        eprintln!(
            "B716 FOREIGN_REFUND charged={} baseline={} refunded={refunded} unregistered={unregistered} budget_after={remaining}",
            self.bytes, self.baseline,
        );
        assert_eq!(refunded, self.bytes);
        assert!(unregistered);
        assert_eq!(remaining, self.baseline);
    }
}

impl Drop for ForeignRefund {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
