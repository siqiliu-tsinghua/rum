use super::equality;

#[test]
fn director_20() {
    equality(
        "s8.director_20",
        r###""###,
        r###"Count[(InputString["t"]; {1, 2, 1}), 1]"###,
        &["t"],
        r###"2"###,
    );
}

#[test]
fn director_21() {
    equality(
        "s8.director_21",
        r###""###,
        r###"MemberQ[(InputString["t"]; {1, 2}), 2]"###,
        &["t"],
        r###"True"###,
    );
}

#[test]
fn director_22() {
    equality(
        "s8.director_22",
        r###""###,
        r###"FreeQ[(InputString["t"]; {1, 2}), 3]"###,
        &["t"],
        r###"True"###,
    );
}

#[test]
fn director_23() {
    equality(
        "s8.director_23",
        r###""###,
        r###"Count[{1, 2, 1}, (InputString["p"]; 1)]"###,
        &["p"],
        r###"2"###,
    );
}

#[test]
fn director_24() {
    equality(
        "s8.director_24",
        r###""###,
        r###"MemberQ[{1, 2}, (InputString["p"]; 2)]"###,
        &["p"],
        r###"True"###,
    );
}

#[test]
fn director_25() {
    equality(
        "s8.director_25",
        r###""###,
        r###"FreeQ[{1, 2}, (InputString["p"]; 2)]"###,
        &["p"],
        r###"False"###,
    );
}

#[test]
fn director_26() {
    equality(
        "s8.director_26",
        r###""###,
        r###"Count[{1, {1}}, 1, (InputString["l"]; {2})]"###,
        &["l"],
        r###"1"###,
    );
}

#[test]
fn director_27() {
    equality(
        "s8.director_27",
        r###""###,
        r###"MemberQ[{1, {2}}, 2, (InputString["l"]; {2})]"###,
        &["l"],
        r###"True"###,
    );
}

#[test]
fn director_28() {
    equality(
        "s8.director_28",
        r###""###,
        r###"FreeQ[{1, {2}}, 2, (InputString["l"]; {1})]"###,
        &["l"],
        r###"True"###,
    );
}

#[test]
fn director_29() {
    equality(
        "s8.director_29",
        r###""###,
        r###"Count[{1, 2, 3}, x_ /; (InputString["t"]; x > 1)]"###,
        &["t", "t", "t"],
        r###"2"###,
    );
}

#[test]
fn director_30() {
    equality(
        "s8.director_30",
        r###""###,
        r###"MemberQ[{1, 2, 3}, x_ /; (InputString["t"]; x > 2)]"###,
        &["t", "t", "t"],
        r###"True"###,
    );
}

#[test]
fn director_31() {
    equality(
        "s8.director_31",
        r###""###,
        r###"FreeQ[{1, 2, 3}, x_ /; (InputString["t"]; x > 5)]"###,
        &["t", "t", "t", "t", "t"],
        r###"True"###,
    );
}

#[test]
fn director_32() {
    equality(
        "s8.director_32",
        r###""###,
        r###"Count[f[f, 1], f, {0, Infinity}, Heads -> (InputString["h"]; True)]"###,
        &["h"],
        r###"2"###,
    );
}

#[test]
fn director_33() {
    equality(
        "s8.director_33",
        r###""###,
        r###"Count[f[g, 1], s_Symbol /; (InputString["g"]; True), {0, Infinity}, Heads -> True]"###,
        &["g", "g"],
        r###"2"###,
    );
}

#[test]
fn director_34() {
    equality(
        "s8.director_34",
        r###""###,
        r###"Count[{System`InternalContinuationNone[], System`InternalContinuationSome[5], 1}, x_ /; (InputString["g"]; True)]"###,
        &["g", "g", "g"],
        r###"3"###,
    );
}

#[test]
fn director_35() {
    equality(
        "s8.director_35",
        r###""###,
        r###"MemberQ[{System`InternalContinuationSome[5]}, (InputString["p"]; System`InternalContinuationSome[5])]"###,
        &["p"],
        r###"True"###,
    );
}

