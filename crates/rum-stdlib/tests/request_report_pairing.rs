use rum_runtime::{EvalError, Session, SessionRequestError};

#[test]
fn non_driver_resume_retires_the_published_report() {
    let mut failures = Vec::new();
    for replace_owner in [false, true] {
        let mut session = Session::new();
        rum_stdlib::install_core(&mut session);
        let first = session
            .eval_request("Print[\"old-prefix\"];InputString[\"old-owner\"]")
            .unwrap_err();
        assert_eq!(first.prints, ["old-prefix"]);
        assert!(matches!(
            first.error,
            SessionRequestError::InputRequired { .. }
        ));
        assert!(
            session
                .resume_inspect_rewrite_rules_with_input("old-answer")
                .unwrap()
                .is_some()
        );
        assert!(session.pending_input_request().is_none());
        if replace_owner {
            let error = session.eval_text("InputString[\"new-owner\"]").unwrap_err();
            println!("B734 PAIRING replacement raw_error={error:?}");
            if !matches!(error, EvalError::InputRequired { ref request } if request.prompt == "new-owner")
            {
                failures.push("a retired report changed the new raw entry's outcome");
            }
        }
        let resumed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            session.resume_eval_request_with_input(if replace_owner {
                "new-answer"
            } else {
                "late"
            })
        }));
        match resumed {
            Ok(Err(failure)) => {
                println!(
                    "B734 PAIRING replace_owner={replace_owner} request_resume={:?} prints={:?} pending={:?}",
                    failure.error,
                    failure.prints,
                    session.pending_input_request()
                );
                assert!(
                    matches!(failure.error, SessionRequestError::Other { ref message } if message == "no suspended evaluation is waiting for host input")
                );
                assert!(failure.prints.is_empty());
                assert!(failure.messages.is_empty());
                assert!(failure.trace.is_empty());
                if replace_owner {
                    assert_eq!(session.pending_input_request().unwrap().prompt, "new-owner");
                    session
                        .resume_inspect_rewrite_rules_with_input("new-answer")
                        .unwrap();
                }
                assert!(session.pending_input_request().is_none());
                assert_eq!(session.eval_request("6*7").unwrap().value.to_string(), "42");
            }
            Ok(Ok(response)) => {
                println!(
                    "B734 PAIRING replace_owner={replace_owner} attached_value={} attached_prints={:?}",
                    response.value, response.prints
                );
                failures.push("a retired report attached to a different suspension");
            }
            Err(_) => {
                println!("B734 PAIRING replace_owner={replace_owner} vacant_slot_panic=true");
                failures.push("a retired report resumed a vacant suspension slot");
            }
        }
    }
    assert!(
        failures.is_empty(),
        "report/suspension pairing violations: {failures:?}"
    );
    println!(
        "B734 PAIRING PASS routes=2 report_retired=true new_raw_owner_preserved=true stale_output=false"
    );
}
