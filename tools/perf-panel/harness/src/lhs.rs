use rum_host::{HostEvalOutcome, HostSession};

struct Workload {
    setup: &'static str,
    source: &'static str,
    projection: &'static str,
    stored: &'static str,
    prompts: &'static [&'static str],
    base_prompts: usize,
    cycle: bool,
}

fn workload(label: &str) -> Option<Workload> {
    let cycle = label.ends_with("-cycle");
    let setup = if cycle {
        "xL := (InputString[\"cycle\"]; xL); yL := (InputString[\"later\"]; yL); zL := (InputString[\"prior\"]; 2);"
    } else {
        "xL := (InputString[\"cycle\"]; aL); yL := (InputString[\"later\"]; bL); zL := (InputString[\"prior\"]; 2);"
    };
    let (source, projection, stored, prompts, base_prompts): (_, _, _, &[&str], _) =
        match label.strip_suffix(if cycle { "-cycle" } else { "-value" })? {
            "lhs-arg" => (
                "fL[xL] = 3",
                "DownValues[fL]",
                if cycle {
                    "List[RuleDelayed[HoldPattern[fL[xL]], 3]]"
                } else {
                    "List[RuleDelayed[HoldPattern[fL[aL]], 3]]"
                },
                &["cycle"],
                1,
            ),
            "lhs-head" => (
                "xL[1] = 3",
                if cycle {
                    "{DownValues[xL], SubValues[xL]}"
                } else {
                    "{DownValues[aL], SubValues[aL]}"
                },
                if cycle {
                    "List[List[RuleDelayed[HoldPattern[xL[1]], 3]], List[]]"
                } else {
                    "List[List[RuleDelayed[HoldPattern[aL[1]], 3]], List[]]"
                },
                &["cycle"],
                1,
            ),
            "lhs-pattern" => (
                "fL[Blank[xL]] = 3",
                "DownValues[fL]",
                if cycle {
                    "List[RuleDelayed[HoldPattern[fL[Blank[xL]]], 3]]"
                } else {
                    "List[RuleDelayed[HoldPattern[fL[Blank[aL]]], 3]]"
                },
                &["cycle"],
                1,
            ),
            "lhs-after-prior" => (
                "fL[zL, xL] = 3",
                "DownValues[fL]",
                if cycle {
                    "List[RuleDelayed[HoldPattern[fL[2, xL]], 3]]"
                } else {
                    "List[RuleDelayed[HoldPattern[fL[2, aL]], 3]]"
                },
                &["prior", "cycle"],
                2,
            ),
            "lhs-twice" => (
                "fL[xL, yL] = 3",
                "DownValues[fL]",
                if cycle {
                    "List[RuleDelayed[HoldPattern[fL[xL, yL]], 3]]"
                } else {
                    "List[RuleDelayed[HoldPattern[fL[aL, bL]], 3]]"
                },
                &["cycle", "later"],
                if cycle { 1 } else { 2 },
            ),
            _ => return None,
        };
    Some(Workload {
        setup,
        source,
        projection,
        stored,
        prompts,
        base_prompts,
        cycle,
    })
}

pub fn run(label: &str, n: usize, arm: Option<&str>) -> bool {
    let Some(w) = workload(label) else {
        return false;
    };
    let broken = arm == Some("base") && w.cycle;
    assert!(
        matches!(arm, Some("base" | "cand")),
        "explicit measurement arm"
    );
    let mut session = HostSession::new();
    let mut resumes = 0;
    for _ in 0..n {
        let setup = format!("Clear[fL, xL, yL, zL, aL, bL];{}", w.setup);
        let HostEvalOutcome::Success(ready) = session.eval(setup.as_str()) else {
            panic!("setup failed")
        };
        assert!(ready.termination.is_none());
        let mut outcome = session.eval(w.source);
        let mut iteration_resumes = 0;
        loop {
            match outcome {
                HostEvalOutcome::InputRequired(suspension) => {
                    assert_eq!(suspension.request.prompt, w.prompts[iteration_resumes]);
                    assert!(suspension.messages.is_empty() && suspension.prints.is_empty());
                    assert!(session.pending_input_request().is_some());
                    iteration_resumes += 1;
                    resumes += 1;
                    outcome = session.resume_with_input("1");
                }
                HostEvalOutcome::Success(done) => {
                    assert!(done.prints.is_empty());
                    if broken {
                        assert_eq!(done.value.text, "TerminatedEvaluation[\"IterationLimit\"]");
                        assert!(done.termination.is_some());
                        assert_eq!(done.messages.len(), 1);
                        assert_eq!(done.messages[0].tag, "passlim");
                    } else {
                        assert_eq!(done.value.text, "3");
                        assert!(done.termination.is_none() && done.messages.is_empty());
                    }
                    assert_eq!(
                        iteration_resumes,
                        if broken {
                            w.base_prompts
                        } else {
                            w.prompts.len()
                        }
                    );
                    break;
                }
                HostEvalOutcome::Failure(failure) => panic!("{failure:?}"),
            }
        }
        assert!(session.pending_input_request().is_none());
        let HostEvalOutcome::Success(stored) = session.eval(w.projection) else {
            panic!("projection failed")
        };
        assert!(
            stored.termination.is_none() && stored.messages.is_empty() && stored.prints.is_empty()
        );
        let expected = if broken {
            if label.starts_with("lhs-head") {
                "List[List[], List[]]"
            } else {
                "List[]"
            }
        } else {
            w.stored
        };
        assert_eq!(stored.value.text, expected);
    }
    println!("PASS {label} resumes={resumes}");
    true
}
