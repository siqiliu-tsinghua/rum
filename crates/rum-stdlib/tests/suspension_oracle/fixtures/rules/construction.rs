use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let name = entry.split_once('.').unwrap().0;
    let args = (1..=5)
        .map(|i| step(&mut f, v, i, &i.to_string()))
        .collect::<Vec<_>>()
        .join(",");
    let rule = format!("h[z_,{args}]:>77");
    let value = match name {
        "NormalizeRules" => {
            f.source = format!("NormalizeRules[{rule}]");
            "List[RuleDelayed[h[Pattern[z, Blank[]], 1, 2, 3, 4, 5], 77]]"
        }
        "Rule" => {
            f.source = format!("Rule[h[z_,{args}],77]");
            "Rule[h[Pattern[z, Blank[]], 1, 2, 3, 4, 5], 77]"
        }
        "RuleDelayed" => {
            f.source = rule;
            "RuleDelayed[h[Pattern[z, Blank[]], 1, 2, 3, 4, 5], 77]"
        }
        _ => unreachable!(),
    };
    finish(f, v, value)
}
