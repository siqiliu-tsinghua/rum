// Historical positional association shapes are Q5 equality pairs only.
use super::equality;

#[test]
fn director_01() {
    equality(
        "s7.director_01",
        r###"sweepList={0};"###,
        r###"sweepList[[(InputString["sweep"];1)]]=System`InternalContinuationNone[]"###,
        &["sweep"],
        Some("InternalContinuationNone[]"),
        "List[]",
    );
}

#[test]
fn q5_director_02() {
    equality(
        "s7.q5_director_02",
        r###"swA=<|"k"->{0}|>;"###,
        r###"swA[[(InputString["sweep"];1)]]=System`InternalContinuationNone[]"###,
        &["sweep"],
        None,
        "List[]",
    );
}

#[test]
fn director_03() {
    equality(
        "s7.director_03",
        r###"sweepList={0};"###,
        r###"sweepList[[(InputString["sweep"];1)]]=System`InternalContinuationSome[5]"###,
        &["sweep"],
        Some("InternalContinuationSome[5]"),
        "List[]",
    );
}

#[test]
fn q5_director_04() {
    equality(
        "s7.q5_director_04",
        r###"swA=<|"k"->{0}|>;"###,
        r###"swA[[(InputString["sweep"];1)]]=System`InternalContinuationSome[5]"###,
        &["sweep"],
        None,
        "List[]",
    );
}

#[test]
fn director_05() {
    equality(
        "s7.director_05",
        r###"sweepList={0};"###,
        r###"sweepList[[(InputString["sweep"];1)]]=System`InternalContinuationSome[]"###,
        &["sweep"],
        Some("InternalContinuationSome[]"),
        "List[]",
    );
}

#[test]
fn q5_director_06() {
    equality(
        "s7.q5_director_06",
        r###"swA=<|"k"->{0}|>;"###,
        r###"swA[[(InputString["sweep"];1)]]=System`InternalContinuationSome[]"###,
        &["sweep"],
        None,
        "List[]",
    );
}

#[test]
fn director_07() {
    equality(
        "s7.director_07",
        r###"l={0,0};"###,
        r###"{l[[(InputString["i"];2)]]=System`InternalContinuationNone[], l}"###,
        &["i"],
        Some("List[InternalContinuationNone[], List[0, InternalContinuationNone[]]]"),
        "List[]",
    );
}

#[test]
fn q5_director_08() {
    equality(
        "s7.q5_director_08",
        r###"a=<|"k"->0|>;"###,
        r###"{a[[(InputString["i"];1)]]=System`InternalContinuationSome[5], a}"###,
        &["i"],
        None,
        "List[]",
    );
}

#[test]
fn director_09() {
    equality(
        "s7.director_09",
        r###"l={0,0,0};"###,
        r###"{l[[(InputString["i"];2)]]=7, l}"###,
        &["i"],
        Some("List[7, List[0, 7, 0]]"),
        "List[]",
    );
}

#[test]
fn director_10() {
    equality(
        "s7.director_10",
        r###"l={{0,0},{0,0}};"###,
        r###"{l[[(InputString["i"];2),(InputString["j"];1)]]=7, l}"###,
        &["i", "j"],
        Some("List[7, List[List[0, 0], List[7, 0]]]"),
        "List[]",
    );
}

#[test]
fn q5_director_11() {
    equality(
        "s7.q5_director_11",
        r###"a=<|"k"->0,"j"->1|>;"###,
        r###"{a[[(InputString["i"];2)]]=9, a}"###,
        &["i"],
        None,
        "List[]",
    );
}

#[test]
fn director_12() {
    equality(
        "s7.director_12",
        r###"a=<|"k"->0|>;"###,
        r###"{a[(InputString["k"];"k")]=5, a}"###,
        &["k"],
        None,
        "List[]",
    );
}

#[test]
fn director_13() {
    equality(
        "s7.director_13",
        r###"a=<|"k"->0|>;"###,
        r###"{a[(InputString["k"];"new")]=5, a}"###,
        &["k"],
        None,
        "List[]",
    );
}

#[test]
fn director_14() {
    equality(
        "s7.director_14",
        r###"a=<|"k"-><|"j"->0|>|>;"###,
        r###"{a[["k",(InputString["k"];"j")]]=5, a}"###,
        &["k"],
        None,
        "List[]",
    );
}

#[test]
fn director_15() {
    equality(
        "s7.director_15",
        r###"x={1,2};"###,
        r###"{x[[(InputString["i"];1)]]+=10, x}"###,
        &["i"],
        Some("List[11, List[11, 2]]"),
        "List[]",
    );
}

#[test]
fn director_16() {
    equality(
        "s7.director_16",
        r###"x={1,2};"###,
        r###"{x[[(InputString["i"];2)]]*=3, x}"###,
        &["i"],
        Some("List[6, List[1, 6]]"),
        "List[]",
    );
}

