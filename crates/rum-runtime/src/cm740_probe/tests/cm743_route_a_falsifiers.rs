use super::*;
use std::sync::Arc;

fn reset_clean() {
    LEDGER.with(|cell| {
        *cell.borrow_mut() = Ledger::default();
    });
    reset_operation_episode();
    identity_proofs::retire_namespace(|_| true);
}

fn begin_with_row(queued: bool, row: &str, program: &str) {
    LEDGER.with(|cell| {
        let mut l = cell.borrow_mut();
        assert!(l.stack.is_empty());
        retire_actual_run(&mut l);
        l.current = None;
        l.profile.reset();
        l.bound_slots.clear();
        DIRECT_CLAIMS.with(|c| c.borrow_mut().clear());
        l.deferred.clear();
        l.deferred_proofs.clear();
        let key = (row.to_owned(), program.to_owned());
        let pair = if queued {
            take_pair_once(&mut l, &key)
        } else {
            retire_pair_mailbox(&mut l, &key);
            l.queued_frame_issues.retain(|(owner, _), _| {
                (&owner.episode.row, &owner.episode.program) != (&key.0, &key.1)
            });
            l.queued_descendant_issues.retain(|(owner, _), _| {
                (&owner.episode.row, &owner.episode.program) != (&key.0, &key.1)
            });
            l.next_pair = l.next_pair.checked_add(1).unwrap();
            l.pending.remove(&key);
            Pair {
                episode: Some(Episode {
                    row: row.to_owned(),
                    program: program.to_owned(),
                    pair: l.next_pair,
                }),
                ..Pair::default()
            }
        };
        if !queued {
            assert!(
                l.pair_stages
                    .insert(pair.episode.clone().unwrap(), PairStage::Open)
                    .is_none()
            );
        }
        let source_authority = std::rc::Rc::new(RunAuthority {
            episode: pair.episode.clone(),
            queued,
            current: std::cell::Cell::new(true),
        });
        assert!(l.run_authority.replace(source_authority.clone()).is_none());
        l.current = Some(Run {
            source_authority,
            pair,
            queued,
            next_child: BTreeMap::new(),
            hits: BTreeMap::new(),
            pending: BTreeMap::new(),
            current_pending_sources: BTreeMap::new(),
            returning: BTreeMap::new(),
            request_returning: BTreeMap::new(),
        });
    });
}

fn check_gate(row: &str, guard: &Guard) -> Result<DescendantEdge, &'static str> {
    LEDGER.with(|cell| descendant_gate_clause(&cell.borrow(), row, guard))
}

