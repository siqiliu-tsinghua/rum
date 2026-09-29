//! REPL input contract: crate rustdoc and guide/getting-started.md, brief-553 U1/U2.

mod support;

use std::cell::RefCell;
use std::io::{self, BufRead, Cursor, Read, Write};
use std::rc::Rc;

use rum_host::HostEvalOutcome;
use rum_repl::Repl;
use support::*;

fn red_resume(source: &str, guest: &str, prompts: &[&str], answers: &[&str]) {
    let mut probe = session();
    let mut outcome = probe.eval(guest);
    for (prompt, answer) in prompts.iter().zip(answers) {
        let HostEvalOutcome::InputRequired(suspension) = outcome else {
            panic!("fixture must suspend before each answer: {outcome:?}");
        };
        assert_eq!(&suspension.request.prompt, prompt);
        assert!(!suspension.request.password);
        outcome = probe.resume_with_input(*answer);
    }
    let mut queued_probe = session();
    for answer in answers {
        queued_probe.queue_input_response(*answer);
    }
    assert_eq!(
        outcome,
        queued_probe.eval(guest),
        "host queued/resumed parity"
    );

    let mut queued = session();
    for answer in answers {
        queued.queue_input_response(*answer);
    }
    let expected = run(&mut queued, &format!("{source}\n1+1\n:quit\n"));
    assert!(
        !expected.contains("input required"),
        "queued path must complete: {expected}"
    );
    let mut interactive = session();
    let actual = run(
        &mut interactive,
        &format!("{source}\n{}\n1+1\n:quit\n", answers.join("\n")),
    );
    assert!(
        interactive.pending_input_request().is_none(),
        "REPL must resume its request: {actual}"
    );
    // Prompts precede final trace rendering, which can itself quote the source prompt.
    let mut without_prompts = actual.clone();
    for prompt in prompts {
        assert!(
            without_prompts.contains(prompt),
            "missing request: {actual}"
        );
        without_prompts = without_prompts.replacen(prompt, "", 1);
    }
    assert_eq!(
        without_prompts, expected,
        "same result, messages, prints and trace rendering"
    );
    assert_usable(&mut interactive);
    assert_eq!(
        value(interactive.eval("{before,after}")),
        value(queued.eval("{before,after}"))
    );
}

#[test]
fn red_line_resume_matches_queued() {
    red_resume("Input[\"p: \"]", "Input[\"p: \"]", &["p: "], &["6*7"]);
}

#[test]
fn red_two_inputs_resume_matches_queued() {
    let source = "a=Input[\"first: \"]; b=InputString[\"second: \"]; {a,b}";
    red_resume(
        source,
        source,
        &["first: ", "second: "],
        &["6*7", "literal"],
    );
}

#[test]
fn red_nested_get_resume_matches_queued() {
    let source = "Get[\"input-outer.rum\"]";
    red_resume(source, source, &["nested: "], &["raw"]);
}

#[test]
fn red_plain_trace_resume_matches_queued() {
    red_resume(
        ":trace InputString[\"p: \"]",
        "Trace[InputString[\"p: \"]]",
        &["p: "],
        &["raw"],
    );
}

#[test]
fn red_trace_resume_matches_queued() {
    red_resume(
        ":trace --rules --full InputString[\"p: \"]",
        "TraceDetail[InputString[\"p: \"]]",
        &["p: "],
        &["raw"],
    );
}

#[test]
fn trace_form_and_depth_resume_matches_queued() {
    red_resume(
        ":trace --rules --form Plus --depth 4 --full a=Input[\"p: \"]; a+1",
        "TraceDetail[a=Input[\"p: \"]; a+1, _Plus, 4]",
        &["p: "],
        &["6*7"],
    );
}

#[test]
fn trace_full_and_default_limit_survive_input() {
    let guest = "InputString[\"p: \"]; Table[i^2, {i, 12}]";
    for flags in ["", "--full "] {
        red_resume(
            &format!(":trace {flags}{guest}"),
            &format!("Trace[{guest}]"),
            &["p: "],
            &["raw"],
        );
    }
}

#[test]
fn red_answer_is_literal_and_not_saved_in_history() {
    assert_suspends("answer=InputString[\"p: \"]", "p: ", false);
    let path = HistoryFile::new();
    let mut session = session();
    let mut output = Vec::new();
    let input = "answer=InputString[\"p: \"]\n:quit\n1+1\n:quit\n";
    Repl::new(Cursor::new(input), &mut output)
        .with_history_path(&path.0)
        .run(&mut session)
        .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(
        output.contains("\":quit\""),
        "answer must be literal: {output}"
    );
    assert_eq!(value(session.eval("answer")), "\":quit\"");
    let history = std::fs::read_to_string(&path.0).unwrap();
    assert_eq!(history.lines().filter(|line| *line == ":quit").count(), 1);
    assert_eq!(history, "answer=InputString[\"p: \"]\n1+1\n:quit\n");
    assert_usable(&mut session);
}

