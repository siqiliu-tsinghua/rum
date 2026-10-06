//! Separate factories cover phase transitions and owners entered from resume.
use super::super::super::descriptors::{Fixture, Variant};
use super::super::primary::Descriptor;
use super::support::*;

pub fn descriptors() -> Vec<Descriptor> {
    let rows: &[(&str, &str, fn(&str, Variant) -> Fixture)] = &[
        (
            "FoldPair.callback/projector-after-resume",
            "FoldPair.callback",
            pair_cross,
        ),
        (
            "FoldWhile.callback/test-after-resume",
            "FoldWhile.callback",
            while_cross,
        ),
        (
            "FoldWhile.callback/positive-tail",
            "FoldWhile.callback",
            while_tail,
        ),
        (
            "SubsetMap.callback/rebuild",
            "SubsetMap.callback",
            |e, v| subset(e, v, false),
        ),
        (
            "SubsetMap.callback/rebuild-after-resume",
            "SubsetMap.callback",
            |e, v| subset(e, v, true),
        ),
        (
            "ArrayFilter.defaultOption/callback-after-resume",
            "ArrayFilter.defaultOption",
            option_cross,
        ),
        ("Dot.arithmetic/later-operand", "Dot.arithmetic", dot_later),
        (
            "Inner.arithmetic/term-to-reduction",
            "Inner.arithmetic",
            inner_cross,
        ),
        (
            "Inner.arithmetic/contracted-prefix",
            "Inner.arithmetic",
            contracted_prefix,
        ),
        (
            "Dot.arithmetic/contracted-prefix",
            "Dot.arithmetic",
            contracted_prefix,
        ),
    ];
    rows.iter()
        .map(|&(id, entry, factory)| Descriptor {
            id,
            entry,
            factory,
            variants: VARIANTS,
            boundary: false,
            lifecycle: super::lifecycle::exercise,
        })
        .collect()
}
fn pair_cross(entry: &str, v: Variant) -> Fixture {
    // Second iteration: callback suspends at 3, then its projector at 4.
    let mut f = base_at(entry, v, 10, [3, 4]);
    let callback = visits(&f, v, 10, "{10 (x+y),x+y}");
    let projector = visits(&f, v, 10, "Total[p]");
    f.source =
        format!("FoldPair[Function[{{x,y}},{callback}],0,{{1,2,3,4,5}},Function[p,{projector}]]");
    finish(f, v, "165")
}
fn while_cross(entry: &str, v: Variant) -> Fixture {
    let mut f = base_at(entry, v, 10, [3, 4]);
    let callback = visits(&f, v, 10, "x+y");
    let predicate = visits(&f, v, 10, "True");
    f.source =
        format!("FoldWhile[Function[{{x,y}},{callback}],0,{{1,2,3,4,5}},Function[x,{predicate}]]");
    finish(f, v, "15")
}
fn while_tail(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v, 5);
    let callback = visits(&f, v, 5, "x+y");
    f.source = format!(
        "FoldWhile[Function[{{x,y}},If[y==0,x,{callback}]],0,{{0,1,2,3,4,5}},Function[x,False],1,5]"
    );
    finish(f, v, "15")
}
fn subset(entry: &str, v: Variant, cross: bool) -> Fixture {
    let mut f = base(entry, v, 5);
    let (callback, rebuild) = if cross {
        let a = (1..=3)
            .map(|i| step(&f, v, i, "Reverse[x]", false))
            .collect::<Vec<_>>()
            .join(";");
        let b = (4..=5)
            .map(|i| step(&f, v, i, "73", i == 5))
            .collect::<Vec<_>>()
            .join(";");
        (format!("Function[x,({a})]"), format!("({b})"))
    } else {
        ("Reverse".into(), sequence(&f, v, "73"))
    };
    f.setup
        .push_str(&format!(";Global`h[5,4,3,2,1]:={rebuild}"));
    f.source =
        format!("SubsetMap[{callback},Global`h[1,2,3,4,5],{{{{1}},{{2}},{{3}},{{4}},{{5}}}}]");
    finish(f, v, "73")
}
fn option_cross(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v, 5);
    let option = (1..=3)
        .map(|i| step(&f, v, i, "0", false))
        .collect::<Vec<_>>()
        .join(";");
    let callback = (4..=5)
        .map(|i| step(&f, v, i, "Total[x]", i == 5))
        .collect::<Vec<_>>()
        .join(";");
    f.setup.push_str(&format!(
        ";Unprotect[ArrayFilter];SetOptions[ArrayFilter,Padding:>({option})]"
    ));
    f.source = format!(
        "ArrayFilter[Function[x,(Global`calls++;If[Global`calls==3,({callback}),Total[x]])],{{1,2,3,4,5}},1]"
    );
    project(
        &mut f,
        "Global`calls",
        if v == Variant::Error { "3" } else { "5" },
    );
    finish(f, v, "List[3, 6, 9, 12, 9]")
}
fn dot_later(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v, 5);
    let body = visits(&f, v, 5, "x y");
    f.setup.push_str(&format!(
        ";Global`a /: Times[Global`a,x_]:=Global`b[x];Global`b /: Times[Global`b[x_],y_]:={body}"
    ));
    f.source =
        "Dot[{{Global`a},{Global`a},{Global`a},{Global`a},{Global`a}},{{2}},{{3}},{{10}}]".into();
    finish(
        f,
        v,
        "List[List[60], List[60], List[60], List[60], List[60]]",
    )
}
fn inner_cross(entry: &str, v: Variant) -> Fixture {
    let mut f = base_at(entry, v, 15, [5, 6]);
    let term = visits(&f, v, 15, "x y");
    let reduce = visits(&f, v, 15, "x+y");
    f.source = format!(
        "Inner[Function[{{x,y}},{term}],{{{{1,2}},{{2,3}},{{3,4}},{{4,5}},{{5,6}}}},{{{{10}},{{20}}}},Function[{{x,y}},{reduce}]]"
    );
    finish(
        f,
        v,
        "List[List[50], List[80], List[110], List[140], List[170]]",
    )
}

fn contracted_prefix(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v, 5);
    let body = visits(&f, v, 5, "x y");
    if entry == "Inner.arithmetic" {
        f.source = format!("Inner[Function[{{x,y}},{body}],{{1,2,3,4,5}},{{10,20,30,40,50}},Plus]");
        finish(f, v, "550")
    } else {
        let body = visits(&f, v, 5, "x");
        f.setup
            .push_str(&format!(";Global`a /: Times[Global`a,x_]:={body}"));
        f.source =
            "Dot[{{Global`a,Global`a,Global`a,Global`a,Global`a}},{{1},{2},{3},{4},{5}},{{10}}]"
                .into();
        finish(f, v, "List[List[150]]")
    }
}
