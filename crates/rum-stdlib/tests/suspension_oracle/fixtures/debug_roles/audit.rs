use super::super::super::descriptors::{Checks, Fixture, Variant, accept};

fn raw(f: &Fixture, queued: bool) -> String {
    std::panic::catch_unwind(|| raw_inner(f, queued))
        .unwrap_or_else(|_| "PANIC (see panic location in log)".into())
}
fn raw_inner(f: &Fixture, queued: bool) -> String {
    let mut s = f.session();
    if queued {
        for p in &f.pauses {
            s.queue_input_response(&p.answer);
        }
    }
    let mut out = s.eval_request(&f.source);
    let mut rows = Vec::new();
    let mut next = 0;
    loop {
        let o = super::super::super::support::observe::Observation::from_output(&out);
        rows.push(format!(
            "value={:?} error={:?} termination={:?} messages={:?} prints={:?}",
            o.value.map(|v| v.to_string()),
            o.error,
            o.termination,
            o.messages,
            o.prints
        ));
        if s.pending_input_request().is_none() || queued || next == f.pauses.len() {
            break;
        }
        out = s.resume_eval_request_with_input(&f.pauses[next].answer);
        next += 1;
    }
    super::super::super::support::session::retire(&mut s);
    rows.push(format!(
        "log={} projections={:?}",
        s.eval_request("Global`log").unwrap().value,
        f.projections
            .iter()
            .map(|p| s.eval_request(p).unwrap().value.to_string())
            .collect::<Vec<_>>()
    ));
    rows.join("\n")
}

#[test]
fn complete_role_descriptors() {
    let mut failures = Vec::new();
    for d in super::descriptors() {
        for &v in d.variants {
            let f = (d.factory)(d.entry, v);
            match std::panic::catch_unwind(|| accept(&f, Checks::ALL)) {
                Ok(Ok(_)) => println!("DEBUG_VARIANT {} {v:?} PASS", d.id),
                error => {
                    let diagnostic = format!(
                        "{} {v:?}: {error:?}\nsetup={}\nsource={}\nanswers={:?}\nresumed={}\nqueued={}\n",
                        d.id,
                        f.setup,
                        f.source,
                        f.pauses,
                        raw(&f, false),
                        raw(&f, true)
                    );
                    println!("{diagnostic}");
                    failures.push(diagnostic);
                }
            }
        }
        let f = (d.factory)(d.entry, Variant::Twice);
        if std::panic::catch_unwind(|| (d.lifecycle)(&f)).is_err() {
            failures.push(format!("{} lifecycle", d.id));
        } else {
            println!("DEBUG_LIFECYCLE {} obligations=6 PASS", d.id);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
