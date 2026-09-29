//! Guest loading follows the same-request input contract in guide/evaluation.md
//! and the package-loading contract in guide/scoping-and-contexts.md.

use rum_core::Expr;
use rum_runtime::kernel_api::{
    BoundedDestruction, FileIo, FileIoError, FileIoProvider, FileProbe, FileWriteMode,
};
use rum_runtime::{
    RuntimeMessage, Session, SessionEvalResponse, SessionRequestError, SessionRequestFailure,
};
use std::{
    collections::BTreeMap,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

const DEP: &str =
    "AppendTo[Global`log,1]\nInputString[\"p: \"]\nAppendTo[Global`log,3]\nDep549`foo=42\n77";
const OBS: &str =
    "{Global`log, $Context, $ContextPath, $Packages, $InputFileName, Global`marker, Global`$Line}";
struct Files {
    sources: BTreeMap<String, String>,
    reads: Arc<Mutex<Vec<String>>>,
}
impl BoundedDestruction for Files {}
impl FileIo for Files {
    fn read_to_string(&self, path: &Path) -> Result<String, FileIoError> {
        let name = path.file_name().and_then(|p| p.to_str()).unwrap_or("");
        self.reads.lock().unwrap().push(name.to_owned());
        self.sources
            .get(name)
            .cloned()
            .ok_or_else(|| FileIoError::new("probe file absent"))
    }
    fn probe(&self, path: &Path) -> FileProbe {
        if self
            .sources
            .contains_key(path.file_name().and_then(|p| p.to_str()).unwrap_or(""))
        {
            FileProbe::file(path.to_owned())
        } else {
            FileProbe::missing()
        }
    }
    fn write_string(&self, _: &Path, _: &str, _: FileWriteMode) -> Result<(), FileIoError> {
        Err(FileIoError::new("read-only probe provider"))
    }
}
fn session(files: &[(&str, &str)]) -> (Session, Arc<Mutex<Vec<String>>>) {
    let reads = Arc::new(Mutex::new(Vec::new()));
    let mut sources = BTreeMap::from([
        ("Dep549.rum".into(), DEP.into()),
        (
            "After549.rum".into(),
            "AppendTo[Global`log,5]\nAfter549`foo=55\n99".into(),
        ),
    ]);
    for (name, source) in files {
        sources.insert((*name).into(), (*source).into());
    }
    let mut s = Session::new();
    rum_stdlib::install_core(&mut s);
    s.set_file_io(Some(FileIoProvider::new(Files {
        sources,
        reads: reads.clone(),
    })));
    s.add_package_search_path(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("guest-loading-fixtures"),
    );
    s.eval_request(
        "Global`log={}; Global`marker=0; Global`$Line=100; Global`probe::tag=\"probe message\"",
    )
    .unwrap();
    (s, reads)
}
#[derive(PartialEq, Eq)]
struct View {
    status: String,
    value: Option<Expr>,
    termination: String,
    messages: Vec<RuntimeMessage>,
    prints: Vec<String>,
}
fn view(out: &Result<SessionEvalResponse, SessionRequestFailure>) -> View {
    match out {
        Ok(r) => View {
            status: "OK".into(),
            value: Some(r.value.clone()),
            termination: format!("{:?}", r.termination),
            messages: r.messages.clone(),
            prints: r.prints.clone(),
        },
        Err(f) => View {
            status: format!("{:?}", f.error),
            value: None,
            termination: "None".into(),
            messages: f.messages.clone(),
            prints: f.prints.clone(),
        },
    }
}
fn pending(s: &Session) -> String {
    format!("{:?}", s.pending_input_request())
}
fn plain(s: &mut Session, source: &str) -> String {
    match catch_unwind(AssertUnwindSafe(|| s.eval_request(source))) {
        Ok(out) => format!("{:?}", view(&out)),
        Err(_) => "PANIC".into(),
    }
}
fn inspect(s: &mut Session) -> String {
    match catch_unwind(AssertUnwindSafe(|| s.eval_text(OBS))) {
        Ok(Ok(out)) => out.value.to_string(),
        Ok(Err(e)) => format!("ERR {e:?}"),
        Err(_) => "PANIC".into(),
    }
}
fn run(label: &str, source: &str, inputs: &[&str], files: &[(&str, &str)]) -> (usize, bool) {
    let mut finals = Vec::new();
    println!("SOURCE\t{label}\t{source:?}\tinputs={inputs:?}\tfiles={files:?}");
    for queued in [false, true] {
        let mode = if queued { "queued" } else { "resumed" };
        let (mut s, reads) = session(files);
        if queued {
            for input in inputs {
                s.queue_input_response(*input);
            }
        }
        let mut result = catch_unwind(AssertUnwindSafe(|| s.eval_request(source)));
        let mut prompts = Vec::new();
        let mut steps = Vec::new();
        let mut n = 0;
        loop {
            match &result {
                Ok(out) => {
                    steps.push(format!("{:?}", view(out)));
                    if let Err(f) = out {
                        if let SessionRequestError::InputRequired { request } = &f.error {
                            prompts.push(request.prompt.clone());
                            if !queued && n < inputs.len() {
                                result = catch_unwind(AssertUnwindSafe(|| {
                                    s.resume_eval_request_with_input(inputs[n])
                                }));
                                n += 1;
                                continue;
                            }
                        }
                    }
                }
                Err(_) => steps.push("PANIC".into()),
            }
            break;
        }
        let final_view = result.as_ref().ok().map(view);
        let pend = pending(&s);
        let next = plain(&mut s, "1+1");
        let state = inspect(&mut s);
        println!(
            "CASE\t{label}\t{mode}\tsteps={steps:?}\tprompts={prompts:?}\tpending={pend}\tnext={next}\tstate={state:?}\treads={:?}",
            reads.lock().unwrap()
        );
        finals.push((final_view, state, next, prompts));
    }
    let equal =
        finals[0].0 == finals[1].0 && finals[0].1 == finals[1].1 && finals[0].2 == finals[1].2;
    println!(
        "ORACLE\t{label}\tsuspensions={}\tequal={equal}",
        finals[0].3.len()
    );
    assert!(finals[1].3.is_empty(), "queued control must finish");
    let prompts = &finals[0].3;
    let expected = match label {
        "red_get_then_q" | "red_needs_then_q" | "red_begin_package_then_q" | "red_stub_then_q" => {
            vec!["p: ", "q: "]
        }
        "red_get_twice" => vec!["p: ", "r: "],
        "red_begin_two_dependencies_suspend" => vec!["p: ", "second-dep: "],
        "red_nested_twice_filename" => vec!["inner: ", "outer: "],
        "red_stub_error_after_resume" => vec!["p: ", "p: "],
        "red_get_input_expression" | "red_get_input_syntax_error" => vec!["expr: "],
        "red_repeat_form_stack" => vec!["same: ", "same: "],
        _ if label.starts_with("green_") => vec![],
        _ => vec!["p: "],
    };
    assert_eq!(*prompts, expected, "must expose each prompt in order");
    if label.starts_with("green_") {
        let expected_value = match label {
            "green_valueq" => "True",
            "green_monitor_display" => "77",
            _ => "3",
        };
        for (out, _, _, _) in &finals {
            let out = out.as_ref().unwrap();
            assert_eq!(out.status, "OK");
            assert_eq!(out.value.as_ref().unwrap().to_string(), expected_value);
            assert_eq!(out.termination, "None");
        }
    }
    for (_, _, next, _) in &finals {
        assert_eq!(
            next,
            "View { status: \"OK\", value: Some(\"2\"), termination: \"None\", messages: [], prints: [] }",
            "fresh request must be accepted"
        );
    }
    (finals[0].3.len(), equal)
}

impl std::fmt::Debug for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("View")
            .field("status", &self.status)
            .field("value", &self.value.as_ref().map(ToString::to_string))
            .field("termination", &self.termination)
            .field("messages", &self.messages)
            .field("prints", &self.prints)
            .finish()
    }
}

#[path = "guest_loading_resume/budgets.rs"]
mod budgets;
#[path = "guest_loading_resume/cancellation.rs"]
mod cancellation;
#[path = "guest_loading_resume/entries.rs"]
mod entries;
#[path = "guest_loading_resume/forms.rs"]
mod forms;

#[test]
fn green_valueq_and_monitor_standalone_controls() {
    for (name, source) in [
        ("green_valueq", "ValueQ[InputString[\"a: \"]]"),
        ("green_monitor_display", "Monitor[77,InputString[\"a: \"]]"),
        (
            "green_monitor_eval",
            "Monitor[1+2,0,EvaluationMonitor:>InputString[\"a: \"]]",
        ),
        (
            "green_monitor_step",
            "Monitor[1+2,0,StepMonitor:>InputString[\"a: \"]]",
        ),
    ] {
        let (count, equal) = run(
            name,
            source,
            &["one", "two", "three", "four", "five", "six"],
            &[],
        );
        assert_eq!(count, 0);
        if name == "green_valueq" {
            assert!(equal);
        }
    }
}
