use super::support::compare;

#[test]
fn cases_heads_option() {
    compare(
        r###"Cases-Heads-option"###,
        r###"Unprotect[Cases];SetOptions[Cases,Heads:>(AppendTo[log,2];InputString["p: "];False)];Cases[{1,2,3},_Integer]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn deletecases_heads_option() {
    compare(
        r###"DeleteCases-Heads-option"###,
        r###"Unprotect[DeleteCases];SetOptions[DeleteCases,Heads:>(AppendTo[log,2];InputString["p: "];False)];DeleteCases[{1,2,3},2]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn count_heads_option() {
    compare(
        r###"Count-Heads-option"###,
        r###"Unprotect[Count];SetOptions[Count,Heads:>(AppendTo[log,2];InputString["p: "];False)];Count[{1,2,3},_Integer]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn memberq_heads_option() {
    compare(
        r###"MemberQ-Heads-option"###,
        r###"Unprotect[MemberQ];SetOptions[MemberQ,Heads:>(AppendTo[log,2];InputString["p: "];False)];MemberQ[{1,2,3},2]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn freeq_heads_option() {
    compare(
        r###"FreeQ-Heads-option"###,
        r###"Unprotect[FreeQ];SetOptions[FreeQ,Heads:>(AppendTo[log,2];InputString["p: "];True)];FreeQ[{1,2,3},2]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn position_heads_option() {
    compare(
        r###"Position-Heads-option"###,
        r###"Unprotect[Position];SetOptions[Position,Heads:>(AppendTo[log,2];InputString["p: "];True)];Position[{1,2,3},2]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn level_heads_option() {
    compare(
        r###"Level-Heads-option"###,
        r###"Unprotect[Level];SetOptions[Level,Heads:>(AppendTo[log,2];InputString["p: "];False)];Level[{1,2,3},{1}]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn depth_heads_option() {
    compare(
        r###"Depth-Heads-option"###,
        r###"Unprotect[Depth];SetOptions[Depth,Heads:>(AppendTo[log,2];InputString["p: "];False)];Depth[{1,{2},3}]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn replace_heads_option() {
    compare(
        r###"Replace-Heads-option"###,
        r###"Unprotect[Replace];SetOptions[Replace,Heads:>(AppendTo[log,2];InputString["p: "];False)];Replace[{1,2,3},2->x,{1}]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn trace_heads_option() {
    compare(
        r###"Trace-Heads-option"###,
        r###"Unprotect[Trace];SetOptions[Trace,Heads:>(AppendTo[log,2];InputString["p: "];False)];Trace[1+2]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn cases_heads_after_target() {
    compare(
        r###"Cases-Heads-after-target"###,
        r###"Unprotect[Cases];SetOptions[Cases,Heads:>(AppendTo[log,2];InputString["option: "];False)];Cases[(AppendTo[log,1];InputString["target: "];{1,2,3}),2]"###,
        &[r###"target"###, r###"two"###],
        &[r###"target: "###, r###"option: "###],
    );
}

#[test]
fn deletecases_heads_after_target() {
    compare(
        r###"DeleteCases-Heads-after-target"###,
        r###"Unprotect[DeleteCases];SetOptions[DeleteCases,Heads:>(AppendTo[log,2];InputString["option: "];False)];DeleteCases[(AppendTo[log,1];InputString["target: "];{1,2,3}),2]"###,
        &[r###"target"###, r###"two"###],
        &[r###"target: "###, r###"option: "###],
    );
}

#[test]
fn cases_heads_option_twice576() {
    compare(
        r###"Cases-Heads-option-twice576"###,
        r###"Unprotect[Cases];SetOptions[Cases,Heads:>(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];False)];Cases[{1,2,3},_Integer]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn deletecases_heads_option_twice576() {
    compare(
        r###"DeleteCases-Heads-option-twice576"###,
        r###"Unprotect[DeleteCases];SetOptions[DeleteCases,Heads:>(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];False)];DeleteCases[{1,2,3},2]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn count_heads_option_twice576() {
    compare(
        r###"Count-Heads-option-twice576"###,
        r###"Unprotect[Count];SetOptions[Count,Heads:>(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];False)];Count[{1,2,3},_Integer]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn memberq_heads_option_twice576() {
    compare(
        r###"MemberQ-Heads-option-twice576"###,
        r###"Unprotect[MemberQ];SetOptions[MemberQ,Heads:>(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];False)];MemberQ[{1,2,3},2]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn freeq_heads_option_twice576() {
    compare(
        r###"FreeQ-Heads-option-twice576"###,
        r###"Unprotect[FreeQ];SetOptions[FreeQ,Heads:>(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];True)];FreeQ[{1,2,3},2]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn position_heads_option_twice576() {
    compare(
        r###"Position-Heads-option-twice576"###,
        r###"Unprotect[Position];SetOptions[Position,Heads:>(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];True)];Position[{1,2,3},2]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn level_heads_option_twice576() {
    compare(
        r###"Level-Heads-option-twice576"###,
        r###"Unprotect[Level];SetOptions[Level,Heads:>(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];False)];Level[{1,2,3},{1}]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn depth_heads_option_twice576() {
    compare(
        r###"Depth-Heads-option-twice576"###,
        r###"Unprotect[Depth];SetOptions[Depth,Heads:>(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];False)];Depth[{1,{2},3}]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn replace_heads_option_twice576() {
    compare(
        r###"Replace-Heads-option-twice576"###,
        r###"Unprotect[Replace];SetOptions[Replace,Heads:>(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];False)];Replace[{1,2,3},2->x,{1}]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn trace_heads_option_twice576() {
    compare(
        r###"Trace-Heads-option-twice576"###,
        r###"Unprotect[Trace];SetOptions[Trace,Heads:>(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];False)];Trace[1+2]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn count_heads_after_target576() {
    compare(
        r###"Count-Heads-after-target576"###,
        r###"Unprotect[Count];SetOptions[Count,Heads:>(AppendTo[log,2];InputString["option: "];False)];Count[(AppendTo[log,1];InputString["target: "];{1,2,3}),_Integer]"###,
        &[r###"a"###, r###"b"###],
        &[r###"target: "###, r###"option: "###],
    );
}

#[test]
fn memberq_heads_after_target576() {
    compare(
        r###"MemberQ-Heads-after-target576"###,
        r###"Unprotect[MemberQ];SetOptions[MemberQ,Heads:>(AppendTo[log,2];InputString["option: "];False)];MemberQ[(AppendTo[log,1];InputString["target: "];{1,2,3}),2]"###,
        &[r###"a"###, r###"b"###],
        &[r###"target: "###, r###"option: "###],
    );
}

#[test]
fn freeq_heads_after_target576() {
    compare(
        r###"FreeQ-Heads-after-target576"###,
        r###"Unprotect[FreeQ];SetOptions[FreeQ,Heads:>(AppendTo[log,2];InputString["option: "];True)];FreeQ[(AppendTo[log,1];InputString["target: "];{1,2,3}),2]"###,
        &[r###"a"###, r###"b"###],
        &[r###"target: "###, r###"option: "###],
    );
}

#[test]
fn position_heads_after_target576() {
    compare(
        r###"Position-Heads-after-target576"###,
        r###"Unprotect[Position];SetOptions[Position,Heads:>(AppendTo[log,2];InputString["option: "];True)];Position[(AppendTo[log,1];InputString["target: "];{1,2,3}),2]"###,
        &[r###"a"###, r###"b"###],
        &[r###"target: "###, r###"option: "###],
    );
}

#[test]
fn replace_heads_after_target576() {
    compare(
        r###"Replace-Heads-after-target576"###,
        r###"Unprotect[Replace];SetOptions[Replace,Heads:>(AppendTo[log,2];InputString["option: "];False)];Replace[(AppendTo[log,1];InputString["target: "];{1,2,3}),2->x,{1}]"###,
        &[r###"a"###, r###"b"###],
        &[r###"target: "###, r###"option: "###],
    );
}

#[test]
fn count_option_error_after_resume() {
    compare(
        r###"Count-option-error-after-resume"###,
        r###"Unprotect[Count];SetOptions[Count,Heads:>(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["option: "];AppendTo[log,2];Print["after"];Message[probe::tag];Throw["g5 boom"])];Count[{1,2,3},_Integer]"###,
        &[r###"a"###],
        &[r###"option: "###],
    );
}

#[test]
fn arrayfilter_unread_default() {
    compare(
        r###"ArrayFilter-unread-default"###,
        r###"Unprotect[ArrayFilter];SetOptions[ArrayFilter,Padding:>(AppendTo[log,99];InputString["unread: "];0)];ArrayFilter[Total,{1,2,3},1,Padding->0]"###,
        &[],
        &[],
    );
}

#[test]
fn testreport_unread_default() {
    compare(
        r###"TestReport-unread-default"###,
        r###"Unprotect[TestReport];SetOptions[TestReport,SameTest->v];v:=(AppendTo[log,99];InputString["unread: "];SameQ);TestReport[{TestCreate[1,1]}]["TestsSucceededCount"]"###,
        &[],
        &[],
    );
}

#[test]
fn names_early_invalid_unread_later() {
    compare(
        r###"Names-early-invalid-unread-later"###,
        r###"Unprotect[Names];SetOptions[Names,SpellingCorrection->True,ResolveContextAliases->v,IgnoreCase->w];v:=(AppendTo[log,98];InputString["unread-alias: "];True);w:=(AppendTo[log,99];InputString["unread-case: "];False);Names["Global`never548"]"###,
        &[],
        &[],
    );
}

#[test]
fn tracedetail_heads_option() {
    compare(
        r###"TraceDetail-Heads-option"###,
        r###"Unprotect[TraceDetail];SetOptions[TraceDetail,Heads:>(AppendTo[log,2];InputString["p: "];False)];TraceDetail[1+2]"###,
        &[r###"a"###],
        &[r###"p: "###],
    );
}

#[test]
fn traceprint_heads_option() {
    compare(
        r###"TracePrint-Heads-option"###,
        r###"Unprotect[TracePrint];SetOptions[TracePrint,Heads:>(AppendTo[log,2];InputString["p: "];False)];TracePrint[1+2]"###,
        &[r###"a"###],
        &[r###"p: "###],
    );
}

#[test]
fn cases_heads_after_target_twice577() {
    compare(
        r###"Cases-Heads-after-target-twice577"###,
        r###"Unprotect[Cases];SetOptions[Cases,Heads:>(AppendTo[log,2];InputString["option: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];False)];Cases[(AppendTo[log,1];InputString["target: "];{1,2,3}),2]"###,
        &[r###"target"###, r###"two"###, r###"four"###],
        &[r###"target: "###, r###"option: "###, r###"second: "###],
    );
}

#[test]
fn deletecases_heads_after_target_twice577() {
    compare(
        r###"DeleteCases-Heads-after-target-twice577"###,
        r###"Unprotect[DeleteCases];SetOptions[DeleteCases,Heads:>(AppendTo[log,2];InputString["option: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];False)];DeleteCases[(AppendTo[log,1];InputString["target: "];{1,2,3}),2]"###,
        &[r###"target"###, r###"two"###, r###"four"###],
        &[r###"target: "###, r###"option: "###, r###"second: "###],
    );
}

#[test]
fn default_before_override_count() {
    compare(
        r###"default_before_override_count"###,
        r###"Unprotect[Count];SetOptions[Count,Heads:>(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["option: "];AppendTo[log,2];Print["after"];Message[probe::tag];False)];Count[{1,2,3},_Integer,Heads->True]"###,
        &[r###"yes"###],
        &[r###"option: "###],
    );
}

#[test]
fn default_before_override_cases() {
    compare(
        r###"default_before_override_cases"###,
        r###"Unprotect[Cases];SetOptions[Cases,Heads:>(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["option: "];AppendTo[log,2];Print["after"];Message[probe::tag];False)];Cases[{1,2,3},_Integer,Heads->True]"###,
        &[r###"yes"###],
        &[r###"option: "###],
    );
}

#[test]
fn default_before_override_deletecases() {
    compare(
        r###"default_before_override_deletecases"###,
        r###"Unprotect[DeleteCases];SetOptions[DeleteCases,Heads:>(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["option: "];AppendTo[log,2];Print["after"];Message[probe::tag];False)];DeleteCases[{1,2,3},2,Heads->True]"###,
        &[r###"yes"###],
        &[r###"option: "###],
    );
}

#[test]
fn default_before_override_memberq() {
    compare(
        r###"default_before_override_memberq"###,
        r###"Unprotect[MemberQ];SetOptions[MemberQ,Heads:>(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["option: "];AppendTo[log,2];Print["after"];Message[probe::tag];False)];MemberQ[{1,2,3},2,Heads->True]"###,
        &[r###"yes"###],
        &[r###"option: "###],
    );
}

#[test]
fn default_before_override_freeq() {
    compare(
        r###"default_before_override_freeq"###,
        r###"Unprotect[FreeQ];SetOptions[FreeQ,Heads:>(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["option: "];AppendTo[log,2];Print["after"];Message[probe::tag];False)];FreeQ[{1,2,3},2,Heads->True]"###,
        &[r###"yes"###],
        &[r###"option: "###],
    );
}

#[test]
fn default_before_override_position() {
    compare(
        r###"default_before_override_position"###,
        r###"Unprotect[Position];SetOptions[Position,Heads:>(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["option: "];AppendTo[log,2];Print["after"];Message[probe::tag];False)];Position[{1,2,3},2,Heads->False]"###,
        &[r###"yes"###],
        &[r###"option: "###],
    );
}

#[test]
fn default_before_override_replace() {
    compare(
        r###"default_before_override_replace"###,
        r###"Unprotect[Replace];SetOptions[Replace,Heads:>(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["option: "];AppendTo[log,2];Print["after"];Message[probe::tag];False)];Replace[{1,2,3},2->x,{1},Heads->True]"###,
        &[r###"yes"###],
        &[r###"option: "###],
    );
}

#[test]
fn default_before_override_level() {
    compare(
        r###"default_before_override_level"###,
        r###"Unprotect[Level];SetOptions[Level,Heads:>(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["option: "];AppendTo[log,2];Print["after"];Message[probe::tag];False)];Level[{1,2,3},{1},Heads->True]"###,
        &[r###"yes"###],
        &[r###"option: "###],
    );
}

#[test]
fn default_before_override_depth() {
    compare(
        r###"default_before_override_depth"###,
        r###"Unprotect[Depth];SetOptions[Depth,Heads:>(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["option: "];AppendTo[log,2];Print["after"];Message[probe::tag];False)];Depth[{1,{2},3},Heads->True]"###,
        &[r###"yes"###],
        &[r###"option: "###],
    );
}

#[test]
fn default_before_override_trace() {
    compare(
        r###"default_before_override_trace"###,
        r###"Unprotect[Trace];SetOptions[Trace,Heads:>(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["option: "];AppendTo[log,2];Print["after"];Message[probe::tag];False)];Trace[1+2,_,Heads->True]"###,
        &[r###"yes"###],
        &[r###"option: "###],
    );
}

#[test]
fn depth_before_invalid_arity() {
    // Invalid residual calls can be revisited by fixed-point evaluation.
    // Reset the default after its first read to isolate read-before-arity order.
    compare(
        r###"depth_before_invalid_arity"###,
        r###"Unprotect[Depth];SetOptions[Depth,Heads:>(SetOptions[Depth,Heads->False];AppendTo[log,1];Print["before"];Message[probe::tag];InputString["option: "];AppendTo[log,2];Print["after"];Message[probe::tag];False)];Depth[]"###,
        &[r###"yes"###],
        &[r###"option: "###],
    );
}

#[test]
fn trace_before_invalid_arity() {
    // Keep the invalid residual inert after observing its first default read.
    compare(
        r###"trace_before_invalid_arity"###,
        r###"Unprotect[Trace];SetOptions[Trace,Heads:>(SetOptions[Trace,Heads->False];AppendTo[log,1];Print["before"];Message[probe::tag];InputString["option: "];AppendTo[log,2];Print["after"];Message[probe::tag];False)];Trace[]"###,
        &[r###"yes"###],
        &[r###"option: "###],
    );
}

#[test]
fn trace_body_after_option() {
    compare(
        r###"trace_body_after_option"###,
        r###"Unprotect[Trace];SetOptions[Trace,Heads:>(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["option: "];AppendTo[log,2];Print["after"];Message[probe::tag];False)];Trace[(AppendTo[log,3];InputString["body: "];AppendTo[log,4];1+2)]"###,
        &[r###"yes"###, r###"yes"###],
        &[r###"option: "###, r###"body: "###],
    );
}
