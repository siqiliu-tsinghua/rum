use super::support::compare;

#[test]
fn oracle_monitor_options() {
    compare(
        r###"Monitor-options"###,
        r###"Monitor[(AppendTo[log,3];77),0,(AppendTo[log,1];InputString["p: "];AppendTo[log,2];EvaluationMonitor:>Null)]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn oracle_names_spellingcorrection() {
    compare(
        r###"Names-SpellingCorrection"###,
        r###"Unprotect[Names];SetOptions[Names,SpellingCorrection->v];v:=(AppendTo[log,2];InputString["p: "];False);Names["Global`never548"]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn oracle_names_resolvecontextaliases() {
    compare(
        r###"Names-ResolveContextAliases"###,
        r###"Unprotect[Names];SetOptions[Names,ResolveContextAliases->v];v:=(AppendTo[log,2];InputString["p: "];True);Names["Global`never548"]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn oracle_names_ignorecase() {
    compare(
        r###"Names-IgnoreCase"###,
        r###"Unprotect[Names];SetOptions[Names,IgnoreCase->v];v:=(AppendTo[log,2];InputString["p: "];False);Names["Global`never548"]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn oracle_monitor_evaluationmonitor() {
    compare(
        r###"Monitor-EvaluationMonitor"###,
        r###"Unprotect[Monitor];SetOptions[Monitor,EvaluationMonitor->v];v:=(AppendTo[log,2];InputString["p: "];None);Monitor[(AppendTo[log,3];77),0]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn oracle_monitor_stepmonitor() {
    compare(
        r###"Monitor-StepMonitor"###,
        r###"Unprotect[Monitor];SetOptions[Monitor,StepMonitor->v];v:=(AppendTo[log,2];InputString["p: "];None);Monitor[(AppendTo[log,3];77),0]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn oracle_testreport_option_sametest() {
    compare(
        r###"TestReport-option-SameTest"###,
        r###"Unprotect[TestReport];SetOptions[TestReport,SameTest->v];v:=(AppendTo[log,2];InputString["p: "];SameQ);TestReport[{TestCreate[1,1]},"TestsSucceededCount",{}]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn oracle_testreport_option_timeconstraint() {
    compare(
        r###"TestReport-option-TimeConstraint"###,
        r###"Unprotect[TestReport];SetOptions[TestReport,TimeConstraint->v];v:=(AppendTo[log,2];InputString["p: "];Infinity);TestReport[{TestCreate[1,1]},"TestsSucceededCount",{}]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn oracle_testreport_option_memoryconstraint() {
    compare(
        r###"TestReport-option-MemoryConstraint"###,
        r###"Unprotect[TestReport];SetOptions[TestReport,MemoryConstraint->v];v:=(AppendTo[log,2];InputString["p: "];Infinity);TestReport[{TestCreate[1,1]},"TestsSucceededCount",{}]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn oracle_dumpsave_option() {
    compare(
        r###"DumpSave-option"###,
        r###"Unprotect[DumpSave];SetOptions[DumpSave,CompressionLevel->v];v:=(AppendTo[log,2];InputString["p: "];1);DumpSave[".temp/brief-578/mock.rum",{}]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn oracle_testreport_two_args() {
    compare(
        r###"TestReport-two-args"###,
        r###"Unprotect[TestReport];SetOptions[TestReport,SameTest->v];v:=(AppendTo[log,2];InputString["p: "];SameQ);TestReport[{TestCreate[1,1]},{}]["TestsSucceededCount"]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn oracle_monitor_options_twice() {
    compare(
        r###"Monitor-options-twice"###,
        r###"Monitor[(AppendTo[log,5];77),0,(AppendTo[log,2];InputString["p: "];EvaluationMonitor:>Null),(AppendTo[log,4];InputString["p: "];StepMonitor:>Null)]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"p: "###],
    );
}

#[test]
fn oracle_monitor_options_twice576() {
    compare(
        r###"Monitor-options-twice576"###,
        r###"Monitor[(AppendTo[log,3];77),0,(AppendTo[log,1];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];AppendTo[log,2];EvaluationMonitor:>Null)]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn oracle_names_spellingcorrection_twice576() {
    compare(
        r###"Names-SpellingCorrection-twice576"###,
        r###"Unprotect[Names];SetOptions[Names,SpellingCorrection->v];v:=(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];False);Names["Global`never548"]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn oracle_names_resolvecontextaliases_twice576() {
    compare(
        r###"Names-ResolveContextAliases-twice576"###,
        r###"Unprotect[Names];SetOptions[Names,ResolveContextAliases->v];v:=(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];True);Names["Global`never548"]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn oracle_names_ignorecase_twice576() {
    compare(
        r###"Names-IgnoreCase-twice576"###,
        r###"Unprotect[Names];SetOptions[Names,IgnoreCase->v];v:=(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];False);Names["Global`never548"]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn oracle_monitor_evaluationmonitor_twice576() {
    compare(
        r###"Monitor-EvaluationMonitor-twice576"###,
        r###"Unprotect[Monitor];SetOptions[Monitor,EvaluationMonitor->v];v:=(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];None);Monitor[(AppendTo[log,3];77),0]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn oracle_monitor_stepmonitor_twice576() {
    compare(
        r###"Monitor-StepMonitor-twice576"###,
        r###"Unprotect[Monitor];SetOptions[Monitor,StepMonitor->v];v:=(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];None);Monitor[(AppendTo[log,3];77),0]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn oracle_testreport_option_sametest_twice576() {
    compare(
        r###"TestReport-option-SameTest-twice576"###,
        r###"Unprotect[TestReport];SetOptions[TestReport,SameTest->v];v:=(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];SameQ);TestReport[{TestCreate[1,1]},"TestsSucceededCount",{}]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn oracle_testreport_option_timeconstraint_twice576() {
    compare(
        r###"TestReport-option-TimeConstraint-twice576"###,
        r###"Unprotect[TestReport];SetOptions[TestReport,TimeConstraint->v];v:=(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];Infinity);TestReport[{TestCreate[1,1]},"TestsSucceededCount",{}]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn oracle_testreport_option_memoryconstraint_twice576() {
    compare(
        r###"TestReport-option-MemoryConstraint-twice576"###,
        r###"Unprotect[TestReport];SetOptions[TestReport,MemoryConstraint->v];v:=(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];Infinity);TestReport[{TestCreate[1,1]},"TestsSucceededCount",{}]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn oracle_dumpsave_option_twice576() {
    compare(
        r###"DumpSave-option-twice576"###,
        r###"Unprotect[DumpSave];SetOptions[DumpSave,CompressionLevel->v];v:=(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];1);DumpSave[".temp/brief-578/mock.rum",{}]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn oracle_testreport_two_args_twice576() {
    compare(
        r###"TestReport-two-args-twice576"###,
        r###"Unprotect[TestReport];SetOptions[TestReport,SameTest->v];v:=(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];SameQ);TestReport[{TestCreate[1,1]},{}]["TestsSucceededCount"]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn oracle_monitor_runtime_two_options() {
    compare(
        r###"Monitor-runtime-two-options"###,
        r###"Unprotect[Monitor];SetOptions[Monitor,EvaluationMonitor->v,StepMonitor->w];v:=(AppendTo[log,1];InputString["eval: "];AppendTo[log,2];None);w:=(AppendTo[log,3];InputString["step: "];AppendTo[log,4];None);Monitor[(AppendTo[log,5];77),0]"###,
        &[r###"a"###, r###"b"###],
        &[r###"eval: "###, r###"step: "###],
    );
}

#[test]
fn oracle_names_two_options() {
    compare(
        r###"Names-two-options"###,
        r###"Unprotect[Names];SetOptions[Names,SpellingCorrection->v,ResolveContextAliases->w];v:=(AppendTo[log,1];InputString["spell: "];AppendTo[log,2];False);w:=(AppendTo[log,3];InputString["alias: "];AppendTo[log,4];True);Names["Global`never548"]"###,
        &[r###"a"###, r###"b"###],
        &[r###"spell: "###, r###"alias: "###],
    );
}

#[test]
fn oracle_testreport_three_options() {
    compare(
        r###"TestReport-three-options"###,
        r###"Unprotect[TestReport];SetOptions[TestReport,SameTest->v,TimeConstraint->w,MemoryConstraint->z];v:=(AppendTo[log,1];InputString["same: "];SameQ);w:=(AppendTo[log,2];InputString["time: "];Infinity);z:=(AppendTo[log,3];InputString["memory: "];Infinity);TestReport[{TestCreate[(AppendTo[log,4];1),1]},"TestsSucceededCount",{}]"###,
        &[r###"a"###, r###"b"###, r###"c"###],
        &[r###"same: "###, r###"time: "###, r###"memory: "###],
    );
}

#[test]
fn oracle_names_later_default_mutation() {
    compare(
        r###"Names-later-default-mutation"###,
        r###"Unprotect[Names];SetOptions[Names,SpellingCorrection->v];v:=(AppendTo[log,1];InputString["spell: "];SetOptions[Names,SpellingCorrection->False,ResolveContextAliases->fresh576];fresh576:=(AppendTo[log,3];InputString["new-alias: "];True);AppendTo[log,2];False);Names["Global`never548"]"###,
        &[r###"a"###, r###"b"###],
        &[r###"spell: "###, r###"new-alias: "###],
    );
}

#[test]
fn oracle_names_later_option_error_after_resume() {
    compare(
        r###"Names-later-option-error-after-resume"###,
        r###"Unprotect[Names];SetOptions[Names,SpellingCorrection->v,ResolveContextAliases->w];v:=(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["spell: "];False);w:=(AppendTo[log,2];Print["after"];Message[probe::tag];Throw["g5 boom"]);Names["Global`never548"]"###,
        &[r###"a"###],
        &[r###"spell: "###],
    );
}

#[test]
fn oracle_monitor_option_streams() {
    compare(
        r###"Monitor-option-streams"###,
        r###"Monitor[(AppendTo[log,3];Print["body"];Message[probe::tag];77),0,(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["option: "];AppendTo[log,2];Print["after"];Message[probe::tag];EvaluationMonitor:>Null)]"###,
        &[r###"a"###],
        &[r###"option: "###],
    );
}

#[test]
fn oracle_testreport_unread_default() {
    compare(
        r###"TestReport-unread-default"###,
        r###"Unprotect[TestReport];SetOptions[TestReport,SameTest->v];v:=(AppendTo[log,99];InputString["unread: "];SameQ);TestReport[{TestCreate[1,1]}]["TestsSucceededCount"]"###,
        &[],
        &[],
    );
}

#[test]
fn oracle_names_early_invalid_unread_later() {
    compare(
        r###"Names-early-invalid-unread-later"###,
        r###"Unprotect[Names];SetOptions[Names,SpellingCorrection->True,ResolveContextAliases->v,IgnoreCase->w];v:=(AppendTo[log,98];InputString["unread-alias: "];True);w:=(AppendTo[log,99];InputString["unread-case: "];False);Names["Global`never548"]"###,
        &[],
        &[],
    );
}

#[test]
fn monitor_runtime_two_options_preserve_ordered_streams() {
    compare(
        "Monitor-runtime-two-options-streams",
        r#"Unprotect[Monitor];SetOptions[Monitor,EvaluationMonitor->v,StepMonitor->w];v:=(AppendTo[log,1];Print["evaluation-before"];Message[probe::tag];InputString["eval: "];AppendTo[log,2];Print["evaluation-after"];None);w:=(AppendTo[log,3];Print["step-before"];Message[probe::tag];InputString["step: "];AppendTo[log,4];Print["step-after"];None);Monitor[(AppendTo[log,5];Print["body"];Message[probe::tag];77),0]"#,
        &["a", "b"],
        &["eval: ", "step: "],
    );
}

#[test]
fn testreport_property_and_invalid_option_forms_leave_defaults_unread() {
    for call in [
        r#"TestReport[{TestCreate[1,1]},"TestsSucceededCount"]"#,
        r#"TestReport[{TestCreate[1,1]},"TestsSucceededCount",invalid]"#,
    ] {
        let source = format!(
            r#"Unprotect[TestReport];SetOptions[TestReport,SameTest->v];v:=(AppendTo[log,99];InputString["unread: "];SameQ);{call}"#
        );
        compare("TestReport-unread-forms", &source, &[], &[]);
    }
}

#[test]
fn names_invalid_alias_leaves_ignorecase_unread() {
    compare(
        "Names-invalid-alias-unread-later",
        r#"Unprotect[Names];SetOptions[Names,ResolveContextAliases->False,IgnoreCase->v];v:=(AppendTo[log,99];InputString["unread: "];False);Names["Global`never578"]"#,
        &[],
        &[],
    );
}

#[test]
fn names_pattern_list_is_reparsed_without_repeating_prior_work() {
    compare(
        "Names-pattern-list",
        r#"Global`pattern578=1;Unprotect[Names];SetOptions[Names,IgnoreCase->v];v:=(AppendTo[log,2];InputString["case: "];False);Names[(AppendTo[log,1];{RegularExpression["Global`pattern578"],StringExpression["Global`",Alternatives["pattern578","absent578"]]})]"#,
        &["a"],
        &["case: "],
    );
}

#[test]
fn runtime_delayed_values_stay_held() {
    compare(
        "Names-held-runtime-default",
        r#"Unprotect[Names];SetOptions[Names,SpellingCorrection:>(AppendTo[log,99];InputString["unread: "];False)];Names["Global`never578"]"#,
        &[],
        &[],
    );
    compare(
        "Monitor-held-explicit",
        r#"Monitor[77,0,EvaluationMonitor:>Null]"#,
        &[],
        &[],
    );
}

#[derive(Clone, Default)]
struct MemoryFiles(std::sync::Arc<std::sync::Mutex<Vec<(String, String)>>>);
impl rum_runtime::kernel_api::BoundedDestruction for MemoryFiles {}
impl rum_runtime::kernel_api::FileIo for MemoryFiles {
    fn read_to_string(
        &self,
        _: &std::path::Path,
    ) -> Result<String, rum_runtime::kernel_api::FileIoError> {
        Err(rum_runtime::kernel_api::FileIoError::new("absent"))
    }
    fn probe(&self, _: &std::path::Path) -> rum_runtime::kernel_api::FileProbe {
        rum_runtime::kernel_api::FileProbe::missing()
    }
    fn write_string(
        &self,
        path: &std::path::Path,
        content: &str,
        _: rum_runtime::kernel_api::FileWriteMode,
    ) -> Result<(), rum_runtime::kernel_api::FileIoError> {
        self.0
            .lock()
            .unwrap()
            .push((path.display().to_string(), content.to_owned()));
        Ok(())
    }
}

#[test]
fn dumpsave_writes_once_after_read_and_context_change() {
    let mut runs = Vec::new();
    for queued in [false, true] {
        let mut s = super::support::session();
        let files = MemoryFiles::default();
        s.set_file_io(Some(rum_runtime::kernel_api::FileIoProvider::new(
            files.clone(),
        )));
        s.eval_request(r#"Before578`a=7;Unprotect[DumpSave];SetOptions[DumpSave,CompressionLevel->Global`compression578];Global`compression578:=(AppendTo[Global`log,1];InputString["compression: "];Begin["After578`"];AppendTo[Global`log,2];1);Begin["Before578`"]"#).unwrap();
        if queued {
            s.queue_input_response("a");
        }
        let mut out = s.eval_request(r#"DumpSave[".temp/brief-578/mock.rum",{"Before578`"}]"#);
        if !queued {
            let rum_runtime::SessionRequestError::InputRequired { request } =
                &out.as_ref().unwrap_err().error
            else {
                panic!("must suspend");
            };
            assert_eq!(request.prompt, "compression: ");
            assert!(files.0.lock().unwrap().is_empty());
            out = s.resume_eval_request_with_input("a");
        }
        let result = super::support::finish(&mut s, out);
        assert_eq!(result.0.value, "List[\"Before578`a\"]");
        assert_eq!(result.1, "List[1, 2]");
        assert_eq!(
            s.eval_request("$Context").unwrap().value.to_string(),
            "\"After578`\""
        );
        let written = files.0.lock().unwrap().clone();
        assert_eq!(written.len(), 1);
        assert!(written[0].1.contains("Before578`a"));
        runs.push((result, written));
    }
    assert_eq!(runs[0], runs[1]);
}

#[test]
fn save_leaves_compression_unread() {
    let mut s = super::support::session();
    let files = MemoryFiles::default();
    s.set_file_io(Some(rum_runtime::kernel_api::FileIoProvider::new(
        files.clone(),
    )));
    let out = s.eval_request(r#"Unprotect[DumpSave];SetOptions[DumpSave,CompressionLevel->v];v:=(AppendTo[log,99];InputString["unread: "];1);Save[".temp/brief-578/mock.rum",{}]"#);
    let (view, log) = super::support::finish(&mut s, out);
    assert_eq!(view.value, "Null");
    assert_eq!(log, "List[]");
    assert_eq!(files.0.lock().unwrap().len(), 1);
}
