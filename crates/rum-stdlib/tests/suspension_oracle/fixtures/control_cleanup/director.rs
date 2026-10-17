use super::equality;

#[test]
fn director_01() {
    equality(
        "s9.director_01",
        r###""###,
        r###"CheckAbort[(InputString["p"]; 1), 2]"###,
        &["p"],
        Some(r###"1"###),
    );
}

#[test]
fn director_02() {
    equality(
        "s9.director_02",
        r###""###,
        r###"CheckAbort[(InputString["p"]; Abort[]), 7]"###,
        &["p"],
        Some(r###"7"###),
    );
}

#[test]
fn director_03() {
    equality(
        "s9.director_03",
        r###""###,
        r###"CheckAbort[Abort[], (InputString["f"]; 8)]"###,
        &["f"],
        Some(r###"8"###),
    );
}

#[test]
fn director_04() {
    equality(
        "s9.director_04",
        r###""###,
        r###"CheckAbort[(InputString["p"]; Abort[]), (InputString["f"]; 9)]"###,
        &["p", "f"],
        Some(r###"9"###),
    );
}

#[test]
fn director_05() {
    equality(
        "s9.director_05",
        r###""###,
        r###"CheckAbort[AbortProtect[Abort[]; 3], (InputString["f"]; 9)]"###,
        &["f"],
        Some(r###"9"###),
    );
}

#[test]
fn director_06() {
    equality(
        "s9.director_06",
        r###""###,
        r###"AbortProtect[CheckAbort[CheckAbort[Abort[], 4, PropagateAborts -> True], (InputString["f"]; 9)]]"###,
        &["f"],
        Some(r###"9"###),
    );
}

#[test]
fn director_07() {
    equality(
        "s9.director_07",
        r###""###,
        r###"{AbortProtect[CheckAbort[CheckAbort[Abort[], 4, PropagateAborts -> True], (InputString["f"]; 9)]; 7], 8}"###,
        &["f"],
        Some(r###"List[7, 8]"###),
    );
}

#[test]
fn director_08() {
    equality(
        "s9.director_08",
        r###""###,
        r###"Table[AbortProtect[CheckAbort[CheckAbort[Abort[], i, PropagateAborts -> True], (InputString["f"]; -i)]], {i, 2}]"###,
        &["f", "f"],
        Some(r###"List[-1, -2]"###),
    );
}

#[test]
fn director_09() {
    equality(
        "s9.director_09",
        r###""###,
        r###"CheckAbort[1, 2, PropagateAborts -> (InputString["o"]; False)]"###,
        &["o"],
        Some(r###"1"###),
    );
}

#[test]
fn director_10() {
    equality(
        "s9.director_10",
        r###""###,
        r###"CheckAbort[Abort[], (InputString["f"]; 9), PropagateAborts -> True]"###,
        &["f"],
        Some(r###"$Aborted"###),
    );
}

#[test]
fn director_11() {
    equality(
        "s9.director_11",
        r###""###,
        r###"AbortProtect[CheckAbort[Abort[], (InputString["f"]; 9), PropagateAborts -> True]; 5]"###,
        &["f"],
        Some(r###"$Aborted"###),
    );
}

#[test]
fn director_12() {
    equality(
        "s9.director_12",
        r###""###,
        r###"Table[CheckAbort[(InputString["p"]; If[i == 2, Abort[], i]), -i], {i, 3}]"###,
        &["p", "p", "p"],
        Some(r###"List[1, -2, 3]"###),
    );
}

#[test]
fn director_13() {
    equality(
        "s9.director_13",
        r###""###,
        r###"AbortProtect[(InputString["p"]; 3)]"###,
        &["p"],
        Some(r###"3"###),
    );
}

#[test]
fn director_14() {
    equality(
        "s9.director_14",
        r###""###,
        r###"AbortProtect[(InputString["p"]; Abort[]; 4)]"###,
        &["p"],
        Some(r###"$Aborted"###),
    );
}

#[test]
fn director_15() {
    equality(
        "s9.director_15",
        r###""###,
        r###"CheckAbort[AbortProtect[(InputString["p"]; Abort[]); 5], 6]"###,
        &["p"],
        Some(r###"6"###),
    );
}

#[test]
fn director_16() {
    equality(
        "s9.director_16",
        r###""###,
        r###"AbortProtect[AbortProtect[(InputString["p"]; Abort[]); 5]; 6]"###,
        &["p"],
        Some(r###"$Aborted"###),
    );
}

#[test]
fn director_17() {
    equality(
        "s9.director_17",
        r###"$AssertFunction := (InputString["f"]; None)"###,
        r###"Assert[1 > 2]"###,
        &["f"],
        Some(r###"Assert[Greater[1, 2]]"###),
    );
}

#[test]
fn director_18() {
    equality(
        "s9.director_18",
        r###"On[Assert]"###,
        r###"Assert[(InputString["t"]; 1 > 0)]"###,
        &["t"],
        Some(r###"Null"###),
    );
}

#[test]
fn director_19() {
    equality(
        "s9.director_19",
        r###"On[Assert]"###,
        r###"Assert[(InputString["t"]; False)]"###,
        &["t"],
        Some(r###"Null"###),
    );
}

#[test]
fn director_20() {
    equality(
        "s9.director_20",
        r###"$AssertFunction = ((InputString["h"]; x = #) &)"###,
        r###"Assert[False]; x"###,
        &["h"],
        Some(r###"HoldComplete[Assert[False]]"###),
    );
}

#[test]
fn director_21() {
    equality(
        "s9.director_21",
        r###"On[Assert]"###,
        r###"Table[Assert[(InputString["t"]; i > 1)], {i, 2}]"###,
        &["t", "t"],
        Some(r###"List[Null, Null]"###),
    );
}

#[test]
fn director_22() {
    equality(
        "s9.director_22",
        r###""###,
        r###"Check[(InputString["p"]; 1), 2]"###,
        &["p"],
        Some(r###"1"###),
    );
}

#[test]
fn director_23() {
    equality(
        "s9.director_23",
        r###""###,
        r###"Check[(InputString["p"]; {1}[[3]]), 9]"###,
        &["p"],
        Some(r###"9"###),
    );
}

#[test]
fn director_24() {
    equality(
        "s9.director_24",
        r###""###,
        r###"Check[{1}[[3]]; (InputString["p"]; 5), 9]"###,
        &["p"],
        Some(r###"9"###),
    );
}

#[test]
fn director_25() {
    equality(
        "s9.director_25",
        r###""###,
        r###"{1}[[3]]; Check[(InputString["p"]; {1}[[3]]), 9]"###,
        &["p"],
        Some(r###"9"###),
    );
}

#[test]
fn director_26() {
    equality(
        "s9.director_26",
        r###""###,
        r###"{1}[[3]]; Check[(InputString["p"]; 5), 9]"###,
        &["p"],
        Some(r###"5"###),
    );
}

#[test]
fn director_27() {
    equality(
        "s9.director_27",
        r###""###,
        r###"Check[(InputString["p"]; {1}[[3]]), 9, Part::partw]"###,
        &["p"],
        Some(r###"9"###),
    );
}

#[test]
fn director_28() {
    equality(
        "s9.director_28",
        r###""###,
        r###"Check[(InputString["p"]; Throw[1]), 9]"###,
        &["p"],
        None,
    );
}

#[test]
fn director_29() {
    equality(
        "s9.director_29",
        r###""###,
        r###"Quiet[(InputString["p"]; {1}[[3]])]"###,
        &["p"],
        Some(r###"Part[List[1], 3]"###),
    );
}

#[test]
fn director_30() {
    equality(
        "s9.director_30",
        r###""###,
        r###"Quiet[{1}[[3]]; (InputString["p"]; 5)]"###,
        &["p"],
        Some(r###"5"###),
    );
}

#[test]
fn director_31() {
    equality(
        "s9.director_31",
        r###""###,
        r###"{1}[[3]]; Quiet[(InputString["p"]; {1}[[3]]); 6]"###,
        &["p"],
        Some(r###"6"###),
    );
}

#[test]
fn director_32() {
    equality(
        "s9.director_32",
        r###""###,
        r###"Quiet[(InputString["p"]; {1}[[3]]; 7), Part::partw]"###,
        &["p"],
        Some(r###"7"###),
    );
}

#[test]
fn director_33() {
    equality(
        "s9.director_33",
        r###""###,
        r###"WithCleanup[(InputString["i"]; 1), (InputString["p"]; 2), (InputString["c"]; c = 3)]; c"###,
        &["i", "p", "c"],
        Some(r###"3"###),
    );
}

#[test]
fn director_34() {
    equality(
        "s9.director_34",
        r###""###,
        r###"{WithCleanup[(InputString["p"]; 2), c = 4], c}"###,
        &["p"],
        Some(r###"List[2, 4]"###),
    );
}

#[test]
fn director_35() {
    equality(
        "s9.director_35",
        r###""###,
        r###"{WithCleanup[2, (InputString["c"]; c = 5)], c}"###,
        &["c"],
        Some(r###"List[2, 5]"###),
    );
}

#[test]
fn director_36() {
    equality(
        "s9.director_36",
        r###""###,
        r###"{Catch[WithCleanup[(InputString["p"]; Throw[1]), (InputString["c"]; c = 6)]], c}"###,
        &["p", "c"],
        Some(r###"List[1, 6]"###),
    );
}

#[test]
fn director_37() {
    equality(
        "s9.director_37",
        r###""###,
        r###"{CheckAbort[WithCleanup[(InputString["p"]; Abort[]), (InputString["c"]; c = 7)], -1], c}"###,
        &["p", "c"],
        Some(r###"List[-1, 7]"###),
    );
}

#[test]
fn director_38() {
    equality(
        "s9.director_38",
        r###""###,
        r###"Catch[WithCleanup[Throw[1, t], (InputString["c"]; Throw[2, t])], t]"###,
        &["c"],
        None,
    );
}

#[test]
fn director_39() {
    equality(
        "s9.director_39",
        r###""###,
        r###"Table[WithCleanup[(InputString["p"]; i), (InputString["c"]; s[i] = i)], {i, 2}]"###,
        &["p", "c", "p", "c"],
        Some(r###"List[1, 2]"###),
    );
}

#[path = "director_nonlocal.rs"]
mod nonlocal;
