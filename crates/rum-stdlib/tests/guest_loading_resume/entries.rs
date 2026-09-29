use super::*;

#[test]
fn red_get() {
    let (n, equal) = run(
        "red_get",
        "Get[\"Dep549`\"]",
        &["one", "two", "three", "four", "five", "six"],
        &[],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_get_then_q() {
    let (n, equal) = run(
        "red_get_then_q",
        "Get[\"Dep549`\"];InputString[\"q: \"]",
        &["one", "two", "three", "four", "five", "six"],
        &[],
    );
    assert_eq!(n, 2, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_needs() {
    let (n, equal) = run(
        "red_needs",
        "Needs[\"Dep549`\"]",
        &["one", "two", "three", "four", "five", "six"],
        &[],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_needs_then_q() {
    let (n, equal) = run(
        "red_needs_then_q",
        "Needs[\"Dep549`\"];InputString[\"q: \"]",
        &["one", "two", "three", "four", "five", "six"],
        &[],
    );
    assert_eq!(n, 2, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_begin_package() {
    let (n, equal) = run(
        "red_begin_package",
        "BeginPackage[\"Main549`\",{\"Dep549`\",\"After549`\"}]",
        &["one", "two", "three", "four", "five", "six"],
        &[],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_begin_package_then_q() {
    let (n, equal) = run(
        "red_begin_package_then_q",
        "BeginPackage[\"Main549`\",{\"Dep549`\",\"After549`\"}];InputString[\"q: \"]",
        &["one", "two", "three", "four", "five", "six"],
        &[],
    );
    assert_eq!(n, 2, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_stub() {
    let (n, equal) = run(
        "red_stub",
        "DeclarePackage[\"Dep549`\",{\"foo\"}];Dep549`foo",
        &["one", "two", "three", "four", "five", "six"],
        &[],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_stub_then_q() {
    let (n, equal) = run(
        "red_stub_then_q",
        "DeclarePackage[\"Dep549`\",{\"foo\"}];Dep549`foo;InputString[\"q: \"]",
        &["one", "two", "three", "four", "five", "six"],
        &[],
    );
    assert_eq!(n, 2, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_needs_explicit() {
    let (n, equal) = run(
        "red_needs_explicit",
        "Needs[\"Other549`\",File[\"Dep549.rum\"]];{Dep549`foo,$Packages}",
        &["one"],
        &[],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_begin_dependency_second() {
    let (n, equal) = run(
        "red_begin_dependency_second",
        "BeginPackage[\"Main549`\",{\"After549`\",\"Dep549`\"}]",
        &["one"],
        &[],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_begin_two_dependencies_suspend() {
    let (n, equal) = run(
        "red_begin_two_dependencies_suspend",
        "BeginPackage[\"Main549`\",{\"Dep549`\",\"Second549`\"}]",
        &["one", "two"],
        &[(
            "Second549.rum",
            "AppendTo[Global`log,5]\nInputString[\"second-dep: \"]\nAppendTo[Global`log,7]\n99",
        )],
    );
    assert_eq!(n, 2, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_nested_twice_filename() {
    let (n, equal) = run(
        "red_nested_twice_filename",
        "Get[\"Outer549`\"]",
        &["one", "two"],
        &[
            (
                "Outer549.rum",
                "AppendTo[Global`log,{1,$InputFileName}]\nGet[\"Inner549`\"]\nAppendTo[Global`log,{5,$InputFileName}]\nInputString[\"outer: \"]\nAppendTo[Global`log,{7,$InputFileName}]\n77",
            ),
            (
                "Inner549.rum",
                "AppendTo[Global`log,{2,$InputFileName}]\nInputString[\"inner: \"]\nAppendTo[Global`log,{4,$InputFileName}]\n44",
            ),
        ],
    );
    assert_eq!(n, 2, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}

#[test]
fn red_nested_resume_then_error() {
    let (n, equal) = run(
        "red_nested_resume_then_error",
        "Get[\"Outer549`\"]",
        &["one"],
        &[
            (
                "Outer549.rum",
                "AppendTo[Global`log,0]\nGet[\"Bad549`\"]\nAppendTo[Global`log,5]\n88",
            ),
            (
                "Bad549.rum",
                "Print[\"before\"]\nGlobal`marker=10\nAppendTo[Global`log,1]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nThrow[9]",
            ),
        ],
    );
    assert_eq!(n, 1, "must actually suspend at every input");
    assert!(
        equal,
        "resumed value/termination/log/state/messages/prints/fresh request must equal queued control"
    );
}
