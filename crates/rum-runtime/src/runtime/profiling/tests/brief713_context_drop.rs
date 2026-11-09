use super::*;

#[test]
fn brief713_context_retired_restore_must_not_readmit_on_drop() {
    let lease = crate::runtime::profiling::lease::lease();
    let epoch = Epoch::for_test(&lease, 1024 * 1024).unwrap();
    let control = Arc::new(crate::runtime::RequestInterruptState::new());
    let id = epoch
        .open(
            Owner {
                control: Arc::clone(&control),
                generation: 1,
                root: 1,
            },
            None,
            AggregateKey::Runtime {
                phase: "brief713-context-drop",
                source: None,
            },
            Policy::Expr,
        )
        .unwrap();
    // Keep the TLS header and one spare slot available: restore needs no claim.
    let native = segments::enter(&epoch, id).unwrap();
    let context = Context::capture(&epoch).unwrap();
    let before = segments::active_count();
    let path = {
        let data = epoch.data.lock().unwrap();
        Arc::clone(&data.records.get(id).unwrap().path)
    };
    let (result, invalid, open, after) = std::thread::scope(|scope| {
        let (pin_ready, wait_ready) = std::sync::mpsc::channel();
        let (release_pin, wait_release) = std::sync::mpsc::channel::<()>();
        let pin_epoch = Arc::clone(&epoch);
        let pin_worker = scope.spawn(move || {
            let data = pin_epoch.data.lock().unwrap();
            pin_ready
                .send(())
                .expect("data pin readiness receiver dropped");
            let released = wait_release.recv_timeout(std::time::Duration::from_secs(30));
            drop(data);
            assert!(
                matches!(
                    released,
                    Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected)
                ),
                "data pin release timed out: {released:?}"
            );
        });
        wait_ready
            .recv_timeout(std::time::Duration::from_secs(30))
            .expect("data pin worker did not acquire the mutex");
        let worker = scope.spawn(|| {
            lease.allow_current_thread();
            epoch.retire(Retire::ProfileReset);
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while epoch.is_valid() {
            assert!(
                std::time::Instant::now() < deadline,
                "retire did not reach validity store"
            );
            std::thread::yield_now();
        }
        let invalid = !epoch.is_valid();
        let open = path.is_open();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            context.enter_with_control(Some(Arc::clone(&control)))
        }));
        let after = segments::active_count();
        // Dropping the sole sender also releases the pin on every unwinding exit.
        drop(release_pin);
        pin_worker.join().unwrap();
        worker.join().unwrap();
        (result, invalid, open, after)
    });
    let no_nested_admission = result.is_ok();
    eprintln!(
        "B713 CONTEXT_DROP invalid={invalid} path_open={open} before={before} after={after} no_nested_admission={no_nested_admission}"
    );
    drop(result);
    drop(native);
    drop(context);
    assert!(invalid && open);
    assert!(
        no_nested_admission,
        "retired Context return must release its restored Segment without a second admission"
    );
    assert_eq!(after, before);
}
