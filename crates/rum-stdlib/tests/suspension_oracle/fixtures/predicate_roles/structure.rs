use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    if entry == "Differences.arithmetic" {
        return differences(entry, v, false);
    }
    let mut f = base(entry, v, 5);
    let callback = visits(&f, v, 5, "x");
    let lazy = format!("LazyMap[Function[x,{{x}}],LazyMap[Function[x,{callback}],LazyRange[1,1]]]");
    let (source, expected) = match entry {
        "First.lazyConsumer" => (format!("First[LazyDrop[{lazy},4]]"), "List[5]"),
        "Part.lazyConsumer" => (format!("{lazy}[[5]]"), "List[5]"),
        "Take.lazyConsumer" => (
            format!("Take[{lazy},5]"),
            "List[List[1], List[2], List[3], List[4], List[5]]",
        ),
        _ => unreachable!("{entry}"),
    };
    f.source = source;
    finish(f, v, expected)
}

pub fn differences(entry: &str, v: Variant, second_order: bool) -> Fixture {
    let mut f = base(entry, v, 5);
    f.setup.push_str(&format!(
        ";d/:Times[-1,d[x_]]:={};d/:Plus[d[y_],z_Integer]:=y+z",
        visits(&f, v, 5, "-x")
    ));
    f.source = format!(
        "Differences[{{d[1],d[2],d[3],d[4],d[5],d[6]}},{}]",
        if second_order { 2 } else { 1 }
    );
    finish(
        f,
        v,
        if second_order {
            "List[0, 0, 0, 0]"
        } else {
            "List[1, 1, 1, 1, 1]"
        },
    )
}
