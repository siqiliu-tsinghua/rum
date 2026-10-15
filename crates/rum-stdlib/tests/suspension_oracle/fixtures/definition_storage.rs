//! S5 application and storage continuations preserve ordered side effects.
use super::super::descriptors::{Checks, accept};

fn check(name: &str, source: &str, prompts: &[&str], expected: &str) {
    let pauses: Vec<_> = prompts
        .iter()
        .map(|prompt| ("InputString", *prompt, "1"))
        .collect();
    let id = format!("s5.{name}");
    let fixture = super::make(&id, source, &pauses, Some(expected), "List[]");
    let result = accept(&fixture, Checks::ALL);
    super::super::support::faults::assert_all();
    result.unwrap_or_else(|error| panic!("{id}: {error:?}"));
    println!("S5 {id} strict equality PASS");
}

#[test]
fn s5_down_rhs() {
    check(
        "s5_down_rhs",
        r#"f[x_]:=(InputString["rhs"];7);f[1]"#,
        &["rhs"],
        "7",
    );
}

#[test]
fn s5_down_guard() {
    check(
        "s5_down_guard",
        r#"f[0]=0;f[1]=1;f[2]=2;f[3]=3;f[4]=4;f[5]=5;f[6]=6;f[7]=7;f[8]=8;f[9]=9;f[10]=10;f[11]=11;f[12]=12;f[13]=13;f[14]=14;f[15]=15;f[16]=16;f[17]=17;f[18]=18;f[19]=19;f[20]=20;f[21]=21;f[22]=22;f[23]=23;f[24]=24;f[25]=25;f[26]=26;f[27]=27;f[28]=28;f[29]=29;f[30]=30;f[31]=31;f[x_ /; (InputString["guard"];True)]:=(InputString["rhs"];7);f[100]"#,
        &["guard", "rhs"],
        "7",
    );
}

#[test]
fn s5_sub_rhs() {
    check(
        "s5_sub_rhs",
        r#"k[1][x_]:=(InputString["rhs"];7);k[1][2]"#,
        &["rhs"],
        "7",
    );
}

#[test]
fn s5_sub_guard() {
    check(
        "s5_sub_guard",
        r#"k[1][x_ /; (InputString["guard"];True)]:=(InputString["rhs"];7);k[1][2]"#,
        &["guard", "rhs"],
        "7",
    );
}

#[test]
fn s5_up_rhs() {
    check(
        "s5_up_rhs",
        r#"g /: h[g,x_]:=(InputString["rhs"];7);h[g,2]"#,
        &["rhs"],
        "7",
    );
}

#[test]
fn s5_up_guard() {
    check(
        "s5_up_guard",
        r#"g /: h[x_ /; (InputString["guard"];True),g]:=(InputString["rhs"];7);h[2,g]"#,
        &["guard", "rhs"],
        "7",
    );
}

#[test]
fn s5_default_rhs() {
    check(
        "s5_default_rhs",
        r#"Default[d]:=(InputString["rhs"];7);d[x_.]:={x};d[]"#,
        &["rhs"],
        "List[7]",
    );
}

#[test]
fn s5_default_guard() {
    check(
        "s5_default_guard",
        r#"Default[d]:=(InputString["rhs"];7) /; (InputString["guard"];True);Default[d]"#,
        &["guard", "rhs"],
        "7",
    );
}

#[test]
fn s5_default_fallback_rhs() {
    check(
        "s5_default_fallback_rhs",
        r#"Default[d]:=(InputString["rhs"];7);Default[d]"#,
        &["rhs"],
        "7",
    );
}

#[test]
fn s5_set() {
    check("s5_set", r#"v=(InputString["rhs"];7)"#, &["rhs"], "7");
}

#[test]
fn s5_upset() {
    check("s5_upset", r#"h[g]^=(InputString["rhs"];7)"#, &["rhs"], "7");
}

#[test]
fn s5_tagset() {
    check(
        "s5_tagset",
        r#"g /: h[g]=(InputString["rhs"];7)"#,
        &["rhs"],
        "7",
    );
}

#[test]
fn s5_rule_rhs() {
    check(
        "s5_rule_rhs",
        r#"(1 -> (InputString["rhs"];7)) === (1 -> 7)"#,
        &["rhs"],
        "True",
    );
}

#[test]
fn s5_rule_lhs() {
    check(
        "s5_rule_lhs",
        r#"(f[x_,(InputString["lhs"];1)] -> 7) === (f[x_,1] -> 7)"#,
        &["lhs"],
        "True",
    );
}

#[test]
fn s5_destructure() {
    check(
        "s5_destructure",
        r#"a={0};b=0;{a[[(InputString["lhs"];1)]],b}={7,8}"#,
        &["lhs"],
        "List[7, 8]",
    );
}

#[test]
fn s5_lhs_twice() {
    check(
        "s5_lhs_twice",
        r#"f[(InputString["first"];1),(InputString["later"];2)]=7"#,
        &["first", "later"],
        "7",
    );
}

#[test]
fn s5_own_rhs() {
    check(
        "s5_own_rhs",
        r#"v:=(InputString["rhs"];7);v"#,
        &["rhs"],
        "7",
    );
}

#[test]
fn s5_destructure_twice() {
    check(
        "s5_destructure_twice",
        r#"a={0,0};{a[[(InputString["first"];1)]],a[[(InputString["later"];2)]]}={7,8}"#,
        &["first", "later"],
        "List[7, 8]",
    );
}

#[test]
fn s5_rule_lhs_twice() {
    check(
        "s5_rule_lhs_twice",
        r#"(f[x_,(InputString["first"];1),(InputString["later"];2)] -> 7) === (f[x_,1,2] -> 7)"#,
        &["first", "later"],
        "True",
    );
}
