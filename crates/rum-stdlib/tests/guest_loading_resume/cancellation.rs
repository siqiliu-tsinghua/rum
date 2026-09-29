use super::*;

#[test]
fn red_cancel_cleanup_get_releases_owner() {
    let (mut s, _) = session(&[]);
    let cancel = s.request_cancel_handle();
    let out = s.eval_request("WithCleanup[InputString[\"p: \"],Get[\"Dep549`\"]]");
    assert!(matches!(out,Err(f) if matches!(f.error,SessionRequestError::InputRequired{..})));
    cancel.cancel();
    let out = s.resume_eval_request_with_input("unused").unwrap();
    assert_eq!(format!("{:?}", out.termination), "Some(Cancelled)");
    assert!(
        s.pending_input_request().is_none(),
        "cancellation cleanup leaked input owner"
    );
    let q = s.eval_request("InputString[\"q: \"]");
    assert!(matches!(q,Err(f) if matches!(f.error,SessionRequestError::InputRequired{..})));
    s.resume_eval_request_with_input("two").unwrap();
    assert_eq!(s.eval_request("1+1").unwrap().value.to_string(), "2");
}
macro_rules! red_cancel {
    ($name:ident,$source:expr) => {
        #[test]
        fn $name() {
            let (mut s,_)=session(&[("Outer549.rum","Get[\"Dep549`\"]\n99")]);
            let cancel=s.request_cancel_handle();
            let initial=s.eval_request($source);
            assert!(matches!(initial,Err(f) if matches!(f.error,SessionRequestError::InputRequired{..})),"must suspend before cancellation");
            cancel.cancel();
            let result=s.resume_eval_request_with_input("unused").unwrap();
            assert_eq!(format!("{:?}",result.termination),"Some(Cancelled)");
            assert!(s.pending_input_request().is_none());
            assert_eq!(s.eval_request("1+1").unwrap().value.to_string(),"2");
            assert!(s.eval_request("$InputFileName").unwrap().value.to_string()=="\"\"");
        }
    }
}
red_cancel!(red_get_cancel_restores, "Get[\"Dep549`\"]");
red_cancel!(red_needs_cancel_restores, "Needs[\"Dep549`\"]");
red_cancel!(
    red_begin_cancel_restores,
    "BeginPackage[\"Main549`\",{\"Dep549`\"}]"
);
red_cancel!(red_nested_cancel_restores, "Get[\"Outer549`\"]");
red_cancel!(
    red_stub_cancel_restores,
    "DeclarePackage[\"Dep549`\",{\"foo\"}];Dep549`foo"
);

macro_rules! red_cancel_twice {
    ($name:ident,$source:expr) => {
        #[test]
        fn $name() {
            for stop in [0,1] {
                let (mut s,_)=session(&[("Twice549.rum","Global`marker=10\nAppendTo[Global`log,1]\nInputString[\"first: \"]\nAppendTo[Global`log,3]\nInputString[\"second: \"]\nTwice549`foo=42\n77")]);
                let cancel=s.request_cancel_handle();
                let mut result=s.eval_request($source);
                assert!(matches!(&result,Err(f) if matches!(f.error,SessionRequestError::InputRequired{..})),"first load suspension missing");
                if stop==1 {
                    result=s.resume_eval_request_with_input("one");
                    assert!(matches!(&result,Err(f) if matches!(f.error,SessionRequestError::InputRequired{..})),"second load suspension missing");
                }
                cancel.cancel();
                let out=s.resume_eval_request_with_input("unused").unwrap();
                assert_eq!(format!("{:?}",out.termination),"Some(Cancelled)");
                assert_eq!(s.eval_request("{Global`log,Global`marker,$InputFileName}").unwrap().value.to_string(),"List[List[], 0, \"\"]");
                assert_eq!(s.eval_request("1+1").unwrap().value.to_string(),"2");
            }
        }
    }
}
red_cancel_twice!(red_get_cancel_at_both_inputs, "Get[\"Twice549`\"]");
red_cancel_twice!(red_needs_cancel_at_both_inputs, "Needs[\"Twice549`\"]");
red_cancel_twice!(
    red_begin_cancel_at_both_inputs,
    "BeginPackage[\"Main549`\",{\"Twice549`\"}]"
);
red_cancel_twice!(
    red_stub_cancel_at_both_inputs,
    "DeclarePackage[\"Twice549`\",{\"foo\"}];Twice549`foo"
);
#[test]
fn green_best_effort_discards_and_next_input() {
    for source in [
        "ValueQ[InputString[\"a: \"]];InputString[\"q: \"]",
        "Monitor[77,InputString[\"a: \"]];InputString[\"q: \"]",
        "Monitor[1+2,0,EvaluationMonitor:>InputString[\"a: \"]];InputString[\"q: \"]",
        "Monitor[1+2,0,StepMonitor:>InputString[\"a: \"]];InputString[\"q: \"]",
        "Monitor[77,WithCleanup[InputString[\"a: \"],InputString[\"cleanup: \"]]];InputString[\"q: \"]",
    ] {
        let (mut s, _) = session(&[]);
        let failure = s.eval_request(source).unwrap_err();
        assert!(
            matches!(failure.error,SessionRequestError::InputRequired{request} if request.prompt=="q: ")
        );
        assert!(
            s.eval_request("1+1").is_err(),
            "active q still rejects overlap"
        );
        s.resume_eval_request_with_input("done").unwrap();
        assert!(s.pending_input_request().is_none());
        assert_eq!(s.eval_request("1+1").unwrap().value.to_string(), "2");
    }
}
