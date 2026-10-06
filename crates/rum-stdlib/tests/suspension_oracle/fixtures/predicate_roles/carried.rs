//! The selected branch/handler is a documented Tail. Its continuation belongs
//! to the selected guest expression and any enclosing caller.
use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v, 5);
    let body = sequence(&f, v, "73");
    f.source = match entry {
        "If.branch" => format!("If[True,{body},999]"),
        "Which.branch" => format!("Which[False,999,True,{body},True,999]"),
        "Switch.branch" => format!("Switch[7,0,999,7,{body},_,999]"),
        "Catch.handler" => {
            format!("Catch[Throw[7,tag],tag,Function[{{value,tag}},({body};value+66)]]")
        }
        "Enclose.handler" => format!(
            "Enclose[Confirm[$Failed],Function[failure,({body};If[FailureQ[failure],73,999])]]"
        ),
        _ => unreachable!("{entry}"),
    };
    finish(f, v, "73")
}
