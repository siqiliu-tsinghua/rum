use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;
pub fn key(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, false, "key")
}
pub fn value(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, false, "value")
}
pub fn cross(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, false, "cross")
}
pub fn nested(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, false, "nested")
}
pub fn reducer(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, false, "reducer")
}
pub fn nested_reducer(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, false, "nested-reducer")
}
pub fn leaf_reducer(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, false, "leaf-reducer")
}
pub fn assoc_key(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, true, "key")
}
pub fn assoc_value(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, true, "value")
}
pub fn assoc_cross(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, true, "cross")
}
pub fn assoc_nested(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, true, "nested")
}
pub fn assoc_reducer(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, true, "reducer")
}
pub fn assoc_nested_reducer(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, true, "nested-reducer")
}
pub fn assoc_leaf_reducer(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, true, "leaf-reducer")
}
fn fixture(entry: &str, v: Variant, assoc: bool, phase: &str) -> Fixture {
    let mut f = base(entry, v);
    let source = if assoc { data() } else { "{1,2,3,4,5}".into() };
    let key = if assoc { "x/10" } else { "x" };
    let value = if assoc { "x+1" } else { "10*x" };
    let reducing = phase.contains("reducer");
    let mut spec = if reducing {
        format!("Function[x,{key}]")
    } else {
        format!("Function[x,{}]", visits(&f, v, key))
    };
    if phase == "value" {
        spec = format!("Function[x,{key}]->Function[x,{}]", visits(&f, v, value));
    } else if phase == "cross" {
        let before = (1..=3)
            .map(|i| step(&f, v, i, key))
            .collect::<Vec<_>>()
            .join(";");
        let after = (4..=5)
            .map(|i| step(&f, v, i, value))
            .collect::<Vec<_>>()
            .join(";");
        spec = format!(
            "Function[x,If[Global`state==0,({before}),{key}]]->Function[x,If[Global`state==3,({after}),{value}]]"
        );
    }
    if phase.starts_with("nested") {
        spec = format!("{{Function[x,0],{spec}}}");
    }
    if phase == "leaf-reducer" {
        spec = "{}".into();
    }
    let reducer = if assoc {
        "Total[Values[x]]"
    } else {
        "Total[x]"
    };
    let tail = if reducing {
        format!(
            ",Function[x,{}]",
            if phase == "leaf-reducer" {
                sequence(&f, v, reducer)
            } else {
                visits(&f, v, reducer)
            }
        )
    } else {
        String::new()
    };
    f.source = format!("GroupBy[{source},{spec}{tail}]");
    let mut expected = if phase == "leaf-reducer" {
        if assoc { "150" } else { "15" }.into()
    } else {
        association((1..=5).map(|i| {
            let n = if assoc {
                i * 10 + usize::from(phase == "value" || phase == "cross")
            } else if phase == "value" || phase == "cross" {
                i * 10
            } else {
                i
            };
            let value = if reducing {
                n.to_string()
            } else if assoc {
                association([(i, n.to_string())])
            } else {
                format!("List[{n}]")
            };
            (i, value)
        }))
    };
    if phase.starts_with("nested") {
        expected = format!("Association[Rule[0, {expected}]]");
    }
    finish(f, v, &expected)
}
pub fn key_group_reducer(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    f.source = format!(
        "KeyGroupBy[{},Identity,Function[x,{}]]",
        data(),
        visits(&f, v, "Total[Values[x]]")
    );
    finish(f, v, &data())
}
