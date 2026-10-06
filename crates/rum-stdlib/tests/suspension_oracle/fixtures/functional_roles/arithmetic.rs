use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    tensor(entry, v, false)
}
pub fn tensor(entry: &str, v: Variant, reduction: bool) -> Fixture {
    let mut f = base(entry, v, 5);
    let name = entry.split_once('.').unwrap().0;
    let (definition, source, expected) = match name {
        "Accumulate" => (
            "Global`a /: Plus[x_,Global`a]:=BODY".to_string(),
            "Accumulate[{0,Global`a,Global`a,Global`a,Global`a,Global`a}]",
            "List[0, 1, 2, 3, 4, 5]",
        ),
        "Dot" if reduction => (
            "Global`a /: Times[Global`a,x_]:=Global`b[x];Global`b /: Plus[Global`b[x_],Global`b[y_]]:=BODY".into(),
            "Dot[{{Global`a,Global`a},{Global`a,Global`a},{Global`a,Global`a},{Global`a,Global`a},{Global`a,Global`a}},{{1},{2}},{{10}}]",
            "List[List[30], List[30], List[30], List[30], List[30]]",
        ),
        "Dot" => (
            "Global`a /: Times[Global`a,x_]:=BODY".into(),
            "Dot[{{Global`a},{Global`a},{Global`a},{Global`a},{Global`a}},{{2}},{{10}}]",
            "List[List[20], List[20], List[20], List[20], List[20]]",
        ),
        "Inner" if reduction => (
            "Global`g[x_,y_]:=BODY".into(),
            "Inner[Times,{{1,2},{2,3},{3,4},{4,5},{5,6}},{{10},{20}},Global`g]",
            "List[List[50], List[80], List[110], List[140], List[170]]",
        ),
        "Inner" => (
            "Global`f[x_,y_]:=BODY".into(),
            "Inner[Global`f,{{1},{2},{3},{4},{5}},{{10}},Plus]",
            "List[List[10], List[20], List[30], List[40], List[50]]",
        ),
        _ => unreachable!("{entry}"),
    };
    let result = if name == "Accumulate" {
        "x+1"
    } else if reduction {
        "x+y"
    } else if name == "Dot" {
        "x"
    } else {
        "x y"
    };
    f.setup.push(';');
    f.setup
        .push_str(&definition.replace("BODY", &visits(&f, v, 5, result)));
    f.source = source.into();
    finish(f, v, expected)
}

pub fn option(entry: &str, v: Variant) -> Fixture {
    if entry.ends_with("defaultOption") {
        let mut f = base(entry, v, 5);
        f.setup.push_str(&format!(
            ";Unprotect[ArrayFilter];SetOptions[ArrayFilter,Padding:>{}]",
            sequence(&f, v, "0")
        ));
        f.source = "ArrayFilter[Total,{1,2,3,4,5},1]".into();
        finish(f, v, "List[3, 6, 9, 12, 9]")
    } else {
        // Each of five eleven-cell neighborhoods contains six padding cells.
        // Delayed padding is forced during the local tensor callback's argument
        // evaluation. Pauses occur in neighborhoods three and five, after two
        // and four completed outputs; construction itself cannot force it.
        let mut f = base_at(entry, v, 30, [14, 26]);
        let body = visits(&f, v, 30, "0");
        f.source = format!("ArrayFilter[Total,{{1,2,3,4,5}},5,Padding:>{body}]");
        finish(f, v, "List[15, 15, 15, 15, 15]")
    }
}
