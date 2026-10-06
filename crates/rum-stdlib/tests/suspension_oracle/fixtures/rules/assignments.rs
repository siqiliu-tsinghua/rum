//! Contract: guide/evaluation.md, definition identity, destructuring and updates.
use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn target(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let name = entry.split_once('.').unwrap().0;
    let args = (1..=5)
        .map(|i| step(&mut f, v, i, &i.to_string()))
        .collect::<Vec<_>>()
        .join(",");
    let tagged = matches!(
        name,
        "TagSet" | "TagSetDelayed" | "TagUnset" | "UpSet" | "UpSetDelayed"
    );
    let lhs = if tagged {
        format!("Global`fn[Global`tag,{args}]")
    } else {
        format!("Global`fn[{args}]")
    };
    let lookup = if tagged {
        "Global`fn[Global`tag,1,2,3,4,5]"
    } else {
        "Global`fn[1,2,3,4,5]"
    };
    if name == "TagUnset" {
        f.setup
            .push_str(";TagSet[Global`tag,Global`fn[Global`tag,1,2,3,4,5],77]");
        f.source = format!("TagUnset[Global`tag,{lhs}]");
    } else if name.starts_with("Tag") {
        f.source = format!("{name}[Global`tag,{lhs},77]");
    } else {
        f.source = format!("{name}[{lhs},77]");
    }
    let raw = if tagged {
        "fn[tag, 1, 2, 3, 4, 5]"
    } else {
        "fn[1, 2, 3, 4, 5]"
    };
    let stored = (name != "TagUnset") != (v == Variant::Error);
    projection(&mut f, lookup, if stored { "77" } else { raw });
    projection(
        &mut f,
        if tagged {
            "Length[UpValues[Global`tag]]"
        } else {
            "Length[DownValues[Global`fn]]"
        },
        if stored { "1" } else { "0" },
    );
    finish(
        f,
        v,
        if name.ends_with("Delayed") || name == "TagUnset" {
            "Null"
        } else {
            "77"
        },
    )
}

pub fn destructuring(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    f.setup.push_str(";Global`a={0,0,0,0,0}");
    let leaves = (1..=5)
        .map(|i| {
            let selector = step(&mut f, v, i, &i.to_string());
            format!("Global`a[[{selector}]]")
        })
        .collect::<Vec<_>>();
    if v == Variant::Nested {
        f.source = format!(
            "{{{},{{{},{}}},{},{}}}={{11,{{12,13}},14,15}}",
            leaves[0], leaves[1], leaves[2], leaves[3], leaves[4]
        );
    } else {
        f.source = format!("{{{}}}={{11,12,13,14,15}}", leaves.join(","));
    }
    projection(
        &mut f,
        "Global`a",
        if v == Variant::Error {
            "List[11, 12, 13, 14, 0]"
        } else {
            "List[11, 12, 13, 14, 15]"
        },
    );
    finish(
        f,
        v,
        if v == Variant::Nested {
            "List[11, List[12, 13], 14, 15]"
        } else {
            "List[11, 12, 13, 14, 15]"
        },
    )
}

fn update_call(name: &str, target: &str, rhs: &str) -> String {
    if matches!(
        name,
        "Increment" | "Decrement" | "PreIncrement" | "PreDecrement"
    ) {
        format!("{name}[{target}]")
    } else {
        format!("{name}[{target},{rhs}]")
    }
}

pub fn update_target(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let name = entry.split_once('.').unwrap().0;
    let (initial, updated, rhs) = match name {
        "AddTo" => ("10", "12", "2"),
        "SubtractFrom" => ("10", "8", "2"),
        "TimesBy" => ("10", "20", "2"),
        "DivideBy" => ("10", "5", "2"),
        "ApplyTo" => ("10", "20", "Function[x,2*x]"),
        "AppendTo" => ("{10}", "List[10, 2]", "2"),
        "PrependTo" => ("{10}", "List[2, 10]", "2"),
        "Increment" | "PreIncrement" => ("10", "11", ""),
        "Decrement" | "PreDecrement" => ("10", "9", ""),
        _ => unreachable!(),
    };
    f.setup
        .push_str(&format!(";Global`a={{{{{{{{{{{initial}}}}}}}}}}}"));
    let selectors = (1..=5)
        .map(|i| step(&mut f, v, i, "1"))
        .collect::<Vec<_>>()
        .join(",");
    f.source = update_call(name, &format!("Global`a[[{selectors}]]"), rhs);
    let initial_value = if initial == "{10}" {
        "List[10]"
    } else {
        initial
    };
    projection(
        &mut f,
        "Global`a[[1,1,1,1,1]]",
        if v == Variant::Error {
            initial_value
        } else {
            updated
        },
    );
    finish(
        f,
        v,
        if matches!(name, "Increment" | "Decrement") {
            initial_value
        } else {
            updated
        },
    )
}

pub fn update(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let name = entry.split_once('.').unwrap().0;
    let body = sequence(&mut f, v, "77");
    let (initial, rhs) = match name {
        "AppendTo" | "PrependTo" => {
            f.setup.push_str(";Global`a=Global`token[1,2,3,4]");
            f.setup
                .push_str(&format!(";Global`token[_,_,_,_,_]:={body}"));
            ("token[1, 2, 3, 4]", "5".into())
        }
        "ApplyTo" => {
            f.setup.push_str(";Global`a=10");
            ("10", format!("Function[x,{body}]"))
        }
        _ => {
            f.setup.push_str(";Global`a=Global`token[]");
            let head = match name {
                "SubtractFrom" => "Subtract",
                "TimesBy" => "Times",
                "DivideBy" => "Divide",
                _ => "Plus",
            };
            f.setup.push_str(&format!(
                ";TagSetDelayed[Global`token,{head}[Global`token[],_],{body}]"
            ));
            ("token[]", "2".into())
        }
    };
    f.source = update_call(name, "Global`a", &rhs);
    projection(
        &mut f,
        "Global`a",
        if v == Variant::Error { initial } else { "77" },
    );
    finish(
        f,
        v,
        if matches!(name, "Increment" | "Decrement") {
            initial
        } else {
            "77"
        },
    )
}
