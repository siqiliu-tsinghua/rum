use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let data = data();
    let (source, expected) = match entry {
        "Association.keyValue" => {
            // Unevaluated hands intact rules to Association's local key loop.
            let rules = (1..=5)
                .map(|i| {
                    format!(
                        "Unevaluated[RuleDelayed[{},{}]]",
                        step(&f, v, i, &i.to_string()),
                        i * 10
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            (
                format!("Association[{rules}]"),
                format!(
                    "Association[{}]",
                    (1..=5)
                        .map(|i| format!("RuleDelayed[{i}, {}]", i * 10))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        }
        "KeyMap.callback" => {
            // Rule-LHS normalization traverses a literal Function as pattern
            // syntax. A named callback makes its body run on actual invocation.
            f.setup.push_str(&format!(
                ";Global`keyCallback[x_]:={}",
                visits(&f, v, "x+10")
            ));
            (
                format!("KeyMap[Global`keyCallback,{data}]"),
                association((1..=5).map(|i| (i + 10, (i * 10).to_string()))),
            )
        }
        "KeyValueMap.callback" => (
            format!(
                "KeyValueMap[Function[{{x,y}},{}],{data}]",
                visits(&f, v, "x+y")
            ),
            "List[11, 22, 33, 44, 55]".into(),
        ),
        "KeyGroupBy.callback" => (
            format!(
                "KeyGroupBy[{data},Function[x,{}]]",
                visits(&f, v, "Mod[x,2]")
            ),
            format!(
                "Association[Rule[1, {}], Rule[0, {}]]",
                association([1, 3, 5].map(|i| (i, (i * 10).to_string()))),
                association([2, 4].map(|i| (i, (i * 10).to_string())))
            ),
        ),
        "KeySelect.predicate" => (
            format!(
                "KeySelect[{data},Function[x,{}]]",
                visits(
                    &f,
                    v,
                    if v == Variant::Decline {
                        "x!=2"
                    } else {
                        "True"
                    }
                )
            ),
            association(
                (1..=5)
                    .filter(|&i| v != Variant::Decline || i != 2)
                    .map(|i| (i, (i * 10).to_string())),
            ),
        ),
        "KeySortBy.keyFunction" => (
            format!("KeySortBy[{data},Function[x,{}]]", visits(&f, v, "-x")),
            association((1..=5).rev().map(|i| (i, (i * 10).to_string()))),
        ),
        "AssociationMap.callback" => (
            format!(
                "AssociationMap[Function[x,{}],{{1,2,3,4,5}}]",
                visits(&f, v, "10*x")
            ),
            data,
        ),
        "CountsBy.keyFunction" => (
            format!(
                "CountsBy[{{1,2,3,4,5}},Function[x,{}]]",
                visits(&f, v, "Mod[x,2]")
            ),
            "Association[Rule[1, 3], Rule[0, 2]]".into(),
        ),
        "KeyUnion.missingValue" => (
            format!(
                "KeyUnion[{{<||>,{data}}},Function[x,{}]]",
                visits(&f, v, "x+100")
            ),
            format!(
                "List[{}, {data}]",
                association((1..=5).map(|i| (i, (i + 100).to_string())))
            ),
        ),
        "Normal.lazyConsumer" => (
            format!(
                "Normal[LazyTake[LazyMap[Function[x,{}],LazyRange[1,1]],5]]",
                visits(&f, v, "10*x")
            ),
            "List[10, 20, 30, 40, 50]".into(),
        ),
        _ => unreachable!("{entry}"),
    };
    f.source = source;
    finish(f, v, &expected)
}
pub fn map_association(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    f.source = format!(
        "AssociationMap[Function[x,{}],{}]",
        visits(&f, v, "x[[1]]->(x[[2]]+1)"),
        data()
    );
    finish(
        f,
        v,
        &association((1..=5).map(|i| (i, (i * 10 + 1).to_string()))),
    )
}
pub fn comparator(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    f.setup.push_str(";Global`pairs={}");
    f.projections.push("Global`pairs".into());
    // The sort schedule is unspecified: compare all pairs between paths, while
    // requiring the first five callback operations and the sorted association.
    f.source = format!(
        "KeySort[{},Function[{{x,y}},(AppendTo[Global`pairs,{{x,y}}];If[Global`state<5,{},x<y])]]",
        association([6, 2, 5, 1, 4, 3].map(|i| (i, (i * 10).to_string()))),
        visits(&f, v, "x<y")
    );
    finish(
        f,
        v,
        &association((1..=6).map(|i| (i, (i * 10).to_string()))),
    )
}

pub fn sort_by_ordering(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    f.source = format!(
        "KeySortBy[{},Function[x,{}],Greater]",
        data(),
        visits(&f, v, "x")
    );
    finish(
        f,
        v,
        &association((1..=5).rev().map(|i| (i, (i * 10).to_string()))),
    )
}
