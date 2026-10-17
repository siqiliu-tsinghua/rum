use super::equality;

#[test]
fn director_40() {
    equality(
        "s9.director_40",
        r###""###,
        r###"Catch[(InputString["p"]; Throw[1])]"###,
        &["p"],
        Some(r###"1"###),
    );
}

#[test]
fn director_41() {
    equality(
        "s9.director_41",
        r###""###,
        r###"Catch[(InputString["p"]; Throw[1, t]), t]"###,
        &["p"],
        Some(r###"1"###),
    );
}

#[test]
fn director_42() {
    equality(
        "s9.director_42",
        r###""###,
        r###"Catch[Throw[1, t], (InputString["f"]; t)]"###,
        &["f"],
        Some(r###"1"###),
    );
}

#[test]
fn director_43() {
    equality(
        "s9.director_43",
        r###""###,
        r###"Catch[Throw[1, t[2]], t[x_] /; (InputString["g"]; x > 1)]"###,
        &["g"],
        Some(r###"1"###),
    );
}

#[test]
fn director_44() {
    equality(
        "s9.director_44",
        r###""###,
        r###"Catch[Catch[Throw[1, t[2]], t[x_] /; (InputString["g"]; x > 5)], _]"###,
        &["g"],
        Some(r###"1"###),
    );
}

#[test]
fn director_45() {
    equality(
        "s9.director_45",
        r###""###,
        r###"Catch[Throw[1, t], t, (InputString["h"]; {#1, #2}) &]"###,
        &["h"],
        Some(r###"List[1, t]"###),
    );
}

#[test]
fn director_46() {
    equality(
        "s9.director_46",
        r###""###,
        r###"Catch[Catch[WithCleanup[Throw[1, t], Throw[2, t]], (InputString["f"]; t)], t]"###,
        &["f"],
        Some(r###"1"###),
    );
}

#[test]
fn director_47() {
    equality(
        "s9.director_47",
        r###""###,
        r###"Catch[Catch[WithCleanup[Throw[1, t[1]], Throw[2, t[1]]], t[x_] /; (InputString["g"]; True)], _]"###,
        &["g"],
        Some(r###"1"###),
    );
}

#[test]
fn director_48() {
    equality(
        "s9.director_48",
        r###""###,
        r###"Catch[Catch[WithCleanup[Throw[1, t], Throw[2, t]], (InputString["f"]; u)], t]"###,
        &["f"],
        None,
    );
}

#[test]
fn director_49() {
    equality(
        "s9.director_49",
        r###""###,
        r###"Table[Catch[(InputString["p"]; If[i > 1, Throw[i], 0])], {i, 3}]"###,
        &["p", "p", "p"],
        Some(r###"List[0, 2, 3]"###),
    );
}

#[test]
fn director_50() {
    equality(
        "s9.director_50",
        r###""###,
        r###"CatchExceptions[(InputString["p"]; ThrowException["Exception", <|"a" -> 1|>])]"###,
        &["p"],
        Some(r###"Exception["Exception", Association[Rule["a", 1]]]"###),
    );
}

#[test]
fn director_51() {
    equality(
        "s9.director_51",
        r###""###,
        r###"CatchExceptions[ThrowException["Exception", <|"a" -> 1|>], {Exception["Exception", p_] /; (InputString["g"]; True) :> p["a"]}]"###,
        &["g"],
        Some(r###"1"###),
    );
}

#[test]
fn director_52() {
    equality(
        "s9.director_52",
        r###""###,
        r###"CatchExceptions[ThrowException["Exception", <|"a" -> 1|>], {Exception["Exception", p_?((InputString["g"]; True) &)] :> p["a"]}]"###,
        &["g"],
        Some(r###"1"###),
    );
}

#[test]
fn director_53() {
    equality(
        "s9.director_53",
        r###""###,
        r###"CatchExceptions[(InputString["p"]; ThrowException["Exception", <|"a" -> 2|>]), {Exception["Exception", p_] /; (InputString["g"]; True) :> p["a"]}]"###,
        &["p", "g"],
        Some(r###"2"###),
    );
}

#[test]
fn director_54() {
    equality(
        "s9.director_54",
        r###""###,
        r###"CatchExceptions[(InputString["p"]; ThrowException["Exception", <|"a" -> 2|>]), {Exception["Exception", p_?((InputString["g"]; True) &)] :> p["a"]}]"###,
        &["p", "g"],
        Some(r###"2"###),
    );
}

#[test]
fn director_55() {
    equality(
        "s9.director_55",
        r###""###,
        r###"CatchExceptions[ThrowException["Exception", <|"a" -> 3|>], {Exception["Exception", p_] /; (InputString["g"]; False) :> 0, Exception["Exception", p_] :> p["a"]}]"###,
        &["g"],
        Some(r###"3"###),
    );
}

#[test]
fn director_56() {
    equality(
        "s9.director_56",
        r###""###,
        r###"CatchExceptions[ThrowException["Exception", <|"a" -> 4|>], {Exception["Exception", p_] :> (InputString["r"]; p["a"])}]"###,
        &["r"],
        Some(r###"4"###),
    );
}

#[test]
fn director_57() {
    equality(
        "s9.director_57",
        r###"RegisterExceptionType["NetError"]; RegisterExceptionType["Timeout", "NetError"]"###,
        r###"CatchExceptions[(InputString["p"]; ThrowException["Timeout", <|"ms" -> 30|>]), {Exception["NetError", p_] /; (InputString["g"]; True) :> {"parent", p["ms"]}}]"###,
        &["p", "g"],
        Some(r###"List["parent", 30]"###),
    );
}

#[test]
fn director_58() {
    equality(
        "s9.director_58",
        r###"RegisterExceptionType["NetError"]; RegisterExceptionType["Timeout", "NetError"]"###,
        r###"CatchExceptions[ThrowException["Timeout", <|"ms" -> 30|>], {Exception[t_, p_] /; (InputString["g"]; t == "NetError") :> {t, p["ms"]}}]"###,
        &["g", "g"],
        Some(r###"List["NetError", 30]"###),
    );
}

#[test]
fn director_59() {
    equality(
        "s9.director_59",
        r###""###,
        r###"Table[CatchExceptions[(InputString["p"]; ThrowException["Exception", <|"i" -> i|>]), {Exception["Exception", p_] /; (InputString["g"]; p["i"] > 1) :> p["i"], _ :> 0}], {i, 2}]"###,
        &["p", "g", "p", "g"],
        Some(r###"List[0, 2]"###),
    );
}

#[test]
fn director_60() {
    equality(
        "s9.director_60",
        r###""###,
        r###"Catch[CheckAbort[AbortProtect[(InputString["p"]; Throw[1])], 2]]"###,
        &["p"],
        Some(r###"1"###),
    );
}

#[test]
fn director_61() {
    equality(
        "s9.director_61",
        r###""###,
        r###"Quiet[Check[CatchExceptions[(InputString["p"]; {1}[[3]]; ThrowException["Exception", <||>])], 9]]"###,
        &["p"],
        Some(r###"9"###),
    );
}
