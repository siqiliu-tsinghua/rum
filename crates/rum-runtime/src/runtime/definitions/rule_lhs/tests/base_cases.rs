use super::*;

fn rule_base_case_outcome(
    source: &str,
    direct: bool,
) -> (Result<Expr, String>, Vec<String>, String, String) {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("effect=0;bound=7;active[]:=(effect=effect+1;2)")
        .unwrap();
    let expr = shallow(&mut session, source);
    let mut out = if direct {
        session.drive_rule_lhs(
            RuleLhsState { stack: Vec::new() },
            RuleLhsStep::Visit(expr, RuleLhsVisit::Classify),
        )
    } else {
        session.evaluate_rule_lhs(expr)
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
fn rule_lhs_entry_base_cases_agree_with_driver() {
    let operands = [r#"InputString["held: "]"#, "(effect=99)", "active[]"];
    let mut inert = Vec::new();
    for arity in 0..=3 {
        inert.push(format!(
            "System`HoldPattern[{}]",
            operands[..arity].join(",")
        ));
    }
    for head in ["Blank", "BlankSequence", "BlankNullSequence"] {
        inert.push(format!("System`{head}[]"));
        inert.push(format!("System`Pattern[(effect=99),System`{head}[]]"));
    }
    for head in [
        "Alternatives",
        "Optional",
        "Except",
        "Repeated",
        "RepeatedNull",
    ] {
        inert.push(format!("System`{head}[]"));
    }
    for source in &inert {
        let entry = rule_base_case_outcome(source, false);
        assert_eq!(entry, rule_base_case_outcome(source, true), "{source}");
        assert!(entry.0.is_ok() && entry.1.is_empty(), "{source}: {entry:?}");
        assert_eq!(entry.2, "0", "inert operands ran: {source}");
    }
    let mut registry_cases = 0;
    for head in MatcherPatternHead::ALL {
        for arity in 0..=3 {
            let source = format!("System`{}[{}]", head.name(), operands[..arity].join(","));
            assert_eq!(
                rule_base_case_outcome(&source, false),
                rule_base_case_outcome(&source, true),
                "{source}"
            );
            registry_cases += 1;
        }
    }
    for source in [
        "7",
        "\"text\"",
        "atom",
        "bound",
        "h[]",
        "active[]",
        "h[1,active[]]",
        r#"(effect=effect+1;Throw["entry-error"])"#,
        r#"(effect=effect+1;Input["root: "])"#,
        r#"(Input["first: "];Input["second: "];active[])"#,
        "System`Pattern[]",
        "System`Pattern[active[]]",
        "System`Pattern[x,_,active[]]",
        "System`PatternTest[]",
        "System`Condition[active[]]",
        "System`OptionsPattern[active[]]",
        "System`Blank[active[]]",
        "System`Pattern[x,System`Blank[active[]]]",
        r#"System`Blank[Input["blank: "]]"#,
        r#"System`Pattern[x,System`Blank[Input["named: "]]]"#,
        "System`Alternatives[h[x_,active[]],k[y_]]",
        r#"System`Condition[h[x_,active[]],InputString["held-condition: "]]"#,
    ] {
        assert_eq!(
            rule_base_case_outcome(source, false),
            rule_base_case_outcome(source, true),
            "{source}"
        );
    }
    let resumed = rule_base_case_outcome(r#"(effect=effect+1;Input["root: "])"#, false);
    assert_eq!(resumed.0, Ok(Expr::small_integer(7)));
    assert_eq!(resumed.1, ["root: "]);
    assert_eq!(resumed.2, "1");
    eprintln!(
        "RULE_ENTRY_DRIVER_PARITY inert={} registry={registry_cases} ordinary/error/resume/malformed/descent=22 passed",
        inert.len()
    );
}
