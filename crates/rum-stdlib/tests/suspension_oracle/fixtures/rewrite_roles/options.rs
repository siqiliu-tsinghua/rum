//! Stored defaults, explicit rhs evaluation and computed traversal options.
use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let (name, role) = entry.split_once('.').unwrap();
    if name == "ReplaceRepeated" {
        // A literal option is admitted before evaluating its rhs. The main
        // rewrite loop then makes five changing passes and one stable pass.
        assert_eq!(role, "explicitOption");
        let limit = sequence(&mut f, v, "10");
        f.source =
            format!("ReplaceRepeated[1,{{1->2,2->3,3->4,4->5,5->6}},MaxIterations->{limit}]");
        return finish(f, v, "6");
    }
    let value = sequence(&mut f, v, "True");
    let option = match role {
        "defaultOption" => {
            f.setup.push_str(&format!(
                ";Unprotect[{name}];SetOptions[{name},Heads:>{value}]"
            ));
            String::new()
        }
        "explicitOption" => format!(",Heads->{value}"),
        "optionSyntax" => format!(",{}", sequence(&mut f, v, "Heads->True")),
        _ => unreachable!(),
    };
    f.source = match name {
        "Cases" => format!("Cases[{{1,2,3,4,5}},List,{{1}}{option}]"),
        "DeleteCases" => format!("DeleteCases[{{1,2,3,4,5}},_Integer,{{1}}{option}]"),
        "Replace" => format!("Replace[{{1,2,3,4,5}},List->h,{{1}}{option}]"),
        _ => unreachable!(),
    };
    let expected = match name {
        "Cases" => "List[List]",
        "DeleteCases" => "List[]",
        "Replace" => "h[1, 2, 3, 4, 5]",
        _ => unreachable!(),
    };
    finish(f, v, expected)
}
