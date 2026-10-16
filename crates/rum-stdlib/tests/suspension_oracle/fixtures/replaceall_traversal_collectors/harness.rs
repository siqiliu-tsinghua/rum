use super::equality;

#[test]
fn s8_replaceall_node() {
    equality(
        "s8.s8_replaceall_node",
        r###""###,
        r###"f[1] /. x_Integer /; (InputString["guard"];True) :> x+1"###,
        &["guard"],
        r###"f[2]"###,
    );
}

#[test]
fn s8_replaceall_atom() {
    equality(
        "s8.s8_replaceall_atom",
        r###""###,
        r###"1 /. x_Integer /; (InputString["guard"];True) :> x+1"###,
        &["guard"],
        r###"2"###,
    );
}

#[test]
fn s8_replaceall_twice() {
    equality(
        "s8.s8_replaceall_twice",
        r###""###,
        r###"{1,2} /. x_Integer /; (InputString["guard"];True) :> x+1"###,
        &["guard", "guard"],
        r###"List[2, 3]"###,
    );
}

#[test]
fn s8_repeated_guard() {
    equality(
        "s8.s8_repeated_guard",
        r###""###,
        r###"ReplaceRepeated[f[1], f[1] /; (InputString["guard"];True) :> f[0]]"###,
        &["guard"],
        r###"f[0]"###,
    );
}

#[test]
fn s8_replace_decline() {
    equality(
        "s8.s8_replace_decline",
        r###""###,
        r###"Replace[3,{x_ /; (InputString["first"];False) :> 0,x_ :> 9}]"###,
        &["first"],
        r###"9"###,
    );
}

#[test]
fn s8_replace_both() {
    equality(
        "s8.s8_replace_both",
        r###""###,
        r###"Replace[3,{x_ /; (InputString["first"];False) :> 0,x_ /; (InputString["later"];True) :> 9}]"###,
        &["first", "later"],
        r###"9"###,
    );
}

#[test]
fn s8_replaceat_guard() {
    equality(
        "s8.s8_replaceat_guard",
        r###""###,
        r###"ReplaceAt[{1,2,3},x_ /; (InputString["guard"];True) :> x+1,2]"###,
        &["guard"],
        r###"List[1, 3, 3]"###,
    );
}

#[test]
fn s8_count_target() {
    equality(
        "s8.s8_count_target",
        r###""###,
        r###"Count[(InputString["target"];{1}),_Integer]"###,
        &["target"],
        r###"1"###,
    );
}

#[test]
fn s8_count_pattern() {
    equality(
        "s8.s8_count_pattern",
        r###""###,
        r###"Count[{1},(InputString["pattern"];_Integer)]"###,
        &["pattern"],
        r###"1"###,
    );
}

#[test]
fn s8_memberq_target() {
    equality(
        "s8.s8_memberq_target",
        r###""###,
        r###"MemberQ[(InputString["target"];{1}),_Integer]"###,
        &["target"],
        r###"True"###,
    );
}

#[test]
fn s8_memberq_pattern() {
    equality(
        "s8.s8_memberq_pattern",
        r###""###,
        r###"MemberQ[{1},(InputString["pattern"];_Integer)]"###,
        &["pattern"],
        r###"True"###,
    );
}

#[test]
fn s8_freeq_target() {
    equality(
        "s8.s8_freeq_target",
        r###""###,
        r###"FreeQ[(InputString["target"];{1}),_Integer]"###,
        &["target"],
        r###"False"###,
    );
}

#[test]
fn s8_freeq_pattern() {
    equality(
        "s8.s8_freeq_pattern",
        r###""###,
        r###"FreeQ[{1},(InputString["pattern"];_Integer)]"###,
        &["pattern"],
        r###"False"###,
    );
}

#[test]
fn s8_count_level() {
    equality(
        "s8.s8_count_level",
        r###""###,
        r###"Count[{1},_Integer,(InputString["level"];{1})]"###,
        &["level"],
        r###"1"###,
    );
}

#[test]
fn s8_count_guard_twice() {
    equality(
        "s8.s8_count_guard_twice",
        r###""###,
        r###"Count[{1,2},x_Integer /; (InputString["guard"];True)]"###,
        &["guard", "guard"],
        r###"2"###,
    );
}

#[test]
fn s8_position_target() {
    equality(
        "s8.s8_position_target",
        r###""###,
        r###"Position[(InputString["target"];{1}),1]"###,
        &["target"],
        r###"List[List[1]]"###,
    );
}

#[test]
fn s8_position_pattern() {
    equality(
        "s8.s8_position_pattern",
        r###""###,
        r###"Position[{1},(InputString["pattern"];1)]"###,
        &["pattern"],
        r###"List[List[1]]"###,
    );
}

#[test]
fn s8_position_level() {
    equality(
        "s8.s8_position_level",
        r###""###,
        r###"Position[{1},1,(InputString["level"];{1})]"###,
        &["level"],
        r###"List[List[1]]"###,
    );
}

#[test]
fn s8_position_guard_twice() {
    equality(
        "s8.s8_position_guard_twice",
        r###""###,
        r###"Position[{1,2},x_Integer /; (InputString["guard"];True)]"###,
        &["guard", "guard"],
        r###"List[List[1], List[2]]"###,
    );
}
