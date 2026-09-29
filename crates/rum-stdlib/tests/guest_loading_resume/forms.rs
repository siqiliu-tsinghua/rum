use super::*;

#[test]
fn red_get_streams_line() {
    let (n, equal) = run(
        "red_get_streams_line",
        "Get[\"Obs549`\"]",
        &["one"],
        &[(
            "Obs549.rum",
            "Print[\"before\"]\nMessage[Global`probe::tag]\nAppendTo[Global`log,{1,$Line,$InputFileName}]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nAppendTo[Global`log,{3,$Line,$InputFileName}]\n77",
        )],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_get_twice() {
    let (n, equal) = run(
        "red_get_twice",
        "Get[\"Twice549`\"]",
        &["one", "two"],
        &[(
            "Twice549.rum",
            "AppendTo[Global`log,1]\nInputString[\"p: \"]\nAppendTo[Global`log,3]\nInputString[\"r: \"]\nAppendTo[Global`log,5]\n77",
        )],
    );
    assert_eq!(n, 2, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_get_error_after_resume() {
    let (n, equal) = run(
        "red_get_error_after_resume",
        "Get[\"Bad549`\"]",
        &["one"],
        &[(
            "Bad549.rum",
            "Print[\"before\"]\nGlobal`marker=10\nAppendTo[Global`log,1]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nThrow[9]",
        )],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_needs_error_after_resume() {
    let (n, equal) = run(
        "red_needs_error_after_resume",
        "Needs[\"Bad549`\"]",
        &["one"],
        &[(
            "Bad549.rum",
            "Print[\"before\"]\nGlobal`marker=10\nAppendTo[Global`log,1]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nThrow[9]",
        )],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_begin_error_after_resume() {
    let (n, equal) = run(
        "red_begin_error_after_resume",
        "BeginPackage[\"Main549`\",{\"After549`\",\"Bad549`\"}]",
        &["one"],
        &[(
            "Bad549.rum",
            "Print[\"before\"]\nGlobal`marker=10\nAppendTo[Global`log,1]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nThrow[9]",
        )],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_stub_error_after_resume() {
    let (n, equal) = run(
        "red_stub_error_after_resume",
        "DeclarePackage[\"Bad549`\",{\"foo\"}];Bad549`foo",
        &["one", "two"],
        &[(
            "Bad549.rum",
            "Print[\"before\"]\nGlobal`marker=10\nAppendTo[Global`log,1]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nThrow[9]",
        )],
    );
    assert_eq!(n, 2, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_get_input_expression() {
    let (n, equal) = run(
        "red_get_input_expression",
        "Get[\"Input549`\"]",
        &["2+3"],
        &[(
            "Input549.rum",
            "AppendTo[Global`log,1]\nInput[\"expr: \"]\nAppendTo[Global`log,3]\n77",
        )],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_get_input_syntax_error() {
    let (n, equal) = run(
        "red_get_input_syntax_error",
        "Get[\"Input549`\"]",
        &["]"],
        &[("Input549.rum", "Global`marker=10\nInput[\"expr: \"]\n77")],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_get_late_syntax_error() {
    let (n, equal) = run(
        "red_get_late_syntax_error",
        "Get[\"Syntax549`\"]",
        &["one"],
        &[(
            "Syntax549.rum",
            "Global`marker=10\nInputString[\"p: \"]\n1+*2",
        )],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_get_return_form() {
    let (n, equal) = run(
        "red_get_return_form",
        "Get[\"Return549`\"]",
        &["one"],
        &[(
            "Return549.rum",
            "InputString[\"p: \"]\nReturn[8]\nAppendTo[Global`log,3]\n77",
        )],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_get_abort_form() {
    let (n, equal) = run(
        "red_get_abort_form",
        "Get[\"Abort549`\"]",
        &["one"],
        &[(
            "Abort549.rum",
            "InputString[\"p: \"]\nAbort[]\nAppendTo[Global`log,3]\n77",
        )],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_get_exit_form() {
    let (n, equal) = run(
        "red_get_exit_form",
        "Get[\"Exit549`\"]",
        &["one"],
        &[(
            "Exit549.rum",
            "Global`marker=10\nInputString[\"p: \"]\nExit[7]\nAppendTo[Global`log,3]",
        )],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_get_cycle_form() {
    let (n, equal) = run(
        "red_get_cycle_form",
        "Get[\"Cycle549`\"]",
        &["one"],
        &[(
            "Cycle549.rum",
            "InputString[\"p: \"]\nGlobal`f549[]:=Global`g549[]\nGlobal`g549[]:=Global`f549[]\nGlobal`f549[]",
        )],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_get_dynamic_context() {
    let (n, equal) = run(
        "red_get_dynamic_context",
        "Block[{Global`marker=20},Get[\"Dyn549`\"]];Global`marker",
        &["one"],
        &[(
            "Dyn549.rum",
            "AppendTo[Global`log,Global`marker]\nInputString[\"p: \"]\nAppendTo[Global`log,Global`marker]\n77",
        )],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_get_message_checkpoint() {
    let (n, equal) = run(
        "red_get_message_checkpoint",
        "Message[Global`probe::tag];Get[\"Obs549`\"];$MessageList",
        &["one"],
        &[(
            "Obs549.rum",
            "Print[\"before\"]\nMessage[Global`probe::tag]\nAppendTo[Global`log,{1,$Line,$InputFileName}]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nAppendTo[Global`log,{3,$Line,$InputFileName}]\n77",
        )],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_get_outer_check() {
    let (n, equal) = run(
        "red_get_outer_check",
        "Check[Get[\"Obs549`\"],999]",
        &["one"],
        &[(
            "Obs549.rum",
            "Print[\"before\"]\nMessage[Global`probe::tag]\nAppendTo[Global`log,{1,$Line,$InputFileName}]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nAppendTo[Global`log,{3,$Line,$InputFileName}]\n77",
        )],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_get_outer_quiet() {
    let (n, equal) = run(
        "red_get_outer_quiet",
        "Quiet[Get[\"Obs549`\"]]",
        &["one"],
        &[(
            "Obs549.rum",
            "Print[\"before\"]\nMessage[Global`probe::tag]\nAppendTo[Global`log,{1,$Line,$InputFileName}]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nAppendTo[Global`log,{3,$Line,$InputFileName}]\n77",
        )],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_same_form_message_scope() {
    let (n, equal) = run(
        "red_same_form_message_scope",
        "Get[\"Scope549`\"]",
        &["one"],
        &[(
            "Scope549.rum",
            "Message[Global`probe::tag]\nInputString[\"p: \"];AppendTo[Global`log,$MessageList];Message[Global`probe::tag];AppendTo[Global`log,$MessageList]\n77",
        )],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_lower_later_forms() {
    let (n, equal) = run(
        "red_lower_later_forms",
        "Get[\"Lower549`\"]",
        &["one"],
        &[(
            "Lower549.rum",
            "InputString[\"p: \"]\nBegin[\"Lower549`\"]\nanswer=42\nEnd[]\nLower549`answer",
        )],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_repeat_form_stack() {
    let (n, equal) = run(
        "red_repeat_form_stack",
        "Get[\"Stack549`\"]",
        &["one", "two"],
        &[(
            "Stack549.rum",
            "Global`step549[]:=(AppendTo[Global`log,1];InputString[\"same: \"];AppendTo[Global`log,3];77)\nGlobal`step549[]\nGlobal`step549[]",
        )],
    );
    assert_eq!(n, 2, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}
