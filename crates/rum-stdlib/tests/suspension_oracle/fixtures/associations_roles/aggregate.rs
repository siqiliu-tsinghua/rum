use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;
pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let expected = match entry {
        "Counts.delayedValue" => {
            f.source = format!("Counts[{}]", delayed(&f, v, |i| (i % 2).to_string()));
            "Association[Rule[1, 3], Rule[0, 2]]".into()
        }
        "Catenate.delayedValue" => {
            f.source = format!(
                "Catenate[{}]",
                delayed(&f, v, |i| format!("{{{i},{}}}", i * 10))
            );
            "List[1, 10, 2, 20, 3, 30, 4, 40, 5, 50]".into()
        }
        "Values.delayedValue" => {
            f.source = format!("Values[{}]", delayed(&f, v, |i| (i * 10).to_string()));
            "List[10, 20, 30, 40, 50]".into()
        }
        "Lookup.delayedValue" => {
            f.source = format!(
                "Lookup[{},{{1,2,3,4,5}}]",
                delayed(&f, v, |i| (i * 10).to_string())
            );
            "List[10, 20, 30, 40, 50]".into()
        }
        "Merge.delayedValue" | "Merge.reducer" => {
            let a = if entry.ends_with("delayedValue") {
                delayed(&f, v, |i| i.to_string())
            } else {
                association((1..=5).map(|i| (i, i.to_string())))
            };
            let reducer = if entry.ends_with("reducer") {
                format!("Function[x,{}]", visits(&f, v, "Total[x]"))
            } else {
                "Total".into()
            };
            f.source = format!("Merge[{{{a},{}}},{reducer}]", data());
            association((1..=5).map(|i| (i, (i * 11).to_string())))
        }
        _ => unreachable!("{entry}"),
    };
    finish(f, v, &expected)
}
pub fn catenate_list(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    f.source = format!(
        "Catenate[{{{{0}},{},{{6}}}}]",
        delayed(&f, v, |i| i.to_string())
    );
    finish(f, v, "List[0, 1, 2, 3, 4, 5, 6]")
}
pub fn nested(entry: &str, v: Variant) -> Fixture {
    nested_at(entry, v, 2)
}
pub fn nested_deep(entry: &str, v: Variant) -> Fixture {
    nested_at(entry, v, 18)
}
fn nested_at(entry: &str, v: Variant, depth: usize) -> Fixture {
    let mut f = base(entry, v);
    let mut a = if entry.ends_with("delayedValue") {
        delayed(&f, v, |i| i.to_string())
    } else {
        association((1..=5).map(|i| (i, i.to_string())))
    };
    let mut b = data();
    let mut expected = association((1..=5).map(|i| (i, (i * 11).to_string())));
    for _ in 0..depth {
        a = format!("<|\"nested\"->{a}|>");
        b = format!("<|\"nested\"->{b}|>");
        expected = format!("Association[Rule[\"nested\", {expected}]]");
    }
    let reducer = if entry.ends_with("reducer") {
        format!("Function[x,{}]", visits(&f, v, "Total[x]"))
    } else {
        "Total".into()
    };
    f.source = format!("MergeNested[{{{a},{b}}},{reducer}]");
    finish(f, v, &expected)
}
