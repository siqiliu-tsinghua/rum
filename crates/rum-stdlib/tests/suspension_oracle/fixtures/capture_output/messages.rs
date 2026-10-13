use super::super::super::descriptors::{Checks, Fixture, Variant, accept};
use super::super::primary::Descriptor;
use super::roles::{message, pause};
use rum_runtime::{RuntimeMessage, SessionRequestError};

fn tagged(tag: &str) -> RuntimeMessage {
    RuntimeMessage {
        symbol: "probe".into(),
        tag: tag.into(),
        text: tag.into(),
    }
}
const VARIANTS: &[Variant] = &[
    Variant::Middle,
    Variant::Twice,
    Variant::Error,
    Variant::AlreadyResumed,
    Variant::Nested,
];
pub fn descriptors() -> Vec<Descriptor> {
    [
        ("capture.check", "Check.body"),
        ("capture.quiet", "Quiet.body"),
        ("capture.confirm-quiet", "ConfirmQuiet.test"),
        ("capture.trace", "Trace.body"),
        ("capture.trace-detail", "TraceDetail.body"),
        ("capture.trace-print", "TracePrint.body"),
    ]
    .into_iter()
    .map(|(id, entry)| Descriptor {
        id,
        entry,
        variants: VARIANTS,
        factory: fixture,
        boundary: false,
        lifecycle: super::roles::exercise,
    })
    .collect()
}
fn fixture(entry: &str, v: Variant) -> Fixture {
    let twice = v != Variant::Middle;
    let mut f = Fixture::new(entry, "");
    f.setup = r#"Global`probe::other="other";Global`probe::outside="outside""#.into();
    f.pauses = vec![pause("message:1")];
    if twice {
        f.pauses.push(pause("message:2"));
    }
    let mut body = String::new();
    for i in 1..=3 {
        body.push_str(&format!("AppendTo[Global`log,{i}];Print[{i}];Message[Global`probe::tag];Message[Global`probe::other];"));
        if i == 1 || (i == 2 && twice) {
            body.push_str(&format!("InputString[\"message:{i}\"];"));
        }
    }
    body.push('7');
    let capture = match entry {
        "Check.body" => format!("Check[({body}),11,{{Global`probe::tag}}]"),
        "Quiet.body" => format!("Quiet[({body}),{{Global`probe::tag}}]"),
        "ConfirmQuiet.test" => {
            format!("Enclose[ConfirmQuiet[({body}),{{Global`probe::tag}}],Function[11]]")
        }
        _ => format!("{}[({body})];7", entry.split_once('.').unwrap().0),
    };
    f.source = format!(
        "Print[\"outside before\"];Message[Global`probe::outside];Global`result=({capture});Print[\"outside after\"];Message[Global`probe::outside];Global`result"
    );
    f.expected_value = Some(
        if matches!(entry, "Check.body" | "ConfirmQuiet.test") {
            "11"
        } else {
            "7"
        }
        .into(),
    );
    f.expected_log = Some("List[1, 2, 3]".into());
    f.pause_prints = Some(
        (1..=f.pauses.len())
            .map(|i| {
                let mut p = vec!["outside before".into()];
                p.extend((1..=i).map(|j| j.to_string()));
                p
            })
            .collect(),
    );
    let trace = entry.starts_with("Trace");
    f.pause_messages = Some(
        (1..=f.pauses.len())
            .map(|i| {
                let mut m = vec![tagged("outside")];
                if trace {
                    for _ in 0..i {
                        m.extend([message(), tagged("other")]);
                    }
                }
                m
            })
            .collect(),
    );
    f.pause_trace = Some(vec![vec![]; f.pauses.len()]);
    let mut messages = vec![tagged("outside")];
    for _ in 0..3 {
        if trace || entry == "Check.body" {
            messages.push(message());
        }
        messages.push(tagged("other"));
    }
    messages.push(tagged("outside"));
    f.expected_messages = Some(messages);
    if entry != "TracePrint.body" {
        f.expected_prints = Some(vec![
            "outside before".into(),
            "1".into(),
            "2".into(),
            "3".into(),
            "outside after".into(),
        ]);
    }
    if v == Variant::Error {
        f.source.push_str(";Exit[7]");
        f.expected_value = None;
        f.expected_error = Some(SessionRequestError::Exit { code: 7 });
    }
    if v == Variant::AlreadyResumed {
        f.source = format!("InputString[\"caller\"];{}", f.source);
        f.pauses.insert(0, pause("caller"));
        f.pause_prints.as_mut().unwrap().insert(0, vec![]);
        f.pause_messages.as_mut().unwrap().insert(0, vec![]);
        f.pause_trace.as_mut().unwrap().insert(0, vec![]);
        f.lifecycle_pauses = [1, 2];
    }
    if v == Variant::Nested {
        f.source = format!("EvaluationData[({}),\"Result\"]", f.source);
        f.pause_prints = Some(vec![vec![]; f.pauses.len()]);
        f.pause_messages = Some(vec![vec![]; f.pauses.len()]);
        f.expected_prints = Some(vec![]);
        f.expected_messages = Some(vec![]);
    }
    f
}
#[test]
fn message_and_trace_supplemental_matrix() {
    for d in descriptors() {
        super::super::primary::execute(&d);
    }
}
#[test]
fn nonmatching_messages_and_settled_fallback_and_info_are_public() {
    for source in [
        r#"Check[(Message[Global`probe::tag];InputString["first"];Message[Global`probe::tag];InputString["second"];7),19,{Global`probe::other}]"#,
        r#"Quiet[(Message[Global`probe::tag];InputString["first"];Message[Global`probe::tag];InputString["second"];7),{Global`probe::other}]"#,
        r#"ConfirmQuiet[(Message[Global`probe::tag];InputString["first"];Message[Global`probe::tag];InputString["second"];7),{Global`probe::other}]"#,
    ] {
        let mut f = Fixture::new("nonmatching", source);
        f.pauses = vec![pause("first"), pause("second")];
        f.expected_value = Some("7".into());
        f.pause_messages = Some(vec![vec![], vec![]]);
        f.expected_messages = Some(vec![message(); 2]);
        f.pause_prints = Some(vec![vec![]; 2]);
        f.pause_trace = Some(vec![vec![]; 2]);
        accept(&f, Checks::ALL).unwrap();
    }
    for (source, expected) in [
        (
            r#"Check[(Message[Global`probe::tag];InputString["first"];7),(Message[Global`probe::tag];InputString["second"];11)]"#,
            vec![message(); 2],
        ),
        (
            r#"Enclose[ConfirmQuiet[(Message[Global`probe::tag];InputString["first"];7),All,(Message[Global`probe::tag];InputString["second"];11)],Function[11]]"#,
            vec![message()],
        ),
    ] {
        let mut f = Fixture::new("settled callback", source);
        f.pauses = vec![pause("first"), pause("second")];
        f.expected_value = Some("11".into());
        f.expected_messages = Some(expected.clone());
        f.pause_messages = Some(vec![vec![], expected]);
        f.pause_prints = Some(vec![vec![]; 2]);
        f.pause_trace = Some(vec![vec![]; 2]);
        accept(&f, Checks::ALL).unwrap();
    }
}