#[test]
fn route_a_positive_and_all_falsifiers() {
    // 1. Positive path: unqueued wrapper W -> strict descendant D -> pause -> real frame
    //    construction -> queued replay at exact activation W -> ReturnOk, while
    //    ordinary()/frame()/eligible() stay FALSE.
    reset_clean();
    begin_with_row(false, "P0256", "prog-pos");
    {
        let g_w = enter_slow("P0256");
        // Pre-arm on unqueued half must fail.
        assert_eq!(check_gate("P0256", &g_w), Err("run_not_queued"));
        {
            let _g_d = enter_slow("P0489");
            arm("P0256", "producer-InputRequired", "p:1");
            let dummy_frame = Arc::new(42_u64);
            record_descendant_frame(&dummy_frame, "DummyFrame", 7);
        }
        // Still unqueued -> must not fire.
        assert_eq!(check_gate("P0256", &g_w), Err("run_not_queued"));
    }
    end();

    begin_with_row(true, "P0256", "prog-pos");
    let action =
        around_ok_profile_return::<i32, &str>("P0256", "OrdinaryActualSuccessBeforeTail", || {
            // Existing gates remain untouched and FALSE!
            assert!(!ordinary("P0256"));
            assert!(!frame("P0256"));
            Ok(99)
        });
    assert!(matches!(action, OkAction::ReturnOk(99)));
    // Second sibling activation has a different path -> must NOT fire.
    let action2 =
        around_ok_profile_return::<i32, &str>("P0256", "OrdinaryActualSuccessBeforeTail", || {
            Ok(100)
        });
    assert!(matches!(action2, OkAction::Continue(Ok(100))));
    end();
    println!("CM743_FALSIFIER PASS case=positive_and_sibling_isolation");

    // 2. Err body on queued half never early-returns.
    reset_clean();
    begin_with_row(false, "P0256", "prog-err");
    {
        let _g_w = enter_slow("P0256");
        let _g_d = enter_slow("P0489");
        arm("P0256", "producer-InputRequired", "p:1");
        let f = Arc::new(1_u8);
        record_descendant_frame(&f, "F", 1);
    }
    end();
    begin_with_row(true, "P0256", "prog-err");
    let action =
        around_ok_profile_return::<i32, &str>("P0256", "OrdinaryActualSuccessBeforeTail", || {
            Err("boom")
        });
    assert!(matches!(action, OkAction::Continue(Err("boom"))));
    end();
    println!("CM743_FALSIFIER PASS case=err_result_never_early_returns");

    // 3. Probe without construction (pause armed, NO record_descendant_frame).
    reset_clean();
    begin_with_row(false, "P0256", "prog-no-frame");
    {
        let _g_w = enter_slow("P0256");
        let _g_d = enter_slow("P0489");
        arm("P0256", "producer-InputRequired", "p:1");
    }
    end();
    begin_with_row(true, "P0256", "prog-no-frame");
    {
        let g_w = enter_slow("P0256");
        assert_eq!(check_gate("P0256", &g_w), Err("descendant_permit_missing"));
    }
    end();
    println!("CM743_FALSIFIER PASS case=probe_without_construction_rejected");

    // 4. Construction at W itself (not a strict descendant) is rejected.
    reset_clean();
    begin_with_row(false, "P0256", "prog-same-level");
    {
        let _g_w = enter_slow("P0256");
        arm("P0256", "producer-InputRequired", "p:1");
        let f = Arc::new(1_u8);
        record_descendant_frame(&f, "SelfFrame", 1);
    }
    end();
    begin_with_row(true, "P0256", "prog-same-level");
    {
        let g_w = enter_slow("P0256");
        assert_eq!(check_gate("P0256", &g_w), Err("descendant_permit_missing"));
    }
    end();
    println!("CM743_FALSIFIER PASS case=non_strict_self_construction_rejected");

    // 5. Generation == 0 (uninstalled suspension slot) rejected.
    reset_clean();
    begin_with_row(false, "P0256", "prog-gen0");
    {
        let _g_w = enter_slow("P0256");
        let _g_d = enter_slow("P0489");
        arm("P0256", "producer-InputRequired", "p:1");
        let f = Arc::new(1_u8);
        record_descendant_frame(&f, "GenZeroFrame", 0);
    }
    end();
    begin_with_row(true, "P0256", "prog-gen0");
    {
        let g_w = enter_slow("P0256");
        assert_eq!(check_gate("P0256", &g_w), Err("descendant_permit_missing"));
    }
    end();
    println!("CM743_FALSIFIER PASS case=zero_generation_rejected");

    // 6. Control source kind rejected (only ProducerPause | OwnedReturn admitted).
    reset_clean();
    begin_with_row(false, "P0256", "prog-ctrl");
    {
        let _g_w = enter_slow("P0256");
        let _g_d = enter_slow("P0489");
        arm("P0256", "synthetic-control-phase", "p:1");
        let f = Arc::new(1_u8);
        record_descendant_frame(&f, "CtrlFrame", 2);
    }
    end();
    begin_with_row(true, "P0256", "prog-ctrl");
    {
        let g_w = enter_slow("P0256");
        assert_eq!(check_gate("P0256", &g_w), Err("descendant_permit_missing"));
    }
    end();
    println!("CM743_FALSIFIER PASS case=control_source_kind_rejected");

    // 7. Construction across a ResumeGuard (None arming boundary) rejected.
    reset_clean();
    begin_with_row(false, "P0256", "prog-resume-boundary");
    {
        let _g_w = enter_slow("P0256");
        let _r = resume_frame(0xdead, "UnboundFrame");
        let _g_d = enter_slow("P0489");
        arm("P0256", "producer-InputRequired", "p:1");
        let f = Arc::new(1_u8);
        record_descendant_frame(&f, "CrossBoundaryFrame", 3);
    }
    end();
    begin_with_row(true, "P0256", "prog-resume-boundary");
    {
        let g_w = enter_slow("P0256");
        assert_eq!(check_gate("P0256", &g_w), Err("descendant_permit_missing"));
    }
    end();
    println!("CM743_FALSIFIER PASS case=resume_boundary_blocks_construction");

    // 8. Construction outside the pair (after W drops, or before begin / after end).
    reset_clean();
    let orphan_frame = Arc::new(7_u8);
    record_descendant_frame(&orphan_frame, "BeforeBegin", 1);
    begin_with_row(false, "P0256", "prog-outside-pair");
    {
        let _g_w = enter_slow("P0256");
        arm("P0256", "producer-InputRequired", "p:1");
    }
    // W has dropped; constructing a frame now is outside W's guard.
    {
        let _g_d = enter_slow("P0489");
        record_descendant_frame(&orphan_frame, "AfterWDrop", 4);
    }
    end();
    record_descendant_frame(&orphan_frame, "AfterEnd", 5);
    begin_with_row(true, "P0256", "prog-outside-pair");
    {
        let g_w = enter_slow("P0256");
        assert_eq!(check_gate("P0256", &g_w), Err("descendant_permit_missing"));
    }
    end();
    println!("CM743_FALSIFIER PASS case=construction_outside_pair_rejected");

    // 9. Wrong pair / replay reuse (queued half consumes pair ONCE).
    reset_clean();
    begin_with_row(false, "P0256", "prog-once");
    {
        let _g_w = enter_slow("P0256");
        let _g_d = enter_slow("P0489");
        arm("P0256", "producer-InputRequired", "p:1");
        let f = Arc::new(1_u8);
        record_descendant_frame(&f, "OnceFrame", 1);
    }
    end();
    begin_with_row(true, "P0256", "prog-once");
    {
        let g_w = enter_slow("P0256");
        assert!(check_gate("P0256", &g_w).is_ok());
    }
    end();
    // Second queued begin with no fresh unqueued pair must fail (no_episode).
    begin_with_row(true, "P0256", "prog-once");
    {
        let g_w = enter_slow("P0256");
        assert_eq!(check_gate("P0256", &g_w), Err("guard_inactive"));
    }
    end();
    println!("CM743_FALSIFIER PASS case=wrong_pair_and_reuse_rejected");

    // 10. Wrong row & wrong episode (different program).
    reset_clean();
    begin_with_row(false, "P0256", "prog-A");
    {
        let _g_w = enter_slow("P0256");
        let _g_d = enter_slow("P0489");
        arm("P0256", "producer-InputRequired", "p:1");
        let f = Arc::new(1_u8);
        record_descendant_frame(&f, "FrameA", 1);
    }
    end();
    // Different row on queued half:
    begin_with_row(true, "P0259", "prog-A");
    {
        let g_w = enter_slow("P0259");
        assert_eq!(check_gate("P0259", &g_w), Err("guard_inactive"));
    }
    end();
    // Different program on queued half:
    begin_with_row(true, "P0256", "prog-B");
    {
        let g_w = enter_slow("P0256");
        assert_eq!(check_gate("P0256", &g_w), Err("guard_inactive"));
    }
    end();
    // Wrong row query inside valid P0256 queued run:
    begin_with_row(true, "P0256", "prog-A");
    {
        let g_w = enter_slow("P0256");
        assert_eq!(check_gate("P0259", &g_w), Err("owner_missing"));
    }
    end();
    println!("CM743_FALSIFIER PASS case=wrong_row_and_wrong_episode_rejected");

    // 11. Tampering Pair::descendant_permits without Ledger::queued_descendant_issues fails!
    reset_clean();
    begin_with_row(false, "P0256", "prog-tamper");
    {
        let _g_w = enter_slow("P0256");
        let _g_d = enter_slow("P0489");
        arm("P0256", "producer-InputRequired", "p:1");
        let f = Arc::new(1_u8);
        record_descendant_frame(&f, "TamperFrame", 1);
    }
    end();
    // Clear independent Ledger table while leaving Pair intact.
    LEDGER.with(|cell| cell.borrow_mut().queued_descendant_issues.clear());
    begin_with_row(true, "P0256", "prog-tamper");
    {
        let g_w = enter_slow("P0256");
        assert_eq!(check_gate("P0256", &g_w), Err("ledger_issue_mismatch"));
    }
    end();
    println!("CM743_FALSIFIER PASS case=pair_tamper_without_ledger_rejected");

    // 12. Empty phase rejected; forged/inactive Guard rejected.
    reset_clean();
    begin_with_row(false, "P0256", "prog-phase");
    {
        let _g_w = enter_slow("P0256");
        let _g_d = enter_slow("P0489");
        arm("P0256", "producer-InputRequired", "p:1");
        let f = Arc::new(1_u8);
        record_descendant_frame(&f, "PhaseFrame", 1);
    }
    end();
    begin_with_row(true, "P0256", "prog-phase");
    {
        let g_w = enter_slow("P0256");
        assert!(!hit_descendant("P0256", "", &g_w));
        let forged_inactive = Guard::inactive();
        assert_eq!(check_gate("P0256", &forged_inactive), Err("guard_inactive"));
    }
    end();
    println!("CM743_FALSIFIER PASS case=empty_phase_and_forged_guard_rejected");
    println!("CM743_ROUTE_A_FALSIFIERS PASS total=12");
}
