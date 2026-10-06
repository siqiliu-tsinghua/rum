//! Default reads, explicit rule rhs evaluation, and option syntax evaluation.
use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let (name, role) = entry.split_once('.').unwrap();
    let value = sequence(&mut f, v, "True");
    let option = match role {
        "defaultOption" => {
            f.setup.push_str(&format!(
                ";Unprotect[{name}];SetOptions[{name},Heads:>{value}]"
            ));
            String::new()
        }
        "explicitOption" => format!(",Heads->{value}"),
        "optionSyntax" => {
            // The whole option expression is evaluated by the traversal
            // resolver; it is not a callback hidden in an ordinary argument.
            let syntax = sequence(&mut f, v, "Heads->True");
            format!(",{syntax}")
        }
        _ => unreachable!(),
    };
    f.source = match name {
        "Level" => format!("Level[{{1,2,3,4,5}},{{1}}{option}]"),
        "FreeQ" => format!("FreeQ[{{1,2,3,4,5}},List{option}]"),
        _ => format!("{name}[{{1,2,3,4,5}},List,{{1}}{option}]"),
    };
    let expected = match name {
        "Count" => "1",
        "MemberQ" => "True",
        "FreeQ" => "False",
        "Position" => "List[List[0]]",
        "Level" => "List[List, 1, 2, 3, 4, 5]",
        _ => unreachable!(),
    };
    finish(f, v, expected)
}