#[test]
fn empty_and_whitespace_answers_are_preserved() {
    let source = "answer=InputString[\"p: \"]";
    assert_suspends(source, "p: ", false);
    for answer in ["", " \t  ", "  literal ", "x\ry", "x\r"] {
        let mut session = session();
        let output = run(&mut session, &format!("{source}\r\n{answer}\r\n:quit\n"));
        assert_usable(&mut session);
        let HostEvalOutcome::Success(response) = session.eval("answer") else {
            panic!("{output}");
        };
        assert_eq!(
            response.value.typed,
            rum_host::HostTyped::Str(answer.to_string())
        );
    }
}

#[test]
fn empty_prompt_uses_input_prompt() {
    assert_suspends("InputString[]", "", false);
    let mut session = session();
    assert_eq!(
        run(&mut session, "InputString[]\n\n:quit\n"),
        "rum> input> \"\"\nrum> "
    );
    assert_usable(&mut session);
}

#[test]
fn malformed_input_is_left_to_host_without_reprompt() {
    assert_suspends("Input[\"p: \"]", "p: ", false);
    let mut session = session();
    let output = run(&mut session, "Input[\"p: \"]\nf[1,]\n1+1\n:quit\n");
    assert_eq!(output.matches("p: ").count(), 1, "{output}");
    assert!(output.contains("error: expected expression"), "{output}");
    assert!(output.ends_with("rum> 2\nrum> "), "{output}");
    assert_usable(&mut session);
}

#[derive(Default)]
struct Transcript {
    text: Vec<u8>,
    flushed: usize,
}

struct ObservedWriter(Rc<RefCell<Transcript>>);
impl Write for ObservedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().text.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        let mut transcript = self.0.borrow_mut();
        transcript.flushed = transcript.text.len();
        Ok(())
    }
}

struct ObservedReader {
    source: Cursor<String>,
    transcript: Rc<RefCell<Transcript>>,
    reads: usize,
}
impl Read for ObservedReader {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.source.read(bytes)
    }
}
impl BufRead for ObservedReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.source.fill_buf()
    }
    fn consume(&mut self, n: usize) {
        self.source.consume(n);
    }
    fn read_line(&mut self, line: &mut String) -> io::Result<usize> {
        if self.reads == 1 || self.reads == 2 {
            let transcript = self.transcript.borrow();
            let text = String::from_utf8_lossy(&transcript.text);
            let prompt = if self.reads == 1 {
                " first?\t"
            } else {
                "second: "
            };
            assert!(
                text.ends_with(prompt),
                "verbatim request prompt before answer: {text}"
            );
            assert_eq!(
                transcript.flushed,
                transcript.text.len(),
                "flush before answer"
            );
            assert_eq!(
                text.matches("printed-once").count(),
                self.reads,
                "print prefix: {text}"
            );
            assert_eq!(
                text.matches("! General::stop:").count(),
                self.reads,
                "message prefix: {text}"
            );
            assert_eq!(text.matches("rum> ").count(), 1);
            assert!(!text.contains("...> "));
        }
        self.reads += 1;
        self.source.read_line(line)
    }
}

#[test]
fn prefix_output_is_flushed_before_each_prompt_and_never_replayed() {
    let source = "Print[\"printed-once\"]; Message[General::stop]; InputString[\" first?\\t\"]; Print[\"printed-once\"]; Message[General::stop]; InputString[\"second: \"]; 42";
    assert_suspends(source, " first?\t", false);
    let transcript = Rc::new(RefCell::new(Transcript::default()));
    let input = ObservedReader {
        source: Cursor::new(format!("{source}\none\ntwo\n:quit\n")),
        transcript: transcript.clone(),
        reads: 0,
    };
    let mut session = session();
    Repl::new(input, ObservedWriter(transcript.clone()))
        .run(&mut session)
        .unwrap();
    let text = String::from_utf8(transcript.borrow().text.clone()).unwrap();
    assert_eq!(text.matches("printed-once").count(), 2, "{text}");
    assert_eq!(text.matches("! General::stop:").count(), 2, "{text}");
    assert!(text.ends_with("second: 42\nrum> "), "{text}");
    assert_usable(&mut session);
}
