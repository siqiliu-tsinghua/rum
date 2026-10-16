use super::equality;

#[test]
fn s7_list_part() {
    equality(
        "s7.list-part",
        r###"s5Tick=1;"###,
        r###"l={0};l[[(InputString["selector"];1)]]=9;l[[1]]"###,
        &["selector"],
        Some("9"),
        "List[]",
    );
}

#[test]
fn s7_assoc_part() {
    equality(
        "s7.assoc-part",
        r###"s5Tick=1;"###,
        r###"a=<|"k"->0|>;a[[(InputString["selector"];"k")]]=9;a["k"]"###,
        &["selector"],
        Some("9"),
        "List[]",
    );
}

#[test]
fn s7_assoc_call() {
    equality(
        "s7.assoc-call",
        r###"s5Tick=1;"###,
        r###"a=<|"k"->0|>;a[(InputString["selector"];"k")]=9;a["k"]"###,
        &["selector"],
        Some("9"),
        "List[]",
    );
}

#[test]
fn s7_assoc_call_thrice() {
    equality(
        "s7.assoc-call-thrice",
        r###"s5Tick=1;"###,
        r###"a=<|"k"-><|"j"-><|"z"->0|>|>|>;a[(InputString["first"];"k"),(InputString["later"];"j"),(InputString["last"];"z")]=9;a[["k","j","z"]]"###,
        &["first", "later", "last"],
        Some("9"),
        "List[]",
    );
}

#[test]
fn s7_held_twice() {
    equality(
        "s7.held-twice",
        r###"s5Tick=1;"###,
        r###"x={{1,2},{3,4}};x[[(InputString["first"];1),(InputString["later"];2)]]+=10;x[[1,2]]"###,
        &["first", "later"],
        Some("12"),
        "List[]",
    );
}

#[test]
fn s7_held_rhs() {
    equality(
        "s7.held-rhs",
        r###"s5Tick=1;"###,
        r###"x={1};x[[1]] //= Function[z,(InputString["rhs"];z+2)];x[[1]]"###,
        &["rhs"],
        Some("3"),
        "List[]",
    );
}

#[test]
fn s7_associate_argument() {
    equality(
        "s7.associate-argument",
        r###"s5Tick=1;"###,
        r###"a=<||>;AssociateTo[a,(InputString["argument"];"k"->9)];a["k"]"###,
        &["argument"],
        Some("9"),
        "List[]",
    );
}

#[test]
fn s7_keydrop_argument() {
    equality(
        "s7.keydrop-argument",
        r###"s5Tick=1;"###,
        r###"a=<|"k"->1|>;KeyDropFrom[a,(InputString["argument"];"k")];Length[a]"###,
        &["argument"],
        Some("0"),
        "List[]",
    );
}

#[test]
fn s7_associate_target() {
    equality(
        "s7.associate-target",
        r###"s7a[i_]:=(InputString["target"];<|"k"->1|>);s5Tick=1;"###,
        r###"Length[AssociateTo[s7a[s5Tick],"j"->2]]"###,
        &["target"],
        Some("2"),
        "List[]",
    );
}

#[test]
fn s7_keydrop_target() {
    equality(
        "s7.keydrop-target",
        r###"s7a[i_]:=(InputString["target"];<|"k"->1,"j"->2|>);s5Tick=1;"###,
        r###"Length[KeyDropFrom[s7a[s5Tick],"j"]]"###,
        &["target"],
        Some("1"),
        "List[]",
    );
}

#[test]
fn s7_associate_held() {
    equality(
        "s7.associate-held",
        r###"s5Tick=1;"###,
        r###"a=<|"k"-><|"j"->1|>|>;AssociateTo[a[[(InputString["selector"];"k")]],"m"->9];a[["k","m"]]"###,
        &["selector"],
        Some("9"),
        "List[]",
    );
}

#[test]
fn s7_keydrop_held() {
    equality(
        "s7.keydrop-held",
        r###"s5Tick=1;"###,
        r###"a=<|"k"-><|"j"->1,"m"->2|>|>;KeyDropFrom[a[[(InputString["selector"];System`Key["k"])]],"j"];Length[a["k"]]"###,
        &["selector"],
        Some("1"),
        "List[]",
    );
}

#[test]
fn held_part_twice_order() {
    equality(
        "held.part.twice",
        r###"x={{1,2},{3,4}};"###,
        r###"x[[(AppendTo[Global`log,1];Print["first"];InputString["first"];1),(AppendTo[Global`log,2];InputString["later"];2)]]+=10;x[[1,2]]"###,
        &["first", "later"],
        Some("12"),
        "List[1, 2]",
    );
}

#[test]
fn frozen_target_during_update() {
    equality(
        "s7.frozen-target",
        r###"x={1};"###,
        r###"x[[1]] //= Function[z,(InputString["rhs"];x={99};z+2)];x[[1]]"###,
        &["rhs"],
        Some("3"),
        "List[]",
    );
}

#[test]
fn selector_throw_after_resume() {
    equality(
        "s7.selector-throw",
        r###"l={0};"###,
        r###"Catch[l[[(InputString["selector"];Throw[7])]]=9]"###,
        &["selector"],
        Some("7"),
        "List[]",
    );
}