#[test]
fn director_36() {
    equality(
        "s8.director_36",
        r###""###,
        r###"Position[(InputString["t"]; {1, 2, 1}), 1]"###,
        &["t"],
        r###"List[List[1], List[3]]"###,
    );
}

#[test]
fn director_37() {
    equality(
        "s8.director_37",
        r###""###,
        r###"Position[{1, 2, 1}, (InputString["p"]; 1)]"###,
        &["p"],
        r###"List[List[1], List[3]]"###,
    );
}

#[test]
fn director_38() {
    equality(
        "s8.director_38",
        r###""###,
        r###"Position[{1, {1}}, 1, (InputString["l"]; {2})]"###,
        &["l"],
        r###"List[List[2, 1]]"###,
    );
}

#[test]
fn director_39() {
    equality(
        "s8.director_39",
        r###""###,
        r###"Position[{1, 1, 1}, 1, {1}, (InputString["n"]; 2)]"###,
        &["n"],
        r###"List[List[1], List[2]]"###,
    );
}

#[test]
fn director_40() {
    equality(
        "s8.director_40",
        r###""###,
        r###"Position[{1, 2, 3}, x_ /; (InputString["t"]; x > 1)]"###,
        &["t", "t", "t", "t", "t"],
        r###"List[List[2], List[3]]"###,
    );
}

#[test]
fn director_41() {
    equality(
        "s8.director_41",
        r###""###,
        r###"Position[{{1}, {2, 1}}, x_Integer /; (InputString["t"]; x == 1)]"###,
        &["t", "t", "t"],
        r###"List[List[1, 1], List[2, 2]]"###,
    );
}

#[test]
fn director_42() {
    equality(
        "s8.director_42",
        r###""###,
        r###"Position[{1, 2, 3}, x_ /; (InputString["g"]; True), {1}, 2]"###,
        &["g", "g"],
        r###"List[List[0], List[1]]"###,
    );
}

#[test]
fn director_43() {
    equality(
        "s8.director_43",
        r###""###,
        r###"Position[{System`InternalContinuationNone[], System`InternalContinuationSome[5]}, x_ /; (InputString["g"]; True)]"###,
        &["g", "g", "g", "g", "g", "g", "g"],
        r###"List[List[0], List[1, 0], List[1], List[2, 0], List[2, 1], List[2], List[]]"###,
    );
}

#[test]
fn director_44() {
    equality(
        "s8.director_44",
        r###""###,
        r###"Position[f[f, 1], f, Heads -> (InputString["h"]; True)]"###,
        &["h"],
        r###"List[List[0], List[1]]"###,
    );
}

#[test]
fn director_45() {
    equality(
        "s8.director_45",
        r###"c = 0;"###,
        r###"Do[c += Count[{1, 2}, x_ /; (InputString["t"]; True)], {3}]; c"###,
        &["t", "t", "t", "t", "t", "t"],
        r###"6"###,
    );
}

#[test]
fn director_46() {
    equality(
        "s8.director_46",
        r###""###,
        r###"Table[{i} /. x_Integer /; (InputString["g"]; True) :> x + 1, {i, 3}]"###,
        &["g", "g", "g"],
        r###"List[List[2], List[3], List[4]]"###,
    );
}

#[test]
fn director_47() {
    equality(
        "s8.director_47",
        r###""###,
        r###"Table[Position[{i, 0}, x_ /; (InputString["t"]; x > 0)], {i, 2}]"###,
        &["t", "t", "t", "t", "t", "t", "t", "t"],
        r###"List[List[List[1]], List[List[1]]]"###,
    );
}

#[test]
fn director_48() {
    equality(
        "s8.director_48",
        r###""###,
        r###"Table[MemberQ[{i}, x_ /; (InputString["g"]; True)] && FreeQ[{i}, x_ /; (InputString["g"]; True)], {i, 2}]"###,
        &["g", "g", "g", "g"],
        r###"List[False, False]"###,
    );
}
