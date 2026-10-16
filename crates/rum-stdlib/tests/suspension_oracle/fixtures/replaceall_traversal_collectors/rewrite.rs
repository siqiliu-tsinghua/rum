use super::equality;

#[test]
fn director_01() {
    equality(
        "s8.director_01",
        r###""###,
        r###"f[1] /. f[x_] /; (InputString["g"]; True) :> x"###,
        &["g"],
        r###"1"###,
    );
}

#[test]
fn director_02() {
    equality(
        "s8.director_02",
        r###""###,
        r###"{1, 2, 3} /. x_Integer /; (InputString["g"]; x > 1) :> 10 x"###,
        &["g", "g", "g"],
        r###"List[1, 20, 30]"###,
    );
}

#[test]
fn director_03() {
    equality(
        "s8.director_03",
        r###""###,
        r###"1 /. x_ /; (InputString["g"]; True) :> 7"###,
        &["g"],
        r###"7"###,
    );
}

#[test]
fn director_04() {
    equality(
        "s8.director_04",
        r###""###,
        r###"{1, 2} /. {x_ /; (InputString["g"]; False) :> 0, x_Integer :> x + 1}"###,
        &["g", "g", "g", "g"],
        r###"List[2, 3]"###,
    );
}

#[test]
fn director_05() {
    equality(
        "s8.director_05",
        r###""###,
        r###"{{1}, {2, {3}}} /. x_Integer /; (InputString["g"]; True) :> -x"###,
        &["g", "g", "g"],
        r###"List[List[-1], List[-2, List[-3]]]"###,
    );
}

#[test]
fn director_06() {
    equality(
        "s8.director_06",
        r###""###,
        r###"g[h[1], 2] /. {h[x_] /; (InputString["g"]; True) :> x, 2 -> 3}"###,
        &["g"],
        r###"g[1, 3]"###,
    );
}

#[test]
fn director_07() {
    equality(
        "s8.director_07",
        r###""###,
        r###"{1, 2} /. x_Integer /; (InputString["g"]; True) :> Sequence[x, x]"###,
        &["g", "g"],
        r###"List[1, 1, 2, 2]"###,
    );
}

#[test]
fn director_08() {
    equality(
        "s8.director_08",
        r###""###,
        r###"Hold[1 + 1] /. x_Integer /; (InputString["g"]; True) :> 5"###,
        &["g", "g"],
        r###"Hold[Plus[5, 5]]"###,
    );
}

#[test]
fn director_09() {
    equality(
        "s8.director_09",
        r###""###,
        r###"f[3] //. f[x_ /; (InputString["g"]; x > 0)] :> f[x - 1]"###,
        &["g", "g", "g", "g"],
        r###"f[0]"###,
    );
}

#[test]
fn director_10() {
    equality(
        "s8.director_10",
        r###""###,
        r###"{a, b} //. {a /; (InputString["g"]; True) -> b, b -> c}"###,
        &["g"],
        r###"List[c, c]"###,
    );
}

#[test]
fn director_11() {
    equality(
        "s8.director_11",
        r###""###,
        r###"Replace[5, x_ /; (InputString["g"]; True) :> x + 1]"###,
        &["g"],
        r###"6"###,
    );
}

#[test]
fn director_12() {
    equality(
        "s8.director_12",
        r###""###,
        r###"Replace[{1, 2}, x_Integer /; (InputString["g"]; True) :> x + 1, {1}]"###,
        &["g", "g"],
        r###"List[2, 3]"###,
    );
}

#[test]
fn director_13() {
    equality(
        "s8.director_13",
        r###""###,
        r###"Replace[3, {x_ /; (InputString["g"]; False) :> 0, x_ /; (InputString["g"]; True) :> 9}]"###,
        &["g", "g"],
        r###"9"###,
    );
}

#[test]
fn director_14() {
    equality(
        "s8.director_14",
        r###""###,
        r###"ReplaceAt[{1, 2, 3}, x_ /; (InputString["g"]; True) :> 0, 2]"###,
        &["g"],
        r###"List[1, 0, 3]"###,
    );
}

#[test]
fn director_15() {
    equality(
        "s8.director_15",
        r###""###,
        r###"{1} /. x_Integer /; (InputString["g"]; True) :> System`InternalContinuationNone[]"###,
        &["g"],
        r###"List[InternalContinuationNone[]]"###,
    );
}

#[test]
fn director_16() {
    equality(
        "s8.director_16",
        r###""###,
        r###"{1} /. x_Integer /; (InputString["g"]; True) :> System`InternalContinuationSome[5]"###,
        &["g"],
        r###"List[InternalContinuationSome[5]]"###,
    );
}

#[test]
fn director_17() {
    equality(
        "s8.director_17",
        r###""###,
        r###"{1} /. x_Integer /; (InputString["g"]; True) :> System`InternalContinuationReplaceAllTraversalOutcome[x, True]"###,
        &["g"],
        r###"List[InternalContinuationReplaceAllTraversalOutcome[1, True]]"###,
    );
}

#[test]
fn director_18() {
    equality(
        "s8.director_18",
        r###""###,
        r###"{System`InternalContinuationNone[], 1} /. x_Integer /; (InputString["g"]; True) :> 2"###,
        &["g"],
        r###"List[InternalContinuationNone[], 2]"###,
    );
}

#[test]
fn director_19() {
    equality(
        "s8.director_19",
        r###""###,
        r###"Replace[1, x_ /; (InputString["g"]; True) :> System`InternalContinuationReplaceAllTraversalOutcome[2, False]]"###,
        &["g"],
        r###"InternalContinuationReplaceAllTraversalOutcome[2, False]"###,
    );
}
