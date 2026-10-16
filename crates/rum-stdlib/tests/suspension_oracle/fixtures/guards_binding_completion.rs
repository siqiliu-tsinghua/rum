//! S6 guards preserve bindings, ordering, and every observable output channel.
//! Contract: guide definitions/rules; queued input equals resumed input.
use super::super::descriptors::{Checks, accept};

fn equality(id: &str, setup: &str, source: &str, prompts: &[&str], value: &str, log: &str) {
    let pauses: Vec<_> = prompts
        .iter()
        .map(|prompt| ("InputString", *prompt, "x"))
        .collect();
    let mut fixture = super::make(&format!("s6.{id}"), source, &pauses, Some(value), log);
    fixture.setup = setup.into();
    let result = accept(&fixture, Checks::ALL);
    super::super::support::faults::assert_all();
    let accepted = result.unwrap_or_else(|error| panic!("s6.{id}: {error:?}"));
    println!(
        "S6 {id} pauses={} strict equality PASS: {:?}; log={}",
        prompts.len(),
        accepted.observation,
        accepted.log
    );
}

macro_rules! director_shape {
    ($name:ident, $setup:literal, $source:literal, $prompts:expr, $value:literal) => {
        #[test]
        fn $name() {
            equality(
                stringify!($name),
                $setup,
                $source,
                $prompts,
                $value,
                "List[]",
            );
        }
    };
}

director_shape!(
    single_guard,
    r#"f[x_]:=x+1/;(InputString["g"];True);"#,
    "f[1]",
    &["g"],
    "2"
);
director_shape!(
    false_guard,
    r#"f[x_]:=1/;(InputString["g"];False);f[x_]:=2;"#,
    "f[1]",
    &["g"],
    "2"
);
director_shape!(
    non_boolean_guard,
    r#"f[x_]:=1/;(InputString["g"];7);f[x_]:=2;"#,
    "f[1]",
    &["g"],
    "2"
);
director_shape!(
    nested_guards,
    r#"f[x_]:=(x/;(InputString["a"];True))/;(InputString["b"];True);"#,
    "f[5]",
    &["a", "b"],
    "5"
);
director_shape!(
    nested_false,
    r#"f[x_]:=(x/;(InputString["a"];False))/;(InputString["b"];True);f[x_]:=0;"#,
    "f[5]",
    &["a"],
    "0"
);
director_shape!(
    multiple_bindings,
    r#"f[x_,y_]:={x,y}/;(InputString["p"];x<y);"#,
    "{f[1,2],f[2,1]}",
    &["p"; 3],
    "List[List[1, 2], f[2, 1]]"
);
director_shape!(
    table_guards,
    r#"f[x_]:=x/;(InputString["p"];OddQ[x]);f[x_]:=0;"#,
    "Table[f[i],{i,3}]",
    &["p"; 3],
    "List[1, 0, 3]"
);
director_shape!(
    left_then_right_true,
    r#"f[x_/;(InputString["a"];True)]:=x/;(InputString["b"];True);"#,
    "f[4]",
    &["a", "b"],
    "4"
);
director_shape!(
    left_then_right_false,
    r#"f[x_/;(InputString["a"];True)]:=x/;(InputString["b"];False);f[x_]:=-1;"#,
    "f[4]",
    &["a", "b"],
    "-1"
);
director_shape!(
    match_true,
    "",
    r#"MatchQ[3,x_/;(InputString["p"];x>2)]"#,
    &["p"],
    "True"
);
director_shape!(
    match_false,
    "",
    r#"MatchQ[1,x_/;(InputString["p"];x>2)]"#,
    &["p"],
    "False"
);
director_shape!(
    cases,
    "",
    r#"Cases[{1,2,3},x_/;(InputString["p"];x>1)]"#,
    &["p"; 3],
    "List[2, 3]"
);
director_shape!(
    replace_all,
    "",
    r#"{1,2,3}/.x_Integer:>10x/;(InputString["p"];x>1)"#,
    &["p"; 3],
    "List[1, 20, 30]"
);
director_shape!(
    replace_part,
    "",
    r#"ReplacePart[{1,2,3},{i_}:>0/;(InputString["p"];i>1)]"#,
    &["p"; 3],
    "List[1, 0, 0]"
);
director_shape!(
    string_replace,
    "",
    r#"StringReplace["abc",x_:>"X"/;(InputString["p"];x=="b")]"#,
    &["p"; 3],
    "\"aXc\""
);
director_shape!(
    own_true,
    r#"v:=5/;(InputString["p"];True);"#,
    "v",
    &["p"],
    "5"
);
director_shape!(
    own_false,
    r#"v:=5/;(InputString["p"];False);"#,
    "v",
    &["p"],
    "v"
);
director_shape!(
    default_guard,
    r#"Default[d]:=1/;(InputString["p"];True);d[x_.]:={x};"#,
    "d[]",
    &["p"],
    "List[1]"
);
director_shape!(
    map_fallback,
    r#"g[x_]:=x/;(InputString["p"];x>0);g[x_]:=-x;"#,
    "Map[g,{1,-2,3}]",
    &["p"; 3],
    "List[1, 2, 3]"
);
director_shape!(
    subvalue,
    r#"k[1][x_]:=x/;(InputString["p"];True);"#,
    "k[1][7]",
    &["p"],
    "7"
);
director_shape!(
    upvalue,
    r#"h/:u[h,x_]:=x/;(InputString["p"];True);"#,
    "u[h,9]",
    &["p"],
    "9"
);
director_shape!(
    nested_calls,
    r#"f[x_]:=x/;(InputString["p"];True);"#,
    "f[f[2]]",
    &["p"; 2],
    "2"
);

// F0266 resumes once and inserts F0267; the guard's bindings remain available.
#[test]
fn cross_type_twice() {
    equality(
        "cross-type-twice",
        r#"f[x_/;(AppendTo[Global`log,1];Print["left"];InputString["left"];True)]:=x/;(AppendTo[Global`log,x];Print["right"];InputString["right"];True);"#,
        "f[4]",
        &["left", "right"],
        "4",
        "List[1, 4]",
    );
}

// The first target pauses in F0267 twice; the second resumes F0268 twice.
#[test]
fn readonly_token_twice() {
    equality(
        "readonly-token-twice",
        "",
        r#"ReplacePart[{1,2},{i_}:>(0/;(AppendTo[Global`log,i];Print[i];InputString["first"];True))/;(AppendTo[Global`log,-i];InputString["later"];True)]"#,
        &["first", "later", "first", "later"],
        "List[0, 0]",
        "List[2, -2, 1, -1]",
    );
}

// F0269 maps each resumed CompiledMatchOutcome to bool for StringReplace.
#[test]
fn string_route_twice() {
    equality(
        "string-route-twice",
        "",
        r#"StringReplace["ab",x_:>("X"/;(AppendTo[Global`log,x];Print[x];InputString["string"];True))]"#,
        &["string"; 2],
        "\"XX\"",
        "List[\"a\", \"b\"]",
    );
}