#[test]
fn body_errors_settle_matching_and_nonmatching_message_policies() {
    let body = r#"(Message[Global`probe::tag];Message[Global`probe::other];InputString["first"];Message[Global`probe::tag];Message[Global`probe::other];InputString["second"];Message[Global`probe::tag];Message[Global`probe::other];Exit[7])"#;
    for (name, matched, catches) in [
        ("Check", true, true),
        ("Quiet", true, false),
        ("ConfirmQuiet", true, true),
        ("Check", false, false),
        ("Quiet", false, false),
        ("ConfirmQuiet", false, false),
    ] {
        let selector = if matched {
            "Global`probe::tag"
        } else {
            "Global`probe::missing"
        };
        let capture = match name {
            "Check" => format!("Check[{body},11,{{{selector}}}]"),
            "Quiet" => format!("Quiet[{body},{{{selector}}}]"),
            _ => format!("Enclose[ConfirmQuiet[{body},{{{selector}}}],Function[11]]"),
        };
        let mut f = Fixture::new("capture body error", &capture);
        f.setup = r#"Global`probe::other="other""#.into();
        f.pauses = vec![pause("first"), pause("second")];
        f.pause_prints = Some(vec![vec![]; 2]);
        f.pause_messages = Some(vec![vec![]; 2]);
        f.pause_trace = Some(vec![vec![]; 2]);
        f.expected_messages = Some(
            (0..3)
                .flat_map(|_| {
                    if matched && name != "Check" {
                        vec![tagged("other")]
                    } else {
                        vec![message(), tagged("other")]
                    }
                })
                .collect(),
        );
        if catches {
            f.expected_value = Some("11".into());
        } else {
            f.expected_error = Some(SessionRequestError::Exit { code: 7 });
        }
        accept(&f, Checks::ALL).unwrap_or_else(|e| panic!("{name} matched={matched}: {e:?}"));
    }
}