#[test]
fn director_17() {
    equality(
        "s7.director_17",
        r###"x={1,2};"###,
        r###"{x[[(InputString["i"];1)]]++, x}"###,
        &["i"],
        Some("List[1, List[2, 2]]"),
        "List[]",
    );
}

#[test]
fn director_18() {
    equality(
        "s7.director_18",
        r###"x={1,2};"###,
        r###"{++x[[(InputString["i"];1)]], x}"###,
        &["i"],
        Some("List[2, List[2, 2]]"),
        "List[]",
    );
}

#[test]
fn director_19() {
    equality(
        "s7.director_19",
        r###"x={{1},{2}};"###,
        r###"{AppendTo[x[[(InputString["i"];1)]], 5], x}"###,
        &["i"],
        Some("List[List[1, 5], List[List[1, 5], List[2]]]"),
        "List[]",
    );
}

#[test]
fn director_20() {
    equality(
        "s7.director_20",
        r###"x={{1},{2}};"###,
        r###"{PrependTo[x[[(InputString["i"];2)]], 0], x}"###,
        &["i"],
        Some("List[List[0, 2], List[List[1], List[0, 2]]]"),
        "List[]",
    );
}

#[test]
fn director_21() {
    equality(
        "s7.director_21",
        r###"x={1,2};"###,
        r###"{x[[(InputString["i"];1)]]+=(InputString["v"];5), x}"###,
        &["v", "i"],
        Some("List[6, List[6, 2]]"),
        "List[]",
    );
}

#[test]
fn director_22() {
    equality(
        "s7.director_22",
        r###"a=<||>;"###,
        r###"{AssociateTo[a, (InputString["k"];"k"->1)], a}"###,
        &["k"],
        None,
        "List[]",
    );
}

#[test]
fn director_23() {
    equality(
        "s7.director_23",
        r###"a=<|"k"->1,"j"->2|>;"###,
        r###"{KeyDropFrom[a, (InputString["k"];"k")], a}"###,
        &["k"],
        None,
        "List[]",
    );
}

#[test]
fn director_24() {
    equality(
        "s7.director_24",
        r###"a=<|"k"-><|"j"->1|>|>;"###,
        r###"{AssociateTo[a[[(InputString["s"];"k")]], "m"->2], a}"###,
        &["s"],
        None,
        "List[]",
    );
}

#[test]
fn director_25() {
    equality(
        "s7.director_25",
        r###"a=<|"k"-><|"j"->1,"m"->2|>|>;"###,
        r###"{KeyDropFrom[a[[(InputString["s"];"k")]], "j"], a}"###,
        &["s"],
        None,
        "List[]",
    );
}

#[test]
fn director_26() {
    equality(
        "s7.director_26",
        r###"a=<|"k"->1|>;"###,
        r###"{AssociateTo[a, (InputString["k"];System`InternalContinuationNone[])], a}"###,
        &["k"],
        None,
        "List[]",
    );
}

#[test]
fn director_27() {
    equality(
        "s7.director_27",
        r###"a=<|"k"->1|>;"###,
        r###"{KeyDropFrom[a, (InputString["k"];System`InternalContinuationSome[5])], a}"###,
        &["k"],
        None,
        "List[]",
    );
}

#[test]
fn director_28() {
    equality(
        "s7.director_28",
        r###"l=Table[0,{3}];"###,
        r###"Do[l[[(InputString["i"];i)]]=i, {i,3}]; l"###,
        &["i", "i", "i"],
        Some("List[1, 2, 3]"),
        "List[]",
    );
}

#[test]
fn director_29() {
    equality(
        "s7.director_29",
        r###"x={0};"###,
        r###"Do[x[[(InputString["i"];1)]]+=1, {3}]; x"###,
        &["i", "i", "i"],
        Some("List[3]"),
        "List[]",
    );
}

#[test]
fn director_30() {
    equality(
        "s7.director_30",
        r###"a=<||>;"###,
        r###"Do[a[(InputString["k"];i)]=i, {i,3}]; a"###,
        &["k", "k", "k"],
        None,
        "List[]",
    );
}

#[test]
fn director_31() {
    equality(
        "s7.director_31",
        r###"l={0};"###,
        r###"{l[[1]]=(InputString["v"];System`InternalContinuationNone[]), l}"###,
        &["v"],
        None,
        "List[]",
    );
}

#[test]
fn director_32() {
    equality(
        "s7.director_32",
        r###"a=<|"k"->0|>;"###,
        r###"{a["k"]=(InputString["v"];System`InternalContinuationSome[5]), a}"###,
        &["v"],
        None,
        "List[]",
    );
}
