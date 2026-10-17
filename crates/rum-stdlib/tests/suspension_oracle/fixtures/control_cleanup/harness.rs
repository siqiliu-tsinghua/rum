use super::equality;

#[test]
fn s9_checkabort_option() {
    equality(
        "s9.s9_checkabort_option",
        r###""###,
        r###"CheckAbort[1, 2, PropagateAborts -> (InputString["opt"]; False)]"###,
        &["opt"],
        Some(r###"1"###),
    );
}

#[test]
fn s9_checkabort_primary() {
    equality(
        "s9.s9_checkabort_primary",
        r###""###,
        r###"CheckAbort[(InputString["primary"]; 1), 2]"###,
        &["primary"],
        Some(r###"1"###),
    );
}

#[test]
fn s9_checkabort_abort() {
    equality(
        "s9.s9_checkabort_abort",
        r###""###,
        r###"CheckAbort[(InputString["primary"]; Abort[]), 7]"###,
        &["primary"],
        Some(r###"7"###),
    );
}

#[test]
fn s9_abortprotect_body() {
    equality(
        "s9.s9_abortprotect_body",
        r###""###,
        r###"AbortProtect[(InputString["body"]; 3)]"###,
        &["body"],
        Some(r###"3"###),
    );
}

#[test]
fn s9_checkabort_pending_fallback() {
    equality(
        "s9.s9_checkabort_pending_fallback",
        r###""###,
        r###"AbortProtect[CheckAbort[(Abort[]; 1), (InputString["fallback"]; 9)]]"###,
        &["fallback"],
        Some(r###"9"###),
    );
}

#[test]
fn s9_checkabort_fallback() {
    equality(
        "s9.s9_checkabort_fallback",
        r###""###,
        r###"CheckAbort[Abort[], (InputString["fallback"]; 8)]"###,
        &["fallback"],
        Some(r###"8"###),
    );
}

#[test]
fn s9_assert_function_lookup() {
    equality(
        "s9.s9_assert_function_lookup",
        r###"On[Assert]; $AssertFunction := (InputString["af"]; None);"###,
        r###"Assert[True]"###,
        &["af"],
        Some(r###"Null"###),
    );
}

#[test]
fn s9_assert_test() {
    equality(
        "s9.s9_assert_test",
        r###"On[Assert];"###,
        r###"Assert[(InputString["test"]; True)]"###,
        &["test"],
        Some(r###"Null"###),
    );
}

#[test]
fn s9_assert_handler() {
    equality(
        "s9.s9_assert_handler",
        r###"$AssertFunction = Function[x, InputString["handler"]];"###,
        r###"Assert[False]"###,
        &["handler"],
        Some(r###"Null"###),
    );
}

#[test]
fn s9_check_body() {
    equality(
        "s9.s9_check_body",
        r###""###,
        r###"Check[(InputString["body"]; 1), 2]"###,
        &["body"],
        Some(r###"1"###),
    );
}

#[test]
fn s9_quiet_body() {
    equality(
        "s9.s9_quiet_body",
        r###""###,
        r###"Quiet[(InputString["body"]; {1}[[3]]; 5)]"###,
        &["body"],
        Some(r###"5"###),
    );
}

#[test]
fn s9_quiet_check_fallback() {
    equality(
        "s9.s9_quiet_check_fallback",
        r###""###,
        r###"Quiet[Check[(InputString["body"]; {1}[[3]]), 9]]"###,
        &["body"],
        Some(r###"9"###),
    );
}

#[test]
fn s9_cleanup_init() {
    equality(
        "s9.s9_cleanup_init",
        r###""###,
        r###"WithCleanup[(InputString["init"]; 1), 2, Null]"###,
        &["init"],
        Some(r###"2"###),
    );
}

#[test]
fn s9_cleanup_primary() {
    equality(
        "s9.s9_cleanup_primary",
        r###""###,
        r###"WithCleanup[(InputString["primary"]; 2), Null]"###,
        &["primary"],
        Some(r###"2"###),
    );
}

#[test]
fn s9_cleanup_finalizer() {
    equality(
        "s9.s9_cleanup_finalizer",
        r###""###,
        r###"WithCleanup[2, (InputString["cleanup"]; Null)]"###,
        &["cleanup"],
        Some(r###"2"###),
    );
}

#[test]
fn s9_cleanup_three_phase() {
    equality(
        "s9.s9_cleanup_three_phase",
        r###""###,
        r###"WithCleanup[(InputString["init"]; 1), (InputString["primary"]; 2), (InputString["cleanup"]; Null)]"###,
        &["init", "primary", "cleanup"],
        Some(r###"2"###),
    );
}

#[test]
fn s9_catchexceptions_body() {
    equality(
        "s9.s9_catchexceptions_body",
        r###""###,
        r###"CatchExceptions[(InputString["body"]; ThrowException["Exception", <|"a" -> 1|>])]"###,
        &["body"],
        Some(r###"Exception["Exception", Association[Rule["a", 1]]]"###),
    );
}

#[test]
fn s9_catchexceptions_guard() {
    equality(
        "s9.s9_catchexceptions_guard",
        r###""###,
        r###"CatchExceptions[ThrowException["Exception", <|"a" -> 1|>], {Exception["Exception", p_] :> (p["a"] /; (InputString["guard"]; True))}]"###,
        &["guard"],
        Some(r###"1"###),
    );
}

#[test]
fn s9_catchexceptions_match() {
    equality(
        "s9.s9_catchexceptions_match",
        r###""###,
        r###"CatchExceptions[ThrowException["Exception", <|"a" -> 1|>], {Exception["Exception", p_] /; (InputString["match"]; True) :> p["a"]}]"###,
        &["match"],
        Some(r###"1"###),
    );
}

#[test]
fn s9_catchexceptions_suspended_match() {
    equality(
        "s9.s9_catchexceptions_suspended_match",
        r###""###,
        r###"CatchExceptions[(InputString["body"]; ThrowException["Exception", <|"a" -> 2|>]), {Exception["Exception", p_] /; (InputString["match"]; True) :> p["a"]}]"###,
        &["body", "match"],
        Some(r###"2"###),
    );
}

#[test]
fn s9_catchexceptions_suspended_guard() {
    equality(
        "s9.s9_catchexceptions_suspended_guard",
        r###""###,
        r###"CatchExceptions[(InputString["body"]; ThrowException["Exception", <|"a" -> 2|>]), {Exception["Exception", p_] :> (p["a"] /; (InputString["guard"]; True))}]"###,
        &["body", "guard"],
        Some(r###"2"###),
    );
}

#[test]
fn s9_catchexceptions_rhs() {
    equality(
        "s9.s9_catchexceptions_rhs",
        r###""###,
        r###"CatchExceptions[ThrowException["Exception", <|"a" -> 4|>], {Exception["Exception", p_] :> (InputString["rhs"]; p["a"])}]"###,
        &["rhs"],
        Some(r###"4"###),
    );
}

#[test]
fn s9_catch_body() {
    equality(
        "s9.s9_catch_body",
        r###""###,
        r###"Catch[(InputString["body"]; Throw[1])]"###,
        &["body"],
        Some(r###"1"###),
    );
}

#[test]
fn s9_catch_form() {
    equality(
        "s9.s9_catch_form",
        r###""###,
        r###"Catch[Throw[1, t], (InputString["form"]; t)]"###,
        &["form"],
        Some(r###"1"###),
    );
}

#[test]
fn s9_catch_form_guard() {
    equality(
        "s9.s9_catch_form_guard",
        r###""###,
        r###"Catch[Throw[1, t[2]], t[x_] /; (InputString["guard"]; x > 1)]"###,
        &["guard"],
        Some(r###"1"###),
    );
}

#[test]
fn s9_catch_bypassing_form() {
    equality(
        "s9.s9_catch_bypassing_form",
        r###""###,
        r###"Catch[Catch[WithCleanup[Throw[1, t], Throw[2, t]], (InputString["form"]; t)], t]"###,
        &["form"],
        Some(r###"1"###),
    );
}

#[test]
fn s9_catch_bypassing_guard() {
    equality(
        "s9.s9_catch_bypassing_guard",
        r###""###,
        r###"Catch[Catch[WithCleanup[Throw[1, t[1]], Throw[2, t[1]]], t[x_] /; (InputString["guard"]; True)], _]"###,
        &["guard"],
        Some(r###"1"###),
    );
}
