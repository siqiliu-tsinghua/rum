//! Ordinary preparation holds RuleDelayed values. The option resolver, not
//! argument preparation, executes these defaults and explicit delayed values.
use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v, 5);
    let (name, role) = entry.split_once('.').unwrap();
    let (option_name, value) = if name == "Depth" {
        ("Heads", "True")
    } else {
        ("SameTest", "SameQ")
    };
    let value = sequence(&f, v, value);
    let option = if role == "defaultOption" {
        f.setup.push_str(&format!(
            ";Unprotect[{name}];SetOptions[{name},{option_name}:>{value}]"
        ));
        String::new()
    } else {
        assert_eq!(role, "explicitOption");
        format!(",{option_name}:>{value}")
    };
    let (arguments, expected) = match name {
        "Depth" => ("f[g[h]][x]", "4"),
        "DuplicateFreeQ" => ("{1,2,3,4,5}", "True"),
        "Complement" => ("{1,2,3,4,5},{2,4}", "List[1, 3, 5]"),
        "ContainsNone" | "DisjointQ" => ("{1,2,3,4,5},{2,4}", "False"),
        "ContainsExactly" => ("{1,2,3,4,5},{5,4,3,2,1}", "True"),
        "ContainsOnly" => ("{2,4},{1,2,3,4,5}", "True"),
        _ => ("{1,2,3,4,5},{2,4}", "True"),
    };
    f.source = format!("{name}[{arguments}{option}]");
    finish(f, v, expected)
}
