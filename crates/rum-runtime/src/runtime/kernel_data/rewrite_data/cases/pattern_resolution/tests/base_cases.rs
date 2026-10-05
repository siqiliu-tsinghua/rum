use super::*;

fn matcher_base_case_outcome(
    source: &str,
    direct: bool,
) -> (Result<Expr, String>, Vec<String>, String, String) {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("effect=0;bound=7;SetAttributes[f,Orderless];active[]:=(effect=effect+1;HoldComplete[f[2,1]])")
        .unwrap();
    let expr = shallow(&mut session, source);
    let root = strip_hold_pattern(&expr);
    let mut out = if direct {
        session.drive_matcher_pattern(
            MatcherPatternState { stack: Vec::new() },
            MatcherPatternStep::Visit(root.clone()),
            Some(root),
        )
    } else {
        session.resolve_held_matcher_pattern(expr.clone())
    };
    let mut prompts = Vec::new();
    for _ in 0..3 {
        let Err(EvalError::InputRequired { request }) = &out else {
            break;
        };
        prompts.push(request.prompt.clone());
        out = session.resume_suspended_evaluation("7");
    }
    assert!(
        session.pending_suspension_generation().is_none(),
        "{source}"
    );
    let messages = format!("{:?}", session.take_messages());
    let effect = session.eval_text("effect").unwrap().value.to_string();
    (
        out.map_err(|error| format!("{error:?}")),
        prompts,
        effect,
        messages,
    )
}

#[test]
fn matcher_pattern_entry_base_cases_agree_with_driver() {
    let operands = [r#"InputString["opaque: "]"#, "(effect=99)", "f[2,1]"];
    let mut opaque_cases = 0;
    for &head in MatcherPatternHead::ALL {
        if head == MatcherPatternHead::OptionsPattern {
            continue;
        }
        for arity in 0..=3 {
            // Unary HoldPattern is stripped before classification in both paths.
            if head == MatcherPatternHead::HoldPattern && arity == 1 {
                continue;
            }
            let source = format!("System`{}[{}]", head.name(), operands[..arity].join(","));
            let entry = matcher_base_case_outcome(&source, false);
            assert_eq!(entry, matcher_base_case_outcome(&source, true), "{source}");
            assert!(entry.0.is_ok() && entry.1.is_empty(), "{source}: {entry:?}");
            assert_eq!(entry.2, "0", "opaque operands ran: {source}");
            opaque_cases += 1;
        }
    }
    for source in [
        "7",
        "\"text\"",
        "atom",
        "bound",
        "active[]",
        "h[1,2]",
        "HoldPattern[HoldPattern[active[]]]",
        "HoldPattern[HoldPattern[_Integer]]",
        "System`OptionsPattern[]",
        "System`OptionsPattern[active[]]",
        r#"(effect=effect+1;Throw["entry-error"])"#,
        r#"(effect=effect+1;InputString["root: "];HoldComplete[f[2,1]])"#,
        r#"(InputString["first: "];InputString["second: "];active[])"#,
        "h[_,active[]]",
        "(active[])[_,2]",
    ] {
        assert_eq!(
            matcher_base_case_outcome(source, false),
            matcher_base_case_outcome(source, true),
            "{source}"
        );
    }
    let normalized = matcher_base_case_outcome(
        r#"(effect=effect+1;InputString["root: "];HoldComplete[f[2,1]])"#,
        false,
    );
    assert_eq!(normalized.0.unwrap().to_string(), "HoldComplete[f[1, 2]]");
    assert_eq!(normalized.1, ["root: "]);
    assert_eq!(normalized.2, "1");
    eprintln!(
        "HELD_ENTRY_DRIVER_PARITY opaque={opaque_cases} ordinary/held/OptionsPattern/error/resume/descent=15 passed"
    );
}
