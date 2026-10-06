use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    // Stable symbols are prepared eagerly without input. Only the Plus call
    // constructed by Total/Mean or Median's even-pair averaging fires this rule.
    let seq = sequence(&f, v, "146");
    f.setup
        .push_str(&format!(";Global`a /: Plus[Global`a,Global`b]:={seq}"));
    let name = entry.split_once('.').unwrap().0;
    f.source = format!("{name}[{{Global`a,Global`b}}]");
    finish(f, v, if name == "Total" { "146" } else { "73" })
}
