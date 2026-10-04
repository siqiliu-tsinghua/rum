use super::*;

// All depth cases execute inside an explicitly sized embedder thread.
const SHALLOW: usize = 32;
const DEEP: usize = 2_000;
fn on_embedder_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name("lazy-pull-2mib".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap();
}
// A setup built by Do keeps parser depth constant. Each incrementing Map must
// execute to reach n; each Drop[1] must skip to reach n. Select counts every
// predicate, including the two different rejection/restart boundaries.
fn depth_case(kind: &str, n: usize) -> (String, Vec<&'static str>, String, String) {
    let inc = "Function[z,(log=log+1;z+1)]";
    let ask = "Function[z,(log=log+1;InputString[\"p: \"];z+1)]";
    match kind {
        "invalid_zero" => (
            format!("log=0;x=LazyList[bad];Do[x=LazyMap[{inc},x],{{{n}}}];Head[Take[x,0]]"),
            vec![],
            "Take".into(),
            "0".into(),
        ),
        "no_pull" => (
            format!("log=0;x=LazyRange[0];Do[x=LazyMap[{inc},x],{{{n}}}];Take[x,0]"),
            vec![],
            "List[]".into(),
            "0".into(),
        ),
        "pull" => (
            format!("log=0;x=LazyRange[0];Do[x=LazyMap[{inc},x],{{{n}}}];First[x]"),
            vec![],
            n.to_string(),
            n.to_string(),
        ),
        "inner" => (
            format!(
                "log=0;x=LazyMap[{ask},LazyRange[0]];Do[x=LazyMap[{inc},x],{{{}}}];Take[x,1]",
                n - 1
            ),
            vec!["a"],
            format!("List[{n}]"),
            n.to_string(),
        ),
        "outer" => (
            format!(
                "log=0;x=LazyRange[0];Do[x=LazyMap[{inc},x],{{{}}}];x=LazyMap[{ask},x];Take[x,1]",
                n - 1
            ),
            vec!["a"],
            format!("List[{n}]"),
            n.to_string(),
        ),
        "twice" => (
            format!(
                "log=0;x=LazyMap[{ask},LazyRange[0]];Do[x=LazyMap[{inc},x],{{{}}}];x=LazyMap[{ask},x];Take[x,1]",
                n - 2
            ),
            vec!["a", "b"],
            format!("List[{n}]"),
            n.to_string(),
        ),
        "select" => (
            format!(
                "log=0;x=LazySelect[Function[z,(log=log+1;z>=1)],LazyRange[0]];Do[x=LazySelect[Function[z,(log=log+1;True)],x],{{{}}}];x=LazySelect[Function[z,(log=log+1;z>=2)],x];First[x]",
                n - 2
            ),
            vec![],
            "2".into(),
            (2 * n + 1).to_string(),
        ),
        "drop" => (
            format!(
                "log=0;x=LazyMap[Function[z,(log=log+1;z)],LazyRange[0]];Do[x=LazyDrop[x,1],{{{n}}}];First[x]"
            ),
            vec![],
            n.to_string(),
            (n + 1).to_string(),
        ),
        _ => panic!("unknown case"),
    }
}
fn depth_compare(kind: &'static str, n: usize) {
    on_embedder_stack(move || {
        let (source, inputs, value, log) = depth_case(kind, n);
        let resumed = run(&source, &inputs, false, None);
        let queued = run(&source, &inputs, true, None);
        record(
            &format!("Depth-{kind}-{n}"),
            &resumed,
            &queued,
            inputs.len(),
        );
        assert_eq!(
            queued.0.value.as_ref().unwrap().to_string(),
            value,
            "exact queued result"
        );
        assert_eq!(queued.1, log, "every stage/source effect in queued control");
        assert_eq!(queued.0.status, "OK");
        assert_eq!(queued.0.termination, "None");
        assert!(queued.0.messages.is_empty());
        assert!(queued.0.prints.is_empty());
        assert_eq!(
            resumed.2,
            vec!["p: "; inputs.len()],
            "all suspensions reached"
        );
        assert_eq!(resumed.0, queued.0, "complete depth result");
        assert_eq!(resumed.1, queued.1, "every stage once");
    });
}
fn depth_cancel_run(n: usize, queued: bool) -> (View, String, Vec<String>) {
    let source = format!(
        "log=0;x=LazyMap[Function[z,(log=log+1;Print[\"boundary\"];InputString[\"p: \"];z+1)],LazyRange[0]];Do[x=LazyMap[Function[z,(log=log+1;z+1)],x],{{{}}}];Take[x,1]",
        n - 1
    );
    let mut s = session(None);
    let cancel = s.request_cancel_handle();
    let on_print = cancel.clone();
    let prints = Arc::new(Mutex::new(Vec::<String>::new()));
    let capture = prints.clone();
    s.set_print_sink(Box::new(ClosurePrintSink::assert_bounded_destruction(
        move |line| {
            capture.lock().unwrap().push(line.to_owned());
            if queued && line == "boundary" {
                on_print.cancel();
            }
        },
    )));
    if queued {
        s.queue_input_response("unused");
    }
    let mut out = s.eval_request(&source);
    let mut prompts = Vec::new();
    if !queued {
        let Err(f) = &out else {
            panic!("must suspend before cancellation")
        };
        let SessionRequestError::InputRequired { request } = &f.error else {
            panic!("must require input")
        };
        prompts.push(request.prompt.clone());
        cancel.cancel();
        out = s.resume_eval_request_with_input("unused");
    }
    s.clear_print_sink();
    let mut result = finish(&mut s, out, prompts);
    assert!(result.0.prints.is_empty());
    result.0.prints = prints.lock().unwrap().clone();
    result
}
fn depth_cancel(n: usize) {
    on_embedder_stack(move || {
        let resumed = depth_cancel_run(n, false);
        let queued = depth_cancel_run(n, true);
        record(&format!("Depth-cancel-{n}"), &resumed, &queued, 1);
        assert_eq!(queued.0.termination, "Some(Cancelled)");
        assert_eq!(
            queued.0.value.as_ref().unwrap().to_string(),
            "TerminatedEvaluation[\"Cancelled\"]"
        );
        assert_eq!(queued.1, "1");
        assert_eq!(resumed.2, vec!["p: "]);
        assert_eq!(resumed.0, queued.0);
        assert_eq!(resumed.1, queued.1, "no outer stage executes after abandon");
    });
}
macro_rules! depth_pair {
    ($shallow:ident,$deep:ident,$kind:literal) => {
        #[test]
        fn $shallow() {
            depth_compare($kind, SHALLOW);
        }
        #[test]
        fn $deep() {
            depth_compare($kind, DEEP);
        }
    };
}
depth_pair!(depth_no_pull_shallow, depth_no_pull_deep, "no_pull");
depth_pair!(depth_pull_shallow, depth_pull_deep, "pull");
depth_pair!(depth_inner_shallow, depth_inner_deep, "inner");
depth_pair!(depth_outer_shallow, depth_outer_deep, "outer");
depth_pair!(depth_twice_shallow, depth_twice_deep, "twice");
depth_pair!(depth_select_shallow, depth_select_deep, "select");
depth_pair!(depth_drop_shallow, depth_drop_deep, "drop");
#[test]
fn depth_cancel_shallow() {
    depth_cancel(SHALLOW);
}
#[test]
fn depth_cancel_deep() {
    depth_cancel(DEEP);
}

// Even a zero-output realization must validate the deepest descriptor.
depth_pair!(
    depth_invalid_zero_shallow,
    depth_invalid_zero_deep,
    "invalid_zero"
);

// This control exercises the descriptor's own ownership without constructing a
// pull machine. The same 2 MiB thread owns Session destruction.
#[test]
fn descriptor_control_deep() {
    on_embedder_stack(|| {
        let mut s = session(None);
        let out = s
            .eval_request(&format!(
                "x=LazyRange[0];Do[x=LazyMap[Function[z,(log=log+1;z+1)],x],{{{DEEP}}}];0"
            ))
            .unwrap();
        assert_eq!(out.value.to_string(), "0");
        drop(out);
        drop(s);
        println!("DESCRIPTOR CONTROL depth={DEEP} stack=2097152 result=0 teardown=complete");
    });
}
