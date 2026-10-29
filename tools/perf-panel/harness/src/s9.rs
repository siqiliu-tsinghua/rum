use crate::s5::Workload;

pub fn workload(label: &str) -> Option<Workload> {
    Some(match label {
        "s9-checkabort-option" => Workload {
            setup: "",
            body: "CheckAbort[1, 2, PropagateAborts -> (InputString[\"opt\"]; False)]",
            item: "1",
            prompts: &["opt"],
        },
        "s9-checkabort-primary" => Workload {
            setup: "",
            body: "CheckAbort[(InputString[\"primary\"]; 1), 2]",
            item: "1",
            prompts: &["primary"],
        },
        "s9-checkabort-abort" => Workload {
            setup: "",
            body: "CheckAbort[(InputString[\"primary\"]; Abort[]), 7]",
            item: "7",
            prompts: &["primary"],
        },
        "s9-abortprotect-body" => Workload {
            setup: "",
            body: "AbortProtect[(InputString[\"body\"]; 3)]",
            item: "3",
            prompts: &["body"],
        },
        "s9-checkabort-pending-fallback" => Workload {
            setup: "",
            body: "AbortProtect[CheckAbort[(Abort[]; 1), (InputString[\"fallback\"]; 9)]]",
            item: "9",
            prompts: &["fallback"],
        },
        "s9-checkabort-fallback" => Workload {
            setup: "",
            body: "CheckAbort[Abort[], (InputString[\"fallback\"]; 8)]",
            item: "8",
            prompts: &["fallback"],
        },
        "s9-assert-function-lookup" => Workload {
            setup: "On[Assert]; $AssertFunction := (InputString[\"af\"]; None);",
            body: "Assert[True]",
            item: "Null",
            prompts: &["af"],
        },
        "s9-assert-test" => Workload {
            setup: "On[Assert];",
            body: "Assert[(InputString[\"test\"]; True)]",
            item: "Null",
            prompts: &["test"],
        },
        "s9-assert-handler" => Workload {
            setup: "$AssertFunction = Function[x, InputString[\"handler\"]];",
            body: "Assert[False]",
            item: "Null",
            prompts: &["handler"],
        },
        "s9-check-body" => Workload {
            setup: "",
            body: "Check[(InputString[\"body\"]; 1), 2]",
            item: "1",
            prompts: &["body"],
        },
        "s9-quiet-body" => Workload {
            setup: "",
            body: "Quiet[(InputString[\"body\"]; {1}[[3]]; 5)]",
            item: "5",
            prompts: &["body"],
        },
        "s9-quiet-check-fallback" => Workload {
            setup: "",
            body: "Quiet[Check[(InputString[\"body\"]; {1}[[3]]), 9]]",
            item: "9",
            prompts: &["body"],
        },
        "s9-cleanup-init" => Workload {
            setup: "",
            body: "WithCleanup[(InputString[\"init\"]; 1), 2, Null]",
            item: "2",
            prompts: &["init"],
        },
        "s9-cleanup-primary" => Workload {
            setup: "",
            body: "WithCleanup[(InputString[\"primary\"]; 2), Null]",
            item: "2",
            prompts: &["primary"],
        },
        "s9-cleanup-finalizer" => Workload {
            setup: "",
            body: "WithCleanup[2, (InputString[\"cleanup\"]; Null)]",
            item: "2",
            prompts: &["cleanup"],
        },
        "s9-cleanup-three-phase" => Workload {
            setup: "",
            body: "WithCleanup[(InputString[\"init\"]; 1), (InputString[\"primary\"]; 2), (InputString[\"cleanup\"]; Null)]",
            item: "2",
            prompts: &["init", "primary", "cleanup"],
        },
        "s9-catchexceptions-body" => Workload {
            setup: "",
            body: "CatchExceptions[(InputString[\"body\"]; ThrowException[\"Exception\", <|\"a\" -> 1|>])]",
            item: "Exception[\"Exception\", Association[Rule[\"a\", 1]]]",
            prompts: &["body"],
        },
        "s9-catchexceptions-guard" => Workload {
            setup: "",
            body: "CatchExceptions[ThrowException[\"Exception\", <|\"a\" -> 1|>], {Exception[\"Exception\", p_] :> (p[\"a\"] /; (InputString[\"guard\"]; True))}]",
            item: "1",
            prompts: &["guard"],
        },
        "s9-catchexceptions-match" => Workload {
            setup: "",
            body: "CatchExceptions[ThrowException[\"Exception\", <|\"a\" -> 1|>], {Exception[\"Exception\", p_] /; (InputString[\"match\"]; True) :> p[\"a\"]}]",
            item: "1",
            prompts: &["match"],
        },
        "s9-catchexceptions-suspended-match" => Workload {
            setup: "",
            body: "CatchExceptions[(InputString[\"body\"]; ThrowException[\"Exception\", <|\"a\" -> 2|>]), {Exception[\"Exception\", p_] /; (InputString[\"match\"]; True) :> p[\"a\"]}]",
            item: "2",
            prompts: &["body", "match"],
        },
        "s9-catchexceptions-suspended-guard" => Workload {
            setup: "",
            body: "CatchExceptions[(InputString[\"body\"]; ThrowException[\"Exception\", <|\"a\" -> 2|>]), {Exception[\"Exception\", p_] :> (p[\"a\"] /; (InputString[\"guard\"]; True))}]",
            item: "2",
            prompts: &["body", "guard"],
        },
        "s9-catchexceptions-rhs" => Workload {
            setup: "",
            body: "CatchExceptions[ThrowException[\"Exception\", <|\"a\" -> 4|>], {Exception[\"Exception\", p_] :> (InputString[\"rhs\"]; p[\"a\"])}]",
            item: "4",
            prompts: &["rhs"],
        },
        "s9-catch-body" => Workload {
            setup: "",
            body: "Catch[(InputString[\"body\"]; Throw[1])]",
            item: "1",
            prompts: &["body"],
        },
        "s9-catch-form" => Workload {
            setup: "",
            body: "Catch[Throw[1, t], (InputString[\"form\"]; t)]",
            item: "1",
            prompts: &["form"],
        },
        "s9-catch-form-guard" => Workload {
            setup: "",
            body: "Catch[Throw[1, t[2]], t[x_] /; (InputString[\"guard\"]; x > 1)]",
            item: "1",
            prompts: &["guard"],
        },
        "s9-catch-bypassing-form" => Workload {
            setup: "",
            body: "Catch[Catch[WithCleanup[Throw[1, t], Throw[2, t]], (InputString[\"form\"]; t)], t]",
            item: "1",
            prompts: &["form"],
        },
        "s9-catch-bypassing-guard" => Workload {
            setup: "",
            body: "Catch[Catch[WithCleanup[Throw[1, t[1]], Throw[2, t[1]]], t[x_] /; (InputString[\"guard\"]; True)], _]",
            item: "1",
            prompts: &["guard"],
        },
        _ => return None,
    })
}
