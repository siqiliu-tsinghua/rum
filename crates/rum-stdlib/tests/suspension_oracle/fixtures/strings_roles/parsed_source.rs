use super::super::super::descriptors::{Checks, Variant, accept, run};

fn check(v: Variant) {
    let f = super::text::fixture("ToExpression.parsedSource", v);
    for queued in [true, false] {
        let result = run(&f, queued, Checks::ALL).unwrap();
        println!(
            "ToExpression.parsedSource {v:?} queued={queued} value={:?} suffix={}",
            result.observation.value.as_ref().map(ToString::to_string),
            result.projections.last().unwrap()
        );
    }
    accept(&f, Checks::ALL).unwrap();
}
#[test]
fn parsed_source_middle() {
    check(Variant::Middle);
}
#[test]
fn parsed_source_twice() {
    check(Variant::Twice);
}
#[test]
fn parsed_source_already_resumed() {
    check(Variant::AlreadyResumed);
}
#[test]
fn parsed_source_nested() {
    check(Variant::Nested);
}
#[test]
fn parsed_source_error() {
    check(Variant::Error);
}
#[test]
fn parsed_source_lifecycle() {
    let f = super::text::fixture("ToExpression.parsedSource", Variant::Twice);
    let obligations = super::lifecycle::exercise(&f);
    println!(
        "ToExpression.parsedSource lifecycle first=PASS second=PASS obligations={obligations}"
    );
}

#[test]
fn parsed_source_then_suspending_head() {
    use super::super::super::descriptors::Pause;
    use rum_runtime::RuntimeInputKind;
    let mut f = super::text::fixture("ToExpression.parsedSource", Variant::Middle);
    f.source = f.source.replace(
        "Function[Global`x,(Global`suffix++;Global`x+100)]",
        "Function[Global`x,(Global`suffix++;Input[\"ToExpression.head\"]+Global`x)]",
    );
    f.pauses.push(Pause {
        prompt: "ToExpression.head".into(),
        kind: RuntimeInputKind::Input,
        answer: "100".into(),
    });
    f.pause_prints
        .as_mut()
        .unwrap()
        .push((1..=5).map(|n| n.to_string()).collect());
    f.pause_messages.as_mut().unwrap().push(vec![]);
    f.pause_trace.as_mut().unwrap().push(vec![]);
    accept(&f, Checks::ALL).unwrap();
}

#[test]
fn parsed_source_without_head() {
    let mut f = super::text::fixture("ToExpression.parsedSource", Variant::Twice);
    f.source = f.source.replace(
        ",InputForm,Function[Global`x,(Global`suffix++;Global`x+100)]",
        "",
    );
    f.expected_value = Some("7".into());
    *f.expected_projections.last_mut().unwrap() = "0".into();
    accept(&f, Checks::ALL).unwrap();
}
